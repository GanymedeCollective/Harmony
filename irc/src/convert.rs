//! Converts IRC protocol messages to core types.

use harmony_core::{PlatformChannel, PlatformId, PlatformMessage, PlatformUser};
use irc::proto::Command;

pub fn irc_to_core(msg: &irc::proto::Message, platform_id: &PlatformId) -> Option<PlatformMessage> {
    match &msg.command {
        Command::PRIVMSG(channel, text) => {
            let nickname = msg.source_nickname().unwrap_or("unknown").to_owned();

            Some(PlatformMessage {
                author: PlatformUser {
                    platform: platform_id.clone(),
                    id: nickname.clone(),
                    display_name: Some(nickname),
                    avatar_url: None,
                },
                channel: PlatformChannel {
                    platform: platform_id.clone(),
                    id: channel.clone(),
                    name: channel.clone(),
                },
                content: text.clone(),
            })
        }
        _ => None,
    }
}
