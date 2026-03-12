//! Converts IRC protocol messages to core types.

use harmony_core::{
    PlatformChannel, PlatformId, PlatformMessage, PlatformMessageRope, PlatformMessageSegment,
    PlatformUser,
};
use irc::proto::Command;

/// Parses a message text into a [`PlatformMessageRope`].
///
/// `@word` tokens are emitted as [`PlatformMessageSegment::Mention`] candidates
/// carrying the nickname (the part after `@`). Core resolves them later.
fn parse_message(text: &str) -> PlatformMessageRope {
    let mention_candidates: Vec<usize> = text.match_indices('@').map(|m| m.0).collect();
    let mut cursor = 0;
    let mut rope = PlatformMessageRope::new();

    for mention_start in mention_candidates {
        let text_part = &text[cursor..mention_start];
        rope.push(PlatformMessageSegment::Text(text_part.to_string()));

        let mention_end = text[mention_start..]
            .find(' ')
            .map_or(text.len(), |i| mention_start + i);

        let nickname = &text[mention_start + 1..mention_end];
        rope.push(PlatformMessageSegment::Mention(nickname.to_string()));
        cursor = mention_end;
    }

    if cursor < text.len() {
        rope.push(PlatformMessageSegment::Text(text[cursor..].to_string()));
    }

    rope
}

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
                content: parse_message(text),
            })
        }
        _ => None,
    }
}
