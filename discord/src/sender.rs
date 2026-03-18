//! Sends bridged messages via per-channel webhooks, and implements listing capabilities.

use std::collections::HashMap;
use std::sync::Arc;

use exn::{Exn, OptionExt as _, ResultExt as _};
use harmony_core::{
    BoxFuture, CoreMessage, HarmonyError, ListChannels, ListUsers, PlatformChannel, PlatformId,
    PlatformUser, SendMessage,
};
use serenity::builder::{CreateWebhook, ExecuteWebhook};
use serenity::model::id::ChannelId;
use serenity::model::webhook::Webhook;
use tokio::sync::RwLock;

const WEBHOOK_NAME: &str = "Bridge";

#[derive(Clone)]
pub struct DiscordSender {
    pub(crate) http: Arc<serenity::http::Http>,
    pub(crate) platform_id: PlatformId,
    webhooks: Arc<RwLock<HashMap<u64, Webhook>>>,
}

impl DiscordSender {
    pub fn new(http: Arc<serenity::http::Http>, platform_id: PlatformId) -> Self {
        Self {
            http,
            platform_id,
            webhooks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn get_or_create_webhook(
        http: &serenity::http::Http,
        channel_id: u64,
    ) -> Result<Webhook, Exn<HarmonyError>> {
        let cid = ChannelId::new(channel_id);
        let existing = cid.webhooks(http).await.or_raise(|| {
            HarmonyError::send(format!("webhook setup failed for channel {channel_id}"))
        })?;
        if let Some(wh) = existing
            .into_iter()
            .find(|w| w.name.as_deref() == Some(WEBHOOK_NAME))
        {
            log::trace!("discord: reusing existing webhook in channel {channel_id}");
            Ok(wh)
        } else {
            log::debug!("discord: creating webhook in channel {channel_id}");
            Ok(cid
                .create_webhook(http, CreateWebhook::new(WEBHOOK_NAME))
                .await
                .or_raise(|| {
                    HarmonyError::send(format!("webhook setup failed for channel {channel_id}"))
                })?)
        }
    }

    async fn ensure_webhook(&self, channel_id: u64) -> Result<Webhook, Exn<HarmonyError>> {
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
        drop(whs);
        Ok(wh)
    }
}

impl SendMessage for DiscordSender {
    fn send_message<'a>(
        &'a self,
        message: &'a CoreMessage,
    ) -> BoxFuture<'a, Result<(), Exn<HarmonyError>>> {
        Box::pin(async move {
            let channel = message
                .channel
                .get_platform_channel(&self.platform_id)
                .ok_or_raise(|| {
                    HarmonyError::send("no channel alias for this platform").permanent()
                })?;
            let channel_id: u64 = channel
                .id
                .parse::<u64>()
                .or_raise(|| HarmonyError::internal("invalid channel id").permanent())?;
            let webhook = self.ensure_webhook(channel_id).await?;

            let display_name = message
                .author
                .get_platform_user(&self.platform_id)
                .and_then(|pu| pu.display_name.as_deref())
                .or_else(|| message.author.display_name())
                .unwrap_or("unknown");

            let avatar_url = message
                .author
                .get_platform_user(&self.platform_id)
                .and_then(|pu| pu.avatar_url.as_deref())
                .or_else(|| message.author.avatar_url());

            let mut exec = ExecuteWebhook::new()
                .content(&message.content)
                .username(display_name);
            if let Some(url) = avatar_url {
                exec = exec.avatar_url(url);
            }
            webhook
                .execute(&self.http, false, exec)
                .await
                .or_raise(|| HarmonyError::send("discord message relay failed"))?;
            Ok(())
        })
    }
}

impl ListUsers for DiscordSender {
    fn list_users(&self) -> BoxFuture<'_, Result<Vec<PlatformUser>, Exn<HarmonyError>>> {
        Box::pin(async {
            let (_channels, users) =
                crate::fetch::fetch_guild_data(&self.http, &self.platform_id).await?;
            Ok(users)
        })
    }
}

impl ListChannels for DiscordSender {
    fn list_channels(&self) -> BoxFuture<'_, Result<Vec<PlatformChannel>, Exn<HarmonyError>>> {
        Box::pin(async {
            let (channels, _users) =
                crate::fetch::fetch_guild_data(&self.http, &self.platform_id).await?;
            Ok(channels)
        })
    }
}
