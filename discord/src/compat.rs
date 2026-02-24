use std::sync::Arc;

use async_trait::async_trait;
use bridge_core::{Attachment, Channel, Message, MessageSender, User};
use serenity::model::channel::Message as SerenityMessage;
use serenity::model::id::ChannelId;

pub struct DiscordConfig {
    pub token: String,
    pub bot_user_id: Option<u64>,
}

#[derive(Clone)]
pub struct DiscordSender {
    pub(crate) http: Arc<serenity::http::Http>,
}

#[async_trait]
impl MessageSender for DiscordSender {
    async fn send_message(
        &self,
        target: &Channel,
        message: &Message,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let channel_id: u64 = target.id.parse()?;
        // FIXME: use the card thing for Discord
        let text = format!("**<{}>** {}", message.author.name, message.content);
        ChannelId::new(channel_id).say(&self.http, text).await?;
        Ok(())
    }
}

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

    Message {
        author: User {
            name: msg.author.name.clone(),
            avatar_url: msg.author.avatar_url().and_then(|u| u.parse().ok()),
        },
        channel: Channel {
            id: msg.channel_id.get().to_string(),
            name: msg.channel_id.get().to_string(),
        },
        content,
        attachments,
    }
}
