//! Converts Serenity messages to core types.

use bridge_core::{Attachment, Channel, Message, User};
use serenity::model::channel::Message as SerenityMessage;

pub(crate) fn discord_to_core(msg: &SerenityMessage) -> Message {
    let attachments: Vec<Attachment> = msg
        .attachments
        .iter()
        .filter_map(|a| {
            let url = a.url.parse().ok()?;
            Some(Attachment {
                url,
                filename: a.filename.clone(),
            })
        })
        .collect();

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
        .or_else(|| msg.author.global_name.clone());

    Message {
        author: User {
            id: Some(msg.author.id.get().to_string()),
            name: msg.author.name.clone(),
            display_name,
            avatar_url: msg.author.avatar_url(),
        },
        channel: Channel {
            id: msg.channel_id.get().to_string(),
            name: msg.channel_id.get().to_string(),
        },
        content,
        attachments,
    }
}
