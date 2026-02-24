mod compat;

use anyhow::Result;
use async_trait::async_trait;
use bridge_core::Message;
use serenity::all::{Context, EventHandler, GatewayIntents, Ready};
use serenity::model::channel::Message as SerenityMessage;
use tokio::sync::mpsc;

pub use compat::{DiscordConfig, DiscordSender};

struct Handler {
    tx: mpsc::Sender<Message>,
    bot_user_id: std::sync::Mutex<Option<u64>>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        log::info!("discord: connected as {}", ready.user.name);
        let mut id = self.bot_user_id.lock().unwrap();
        *id = Some(ready.user.id.get());
    }

    async fn message(&self, _ctx: Context, msg: SerenityMessage) {
        let is_self = {
            let id = self.bot_user_id.lock().unwrap();
            id.map_or(false, |bot_id| msg.author.id.get() == bot_id)
        };
        if is_self || msg.author.bot {
            return;
        }

        let core_msg = compat::discord_to_core(&msg);
        if self.tx.send(core_msg).await.is_err() {
            log::warn!("discord: receiver dropped, handler will stop forwarding");
        }
    }
}

pub async fn run(config: DiscordConfig) -> Result<(mpsc::Receiver<Message>, DiscordSender)> {
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let (tx, rx) = mpsc::channel::<Message>(256);

    let handler = Handler {
        tx,
        bot_user_id: std::sync::Mutex::new(config.bot_user_id),
    };

    let mut client = serenity::Client::builder(&config.token, intents)
        .event_handler(handler)
        .await?;

    let http = client.http.clone();
    let sender = DiscordSender { http };

    tokio::spawn(async move {
        if let Err(e) = client.start().await {
            log::error!("discord: client error: {e}");
        }
    });

    Ok((rx, sender))
}
