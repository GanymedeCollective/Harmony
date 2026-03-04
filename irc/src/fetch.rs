//! One-shot IRC connection to discover channels and users via LIST/NAMES.

use std::collections::HashSet;
use std::time::Duration;

use anyhow::Result;
use bridge_core::{Channel, User};
use futures::prelude::*;
use irc::client::Client;
use irc::client::data::Config as IrcConfig;
use irc::proto::{Command, Response};

pub async fn fetch_data(
    config: IrcConfig,
    bot_nickname: &str,
) -> Result<(Vec<Channel>, Vec<User>)> {
    let mut client = Client::from_config(config).await?;
    client.identify()?;

    let raw = client.sender();
    let mut stream = client.stream()?;
    let mut channels: Vec<Channel> = Vec::new();
    let mut nicknames = HashSet::new();
    let mut list_done = false;
    let mut names_pending: usize = 0;

    log::info!("irc: connecting and waiting for registration...");

    let result = tokio::time::timeout(Duration::from_secs(60), async {
        while let Some(result) = stream.next().await {
            match result {
                Ok(msg) => match &msg.command {
                    Command::Response(Response::RPL_ENDOFMOTD | Response::ERR_NOMOTD, _) => {
                        log::info!("irc: registered, discovering channels via LIST");
                        raw.send(Command::LIST(None, None))?;
                    }
                    Command::Response(Response::RPL_LIST, args) => {
                        if let Some(name) = args.get(1) {
                            channels.push(Channel {
                                id: name.clone(),
                                name: name.clone(),
                            });
                        }
                    }
                    Command::Response(Response::RPL_LISTEND, _) => {
                        list_done = true;
                        names_pending = channels.len();
                        log::info!(
                            "irc: found {} channel(s), querying NAMES...",
                            channels.len()
                        );
                        for ch in &channels {
                            raw.send(Command::NAMES(Some(ch.name.clone()), None))?;
                        }
                        if names_pending == 0 {
                            break;
                        }
                    }
                    Command::Response(Response::RPL_NAMREPLY, args) if list_done => {
                        if let Some(names_str) = args.last() {
                            for raw_nick in names_str.split_whitespace() {
                                let nick = raw_nick.trim_start_matches(['@', '+', '%', '~', '&']);
                                if !nick.eq_ignore_ascii_case(bot_nickname) {
                                    nicknames.insert(nick.to_owned());
                                }
                            }
                        }
                    }
                    Command::Response(Response::RPL_ENDOFNAMES, _) if list_done => {
                        names_pending = names_pending.saturating_sub(1);
                        if names_pending == 0 {
                            break;
                        }
                    }
                    _ => {}
                },
                Err(e) => {
                    log::error!("irc: stream error during fetch: {e}");
                    break;
                }
            }
        }
        anyhow::Ok(())
    })
    .await;

    if result.is_err() {
        log::warn!("irc: fetch_data timed out after 60s");
    }

    let _ = raw.send(Command::QUIT(Some("fetch complete".to_owned())));

    let mut users: Vec<User> = nicknames
        .into_iter()
        .map(|nick| User {
            id: Some(nick.clone()),
            name: nick,
            display_name: None,
            avatar_url: None,
        })
        .collect();
    users.sort_by(|a, b| a.name.cmp(&b.name));

    log::info!(
        "irc: fetched {} channel(s) and {} unique user(s)",
        channels.len(),
        users.len()
    );

    Ok((channels, users))
}
