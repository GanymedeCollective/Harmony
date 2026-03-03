//! Converts IRC protocol messages to core types.

use bridge_core::{Channel, Message, User};
use irc::proto::Command;

pub fn irc_to_core(msg: &irc::proto::Message) -> Option<Message> {
    match &msg.command {
        Command::PRIVMSG(channel, text) => {
            let nickname = msg.source_nickname().unwrap_or("unknown").to_owned();

            Some(Message {
                author: User {
                    id: Some(nickname.clone()),
                    name: nickname,
                    display_name: None,
                    avatar_url: None,
                },
                channel: Channel {
                    id: channel.clone(),
                    name: channel.clone(),
                },
                content: text.clone(),
                attachments: Vec::new(),
            })
        }
        _ => None,
    }
}
