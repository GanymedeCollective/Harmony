//! Converts IRC protocol messages to core types.

use bridge_core::{
    PlatformChannel, PlatformId, PlatformMessage, PlatformMessageRope, PlatformMessageSegment,
    PlatformUser,
};
use irc::proto::Command;

/// Parses a message text into a [`PlatformMessageRope`].
fn parse_message(text: String, users: &Vec<PlatformUser>) -> PlatformMessageRope {
    let mention_candidates: Vec<usize> = text.match_indices('@').map(|m| m.0).collect();
    let mut cursor = 0;
    let mut rope = PlatformMessageRope::new();

    for mention_candidate in mention_candidates {
        // Everything between the end of the previous mention and the start of the current one is text
        let text_part = &text[cursor..mention_candidate];
        rope.push(PlatformMessageSegment::Text(text_part.to_string()));

        // find the next ' ' char after mention_candidate
        let mention_end = text[mention_candidate..]
            .find(' ')
            .map(|i| mention_candidate + i)
            .unwrap_or(text.len());
        let mention = text[mention_candidate..mention_end].to_string();

        let Some(user) = users.iter().find(|u| {
            u.display_name
                .as_deref()
                .is_some_and(|name| format!("@{name}") == mention)
        }) else {
            // We don't know this user, fallback to text
            log::error!("User not found with name {mention}");
            log::error!("{:?}", users);
            rope.push(PlatformMessageSegment::Text(mention));
            cursor = mention_end;
            continue;
        };

        log::error!("Found user with name {mention}");

        rope.push(PlatformMessageSegment::Mention(user.clone()));
        cursor = mention_end;
    }

    // push any remaining text after the last mention
    if cursor < text.len() {
        let text = text[cursor..].to_string();
        rope.push(PlatformMessageSegment::Text(text));
    }

    rope
}

pub fn irc_to_core(
    msg: &irc::proto::Message,
    platform_id: &PlatformId,
    users: &Vec<PlatformUser>,
) -> Option<PlatformMessage> {
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
                content: parse_message(text.clone(), users),
            })
        }
        _ => None,
    }
}
