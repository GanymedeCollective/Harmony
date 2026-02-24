mod args;
mod config;
mod logger;

use std::path::PathBuf;

use anyhow::Result;
use bridge_core::{Channel, Message, MessageSender};
use tokio::sync::mpsc;

async fn relay<'a>(
    mut rx: mpsc::Receiver<Message>,
    sender: &impl MessageSender,
    lookup: impl Fn(&str) -> Option<&'a str>,
    direction: &str,
) -> Result<()> {
    while let Some(msg) = rx.recv().await {
        if let Some(target_id) = lookup(&msg.channel.id) {
            let target = Channel {
                id: target_id.to_owned(),
                name: target_id.to_owned(),
            };
            if let Err(e) = sender.send_message(&target, &msg).await {
                log::error!("{direction}: failed to relay message: {e}");
            }
        }
    }
    log::warn!("{direction}: message stream ended");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = args::parse();
    logger::init(args.verbose, args.log_path.as_deref());

    let runtime_dir = std::env::var("BRIDGE_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("runtime"));

    let config_path = runtime_dir.join("config.toml");

    log::info!("runtime directory: {}", runtime_dir.display());

    let cfg = config::load(&config_path)?;
    let channels = cfg.channel_map();

    log::info!("starting bridge with {} channel pair(s)", channels.len());

    let irc_config = bridge_irc::IrcConfig {
        nickname: Some(cfg.irc.nickname.clone()),
        server: Some(cfg.irc.server),
        port: Some(cfg.irc.port),
        use_tls: Some(cfg.irc.use_tls),
        channels: cfg.irc.channels,
        ..Default::default()
    };

    let discord_config = bridge_discord::DiscordConfig {
        token: cfg.discord.token,
        bot_user_id: None,
    };

    let (irc_rx, irc_sender) = bridge_irc::run(irc_config, cfg.irc.nickname).await?;
    let (discord_rx, discord_sender) = bridge_discord::run(discord_config).await?;

    log::info!("bridge is running, ctrl+c to stop");

    tokio::select! {
        r = relay(irc_rx, &discord_sender, |id| channels.get_by_left(id).map(|s| s.as_str()), "IRC->Discord") => r?,
        r = relay(discord_rx, &irc_sender, |id| channels.get_by_right(id).map(|s| s.as_str()), "Discord->IRC") => r?,
        _ = tokio::signal::ctrl_c() => {
            log::info!("shutting down...");
        }
    }

    Ok(())
}
