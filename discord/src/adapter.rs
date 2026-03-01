//! Starts a Serenity client, produces a `PlatformHandle`.

use bridge_core::{
    BoxFuture, Channel, Message, MetaEvent, PlatformAdapter, PlatformHandle, PlatformId, User,
};
use tokio::sync::{mpsc, oneshot};

use crate::sender::DiscordSender;

pub struct DiscordAdapter {
    token: String,
    platform_id: PlatformId,
}

impl DiscordAdapter {
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
        msg_tx: mpsc::Sender<(PlatformId, Message)>,
        event_tx: mpsc::Sender<MetaEvent>,
    ) -> BoxFuture<'static, Result<PlatformHandle, Box<dyn std::error::Error + Send + Sync>>> {
        Box::pin(async move {
            let platform_id = self.platform_id.clone();
            let intents = serenity::all::GatewayIntents::GUILD_MESSAGES
                | serenity::all::GatewayIntents::DIRECT_MESSAGES
                | serenity::all::GatewayIntents::MESSAGE_CONTENT
                | serenity::all::GatewayIntents::GUILDS
                | serenity::all::GatewayIntents::GUILD_MEMBERS;

            let handler = crate::handler::Handler {
                msg_tx,
                event_tx: event_tx.clone(),
                platform_id: platform_id.clone(),
                bot_user_id: std::sync::OnceLock::new(),
            };

            let mut client = serenity::Client::builder(&self.token, intents)
                .event_handler(handler)
                .await?;

            let http = client.http.clone();
            let shard_manager = client.shard_manager.clone();
            let sender = DiscordSender::new(http.clone());

            let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

            tokio::spawn(async move {
                if let Err(e) = client.start().await {
                    log::error!("discord: client error: {e}");
                }
            });

            let sm = shard_manager.clone();
            tokio::spawn(async move {
                let _ = shutdown_rx.await;
                sm.shutdown_all().await;
            });

            let pid = platform_id.clone();
            tokio::spawn(async move {
                log::info!("discord: performing initial data fetch...");
                match crate::fetch::fetch_guild_data(&http).await {
                    Ok((channels, users)) => {
                        for ch in channels {
                            let _ = event_tx
                                .send(MetaEvent::ChannelCreated {
                                    platform: pid.clone(),
                                    id: ch.id,
                                    name: ch.name,
                                })
                                .await;
                        }
                        if !users.is_empty() {
                            let _ = event_tx
                                .send(MetaEvent::UsersDiscovered {
                                    platform: pid.clone(),
                                    users,
                                })
                                .await;
                        }
                        log::info!("discord: initial data fetch complete");
                    }
                    Err(e) => log::error!("discord: initial data fetch failed: {e}"),
                }
            });

            Ok(PlatformHandle {
                id: platform_id,
                sender: Box::new(sender),
                shutdown_tx,
            })
        })
    }

    fn fetch(
        &self,
    ) -> BoxFuture<'_, Result<(Vec<Channel>, Vec<User>), Box<dyn std::error::Error + Send + Sync>>>
    {
        Box::pin(async {
            let http = serenity::http::Http::new(&self.token);
            let result = crate::fetch::fetch_guild_data(&http).await?;
            Ok(result)
        })
    }
}
