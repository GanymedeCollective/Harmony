//! Converts Serenity messages to core types.

use bridge_core::{
    PlatformChannel, PlatformId, PlatformMessage, PlatformMessageRope, PlatformMessageSegment,
    PlatformUser,
};
use serenity::model::channel::Message as SerenityMessage;

fn parse_message(text: String) -> PlatformMessageRope {
    let mention_candidates: Vec<usize> = text.match_indices("<@").map(|m| m.0).collect();
    let mut cursor = 0;
    let mut rope = PlatformMessageRope::new();

    for mention_candidate in mention_candidates {
        let text_part = &text[cursor..mention_candidate];
        rope.push(PlatformMessageSegment::Text(text_part.to_string()));

        let mention_end = text[mention_candidate..]
            .find('>')
            .map(|i| mention_candidate + i + 1)
            .unwrap_or(text.len());
        let mention = text[mention_candidate..mention_end].to_string();
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
        content: parse_message(content),
    }
}
