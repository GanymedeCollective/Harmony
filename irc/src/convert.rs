//! Converts IRC protocol messages to core types.

use harmony_core::{
    PlatformChannel, PlatformId, PlatformMessage, PlatformMessageRope, PlatformMessageSegment,
    PlatformUser,
};
use irc::proto::Command;

const MENTION_PREFIX: &str = "@";
const MENTION_SEPARATOR: char = ' ';

pub fn format_mention(id: &str) -> String {
    format!("{MENTION_PREFIX}{id}")
}

/// Parses a message text into a [`PlatformMessageRope`].
///
/// `@word` tokens are emitted as [`PlatformMessageSegment::Mention`] candidates
/// carrying the nickname (the part after `@`). Core resolves them later.
fn parse_message(text: &str) -> PlatformMessageRope {
    let mention_candidates: Vec<usize> = text.match_indices(MENTION_PREFIX).map(|m| m.0).collect();
    let mut cursor = 0;
    let mut rope = PlatformMessageRope::new();

    for mention_start in mention_candidates {
        if mention_start > 0 && !text.as_bytes()[mention_start - 1].is_ascii_whitespace() {
            continue;
        }

        let mention_end = text[mention_start..]
            .find(MENTION_SEPARATOR)
            .map_or(text.len(), |i| mention_start + i);

        let nickname = &text[mention_start + MENTION_PREFIX.len()..mention_end];
        if nickname.is_empty() {
            continue;
        }

        if cursor < mention_start {
            rope.push(PlatformMessageSegment::Text(
                text[cursor..mention_start].to_owned(),
            ));
        }
        rope.push(PlatformMessageSegment::Mention(nickname.to_owned()));
        cursor = mention_end;
    }

    if cursor < text.len() {
        rope.push(PlatformMessageSegment::Text(text[cursor..].to_owned()));
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
