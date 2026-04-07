//! Starts a Serenity client, produces a `PlatformHandle`.
use crate::{
    proxy::{DiscordSendProxy, SendRequest},
    sender::DiscordSender,
};

use {
    exn::{Exn, ResultExt as _},
    harmony_core::{
        BoxFuture, HarmonyError, MetaEvent, PlatformAdapter, PlatformHandle, PlatformId,
        PlatformMessage, SendMessage,
    },
    std::sync::Arc,
    tokio::sync::{mpsc, oneshot},
};

pub struct DiscordAdapter {
    token: String,
    platform_id: PlatformId,
}

impl DiscordAdapter {
    #[must_use]
    pub fn new(token: String) -> Self {
        Self {
            token,
            platform_id: PlatformId::new("discord"),
        }
    }
}

impl PlatformAdapter for DiscordAdapter {
    fn platform_id(&self) -> &PlatformId {
        &self.platform_id
    }

    fn start(
        self: Box<Self>,
        msg_tx: mpsc::Sender<(PlatformId, PlatformMessage)>,
        event_tx: mpsc::Sender<MetaEvent>,
    ) -> BoxFuture<'static, Result<PlatformHandle, Exn<HarmonyError>>> {
        Box::pin(async move {
            let platform_id = self.platform_id.clone();
            let intents = serenity::all::GatewayIntents::GUILD_MESSAGES
                | serenity::all::GatewayIntents::DIRECT_MESSAGES
                | serenity::all::GatewayIntents::MESSAGE_CONTENT
                | serenity::all::GatewayIntents::GUILDS
                | serenity::all::GatewayIntents::GUILD_MEMBERS;

            let handler = crate::handler::Handler {
                msg_tx,
                event_tx,
                platform_id: platform_id.clone(),
                bot_user_id: std::sync::OnceLock::new(),
            };

            let mut client = serenity::Client::builder(&self.token, intents)
                .event_handler(handler)
                .await
                .or_raise(|| HarmonyError::connection("discord client setup failed"))?;

            let shard_manager = Arc::clone(&client.shard_manager);
            let http = Arc::clone(&client.http);
            let sender = DiscordSender::new(http, platform_id.clone());

            let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

            tokio::spawn(async move {
                if let Err(e) = client.start().await {
                    log::error!("discord: client error: {e}");
                }
            });

            let lister = Arc::new(sender.clone());

            let sm = shard_manager;
            let (send_tx, mut send_rx) = mpsc::channel(256);
            tokio::spawn(async move {
                let mut shutdown = shutdown_rx;
                loop {
                    tokio::select! {
                        req = send_rx.recv() => {
                            let Some(req) : Option<SendRequest> = req else { break };
                            let result = sender.send_message(&req.message).await;
                            let _ = req.response_tx.send(result);
                        }
                        _ = &mut shutdown => {
                            sm.shutdown_all().await;
                            break;
                        }
                    }
                }
            });

            Ok(PlatformHandle {
                id: platform_id,
                sender: Box::new(DiscordSendProxy { tx: send_tx }),
                user_lister: Box::new(Arc::clone(&lister)),
                channel_lister: Box::new(lister),
                shutdown_tx,
            })
        })
    }
}
