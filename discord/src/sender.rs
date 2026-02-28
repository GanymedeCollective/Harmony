use std::collections::HashMap;
use std::sync::Arc;

use bridge_core::{BoxFuture, Channel, Message, MessageSender};
use serenity::builder::{CreateWebhook, ExecuteWebhook};
use serenity::model::id::ChannelId;
use serenity::model::webhook::Webhook;
use tokio::sync::RwLock;

const WEBHOOK_NAME: &str = "Bridge";

#[derive(Clone)]
pub struct DiscordSender {
    pub(crate) http: Arc<serenity::http::Http>,
    webhooks: Arc<RwLock<HashMap<u64, Webhook>>>,
}

impl DiscordSender {
    pub fn new(http: Arc<serenity::http::Http>) -> Self {
        Self {
            http,
            webhooks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn get_or_create_webhook(
        http: &serenity::http::Http,
        channel_id: u64,
    ) -> anyhow::Result<Webhook> {
        let cid = ChannelId::new(channel_id);
        let existing = cid.webhooks(http).await?;
        if let Some(wh) = existing
            .into_iter()
            .find(|w| w.name.as_deref() == Some(WEBHOOK_NAME))
        {
            log::debug!("discord: reusing existing webhook in channel {channel_id}");
            Ok(wh)
        } else {
            log::info!("discord: creating webhook in channel {channel_id}");
            Ok(cid
                .create_webhook(http, CreateWebhook::new(WEBHOOK_NAME))
                .await?)
        }
    }

    async fn ensure_webhook(
        &self,
        channel_id: u64,
    ) -> Result<Webhook, Box<dyn std::error::Error + Send + Sync>> {
        {
            let whs = self.webhooks.read().await;
            if let Some(wh) = whs.get(&channel_id) {
                return Ok(wh.clone());
            }
        }
        let mut whs = self.webhooks.write().await;
        if let Some(wh) = whs.get(&channel_id) {
            return Ok(wh.clone());
        }
        let wh = Self::get_or_create_webhook(&self.http, channel_id).await?;
        whs.insert(channel_id, wh.clone());
        Ok(wh)
    }
}

impl MessageSender for DiscordSender {
    fn send_message<'a>(
        &'a self,
        target: &'a Channel,
        message: &'a Message,
    ) -> BoxFuture<'a, Result<(), Box<dyn std::error::Error + Send + Sync>>> {
        Box::pin(async move {
            let channel_id: u64 = target.id.parse()?;
            let webhook = self.ensure_webhook(channel_id).await?;

            let mut exec = ExecuteWebhook::new()
                .content(&message.content)
                .username(&message.author.name);
            if let Some(avatar_url) = &message.author.avatar_url {
                exec = exec.avatar_url(avatar_url);
            }
            webhook.execute(&self.http, false, exec).await?;
            Ok(())
        })
    }
}
