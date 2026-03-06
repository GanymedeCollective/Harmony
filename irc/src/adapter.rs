//! Connects to IRC, spawns the message stream, produces a `PlatformHandle`.

use std::collections::HashSet;
use std::time::Duration;

use bridge_core::{
    BoxFuture, MetaEvent, PlatformAdapter, PlatformChannel, PlatformHandle, PlatformId,
    PlatformMessage, PlatformUser,
};
use futures::prelude::*;
use irc::client::{Client, ClientStream, Sender as RawSender};
use irc::proto::{Command, Response};
use tokio::sync::{mpsc, oneshot};

use crate::lister::IrcLister;
use crate::sender::IrcSender;

pub use irc::client::data::Config as IrcConfig;

pub struct IrcAdapter {
    config: IrcConfig,
    nickname: String,
    platform_id: PlatformId,
}

impl IrcAdapter {
    #[must_use]
    pub fn new(config: IrcConfig, nickname: String) -> Self {
        Self {
            config,
            nickname,
            platform_id: PlatformId::new("irc"),
        }
    }
}

impl PlatformAdapter for IrcAdapter {
    fn platform_id(&self) -> &PlatformId {
        &self.platform_id
    }

    fn start(
        self: Box<Self>,
        msg_tx: mpsc::Sender<(PlatformId, PlatformMessage)>,
        event_tx: mpsc::Sender<MetaEvent>,
    ) -> BoxFuture<'static, Result<PlatformHandle, Box<dyn std::error::Error + Send + Sync>>> {
        Box::pin(async move {
            let platform_id = self.platform_id.clone();
            let mut config = self.config;
            config.channels = vec![];
            let mut client = Client::from_config(config).await?;
            client.identify()?;

            let raw_sender = client.sender();
            let sender = IrcSender {
                inner: raw_sender.clone(),
                platform_id: platform_id.clone(),
            };

            let mut stream = client.stream()?;
            let bot_nickname = self.nickname;

            let (channels, users) =
                discover_and_join(&raw_sender, &mut stream, &platform_id, &bot_nickname).await;

            let lister = IrcLister { channels, users };

            let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
            let pid = platform_id.clone();
            let bn = bot_nickname;

            tokio::spawn(async move {
                tokio::select! {
                    () = process_stream(stream, msg_tx, event_tx, pid, bn) => {}
                    _ = shutdown_rx => {
                        let _ = raw_sender.send(Command::QUIT(Some("Bridge shutting down".to_owned())));
                    }
                }
            });

            Ok(PlatformHandle {
                id: platform_id,
                sender: Box::new(sender),
                user_lister: Box::new(lister.clone()),
                channel_lister: Box::new(lister),
                shutdown_tx,
            })
        })
    }
}

/// Discover channels via LIST, join them all, and collect users from NAMES
/// replies (sent automatically by the server after each JOIN).
async fn discover_and_join(
    raw: &RawSender,
    stream: &mut ClientStream,
    platform_id: &PlatformId,
    bot_nickname: &str,
) -> (Vec<PlatformChannel>, Vec<PlatformUser>) {
    let mut channels: Vec<PlatformChannel> = Vec::new();
    let mut nicknames: HashSet<String> = HashSet::new();
    let mut joining = false;
    let mut names_pending: usize = 0;

    log::info!("irc: waiting for registration...");

    let result = tokio::time::timeout(Duration::from_mins(1), async {
        while let Some(result) = stream.next().await {
            let msg = match result {
                Ok(msg) => msg,
                Err(e) => {
                    log::error!("irc: stream error during discovery: {e}");
                    break;
                }
            };

            match &msg.command {
                Command::Response(Response::RPL_ENDOFMOTD | Response::ERR_NOMOTD, _) => {
                    log::info!("irc: registered, discovering channels via LIST");
                    if let Err(e) = raw.send(Command::LIST(None, None)) {
                        log::error!("irc: failed to send LIST: {e}");
                        break;
                    }
                }
                Command::Response(Response::RPL_LIST, args) => {
                    if let Some(name) = args.get(1) {
                        channels.push(PlatformChannel {
                            platform: platform_id.clone(),
                            id: name.clone(),
                            name: name.clone(),
                        });
                    }
                }
                Command::Response(Response::RPL_LISTEND, _) => {
                    names_pending = channels.len();
                    log::info!("irc: found {} channel(s), joining all...", channels.len());
                    for ch in &channels {
                        if let Err(e) = raw.send(Command::JOIN(ch.name.clone(), None, None)) {
                            log::error!("irc: failed to join {}: {e}", ch.name);
                            names_pending = names_pending.saturating_sub(1);
                        }
                    }
                    joining = true;
                    if names_pending == 0 {
                        break;
                    }
                }
                Command::Response(Response::RPL_NAMREPLY, args) if joining => {
                    if let Some(names_str) = args.last() {
                        for raw_nick in names_str.split_whitespace() {
                            let nick = raw_nick.trim_start_matches(['@', '+', '%', '~', '&']);
                            if !nick.eq_ignore_ascii_case(bot_nickname) {
                                nicknames.insert(nick.to_owned());
                            }
                        }
                    }
                }
                Command::Response(Response::RPL_ENDOFNAMES, _) if joining => {
                    names_pending = names_pending.saturating_sub(1);
                    if names_pending == 0 {
                        break;
                    }
                }
                _ => {}
            }
        }
    })
    .await;

    if result.is_err() {
        log::warn!("irc: discovery timed out after 60s");
    }

    let mut users: Vec<PlatformUser> = nicknames
        .into_iter()
        .map(|nick| PlatformUser {
            platform: platform_id.clone(),
            id: nick.clone(),
            display_name: Some(nick),
            avatar_url: None,
        })
        .collect();
    users.sort_by(|a, b| a.id.cmp(&b.id));

    log::info!(
        "irc: joined {} channel(s), found {} unique user(s)",
        channels.len(),
        users.len()
    );

    (channels, users)
}

// TODO: logic should probably be split accordingly, so we can remove the #[allow(...)]
#[allow(clippy::too_many_lines)]
async fn process_stream(
    mut stream: ClientStream,
    msg_tx: mpsc::Sender<(PlatformId, PlatformMessage)>,
    event_tx: mpsc::Sender<MetaEvent>,
    pid: PlatformId,
    mut bot_nickname: String,
) {
    while let Some(result) = stream.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                log::error!("irc: stream error: {e}");
                break;
            }
        };

        match &msg.command {
            Command::PRIVMSG(_, _) => {
                let Some(core_msg) = crate::convert::irc_to_core(&msg, &pid) else {
                    continue;
                };
                if core_msg.author.id == bot_nickname {
                    continue;
                }
                if msg_tx.send((pid.clone(), core_msg)).await.is_err() {
                    log::warn!("irc: receiver dropped, stopping stream");
                    break;
                }
            }
            Command::Response(Response::RPL_NAMREPLY, args) => {
                let Some(names_str) = args.last() else {
                    continue;
                };
                let users: Vec<PlatformUser> = names_str
                    .split_whitespace()
                    .filter_map(|raw| {
                        let nick = raw.trim_start_matches(['@', '+', '%', '~', '&']);
                        (!nick.eq_ignore_ascii_case(&bot_nickname)).then(|| PlatformUser {
                            platform: pid.clone(),
                            id: nick.to_owned(),
                            display_name: Some(nick.to_owned()),
                            avatar_url: None,
                        })
                    })
                    .collect();
                if !users.is_empty() {
                    let _ = event_tx
                        .send(MetaEvent::UsersDiscovered {
                            platform: pid.clone(),
                            users,
                        })
                        .await;
                }
            }
            Command::JOIN(_, _, _) => {
                let Some(nick) = msg.source_nickname() else {
                    continue;
                };
                if nick.eq_ignore_ascii_case(&bot_nickname) {
                    continue;
                }
                let _ = event_tx
                    .send(MetaEvent::UserJoined {
                        platform: pid.clone(),
                        user: PlatformUser {
                            platform: pid.clone(),
                            id: nick.to_owned(),
                            display_name: Some(nick.to_owned()),
                            avatar_url: None,
                        },
                    })
                    .await;
            }
            Command::QUIT(_) => {
                let Some(nick) = msg.source_nickname() else {
                    continue;
                };
                if nick.eq_ignore_ascii_case(&bot_nickname) {
                    continue;
                }
                let _ = event_tx
                    .send(MetaEvent::UserLeft {
                        platform: pid.clone(),
                        id: nick.to_owned(),
                    })
                    .await;
            }
            Command::NICK(new_nick) => {
                let Some(old_nick) = msg.source_nickname() else {
                    continue;
                };
                if old_nick.eq_ignore_ascii_case(&bot_nickname) {
                    bot_nickname.clone_from(new_nick);
                    continue;
                }
                let _ = event_tx
                    .send(MetaEvent::UserRenamed {
                        platform: pid.clone(),
                        old_id: old_nick.to_owned(),
                        new_id: new_nick.clone(),
                        new_display_name: Some(new_nick.clone()),
                    })
                    .await;
            }
            _ => {}
        }
    }
}
