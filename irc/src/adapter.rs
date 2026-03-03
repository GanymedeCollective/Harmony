//! Connects to IRC, spawns the message stream, produces a `PlatformHandle`.

use bridge_core::{
    BoxFuture, Channel, Message, MetaEvent, PlatformAdapter, PlatformHandle, PlatformId, User,
};
use futures::prelude::*;
use irc::client::{Client, ClientStream};
use irc::proto::{Command, Response};
use tokio::sync::{mpsc, oneshot};

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
        msg_tx: mpsc::Sender<(PlatformId, Message)>,
        event_tx: mpsc::Sender<MetaEvent>,
    ) -> BoxFuture<'static, Result<PlatformHandle, Box<dyn std::error::Error + Send + Sync>>> {
        Box::pin(async move {
            let platform_id = self.platform_id.clone();
            let mut client = Client::from_config(self.config).await?;
            client.identify()?;

            let raw_sender = client.sender();
            let sender = IrcSender {
                inner: raw_sender.clone(),
            };

            let stream = client.stream()?;
            let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

            let pid = platform_id.clone();
            let bot_nickname = self.nickname;

            tokio::spawn(async move {
                tokio::select! {
                () = process_stream(stream, msg_tx, event_tx, pid, bot_nickname) => {}
                    _ = shutdown_rx => {
                        let _ = raw_sender.send(Command::QUIT(Some("Bridge shutting down".to_owned())));
                    }
                }
            });

            Ok(PlatformHandle {
                id: platform_id,
                sender: Box::new(sender),
                shutdown_tx,
            })
        })
    }

    fn fetch(
        &self,
    ) -> BoxFuture<'_, Result<(Vec<Channel>, Vec<User>), Box<dyn std::error::Error + Send + Sync>>>
    {
        Box::pin(async {
            let mut config = self.config.clone();
            config.channels = vec![];
            let result = crate::fetch::fetch_data(config, &self.nickname).await?;
            Ok(result)
        })
    }
}

// TODO: logic should probably be split accordingly, so we can remove the #[allow(...)]
#[allow(clippy::too_many_lines)]
async fn process_stream(
    mut stream: ClientStream,
    msg_tx: mpsc::Sender<(PlatformId, Message)>,
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
                let Some(core_msg) = crate::convert::irc_to_core(&msg) else {
                    continue;
                };
                if core_msg.author.name == bot_nickname {
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
                let users: Vec<User> = names_str
                    .split_whitespace()
                    .filter_map(|raw| {
                        let nick = raw.trim_start_matches(['@', '+', '%', '~', '&']);
                        (!nick.eq_ignore_ascii_case(&bot_nickname)).then(|| User {
                            id: Some(nick.to_owned()),
                            name: nick.to_owned(),
                            display_name: None,
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
                        user: User {
                            id: Some(nick.to_owned()),
                            name: nick.to_owned(),
                            display_name: None,
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
                        new_name: new_nick.clone(),
                    })
                    .await;
            }
            _ => {}
        }
    }
}
