//! Converts Serenity messages to core types.

use harmony_core::{
    PlatformChannel, PlatformId, PlatformMessage, PlatformMessageRope, PlatformMessageSegment,
    PlatformUser,
};
use serenity::model::channel::Message as SerenityMessage;

fn parse_message(text: &str) -> PlatformMessageRope {
    let mention_candidates: Vec<usize> = text.match_indices("<@").map(|m| m.0).collect();
    let mut cursor = 0;
    let mut rope = PlatformMessageRope::new();

    for mention_start in mention_candidates {
        let Some(close_offset) = text[mention_start..].find('>') else {
            break;
        };
        let mention_end = mention_start + close_offset + 1;

        let mut inner = &text[mention_start + 2..mention_end - 1];

        if inner.starts_with('&') {
            continue;
        }

        if inner.starts_with('!') {
            inner = &inner[1..];
        }

        if inner.is_empty() {
            continue;
        }

        if cursor < mention_start {
            rope.push(PlatformMessageSegment::Text(
                text[cursor..mention_start].to_owned(),
            ));
        }
        rope.push(PlatformMessageSegment::Mention(inner.to_owned()));
        cursor = mention_end;
    }

    if cursor < text.len() {
        rope.push(PlatformMessageSegment::Text(text[cursor..].to_owned()));
    }

    rope
}

pub fn discord_to_core(msg: &SerenityMessage, platform_id: &PlatformId) -> PlatformMessage {
    let mut content = msg.content.clone();
    for attachment in &msg.attachments {
        if !content.is_empty() {
            content.push(' ');
        }
        content.push_str(&attachment.url);
    }

    let display_name = msg
        .member
        .as_ref()
        .and_then(|m| m.nick.clone())
        .or_else(|| msg.author.global_name.clone())
        .or_else(|| Some(msg.author.name.clone()));

    PlatformMessage {
        author: PlatformUser {
            platform: platform_id.clone(),
            id: msg.author.id.get().to_string(),
            display_name,
            avatar_url: msg.author.avatar_url(),
        },
        channel: PlatformChannel {
            platform: platform_id.clone(),
            id: msg.channel_id.get().to_string(),
            name: msg.channel_id.get().to_string(),
        },
        content: parse_message(&content),
    }
}
