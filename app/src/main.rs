mod args;
mod config;
mod logger;

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use bridge_core::{Channel, Message, MessageSender};
use config::UserProfile;
use tokio::sync::mpsc;

fn enrich_message(msg: &mut Message, profiles: &HashMap<String, UserProfile>) {
    let key = msg.author.id.as_deref().unwrap_or(&msg.author.name);
    if let Some(profile) = profiles.get(key) {
        if let Some(name) = &profile.display_name {
            msg.author.name = name.clone();
        }
        if let Some(url) = &profile.avatar_url {
            if let Ok(uri) = url.parse() {
                msg.author.avatar_url = Some(uri);
            }
        }
        if profile.colour.is_some() {
            msg.author.colour = profile.colour;
        }
    }
}

async fn relay<'a>(
    mut rx: mpsc::Receiver<Message>,
    sender: &impl MessageSender,
    channel_lookup: impl Fn(&str) -> Option<&'a str>,
    profiles: &HashMap<String, UserProfile>,
    direction: &str,
) -> Result<()> {
    while let Some(mut msg) = rx.recv().await {
        if let Some(target_id) = channel_lookup(&msg.channel.id) {
            let target = Channel {
                id: target_id.to_owned(),
                name: target_id.to_owned(),
            };
            enrich_message(&mut msg, profiles);
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

    let config_path = args.config.unwrap_or_else(|| {
        let runtime_dir = std::env::var("BRIDGE_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("runtime"));
        runtime_dir.join("config.toml")
    });

    log::info!("config: {}", config_path.display());

    let cfg = config::load(&config_path)?;
    let channels = cfg.channel_map();
    let user_profiles = cfg.user_profiles();

    log::info!(
        "starting bridge with {} channel pair(s) and {} user mapping(s)",
        channels.len(),
        cfg.users.len(),
    );

    let irc_config = bridge_irc::IrcConfig {
        nickname: Some(cfg.irc.nickname.clone()),
        server: Some(cfg.irc.server),
        port: Some(cfg.irc.port),
        use_tls: Some(cfg.irc.use_tls),
        dangerously_accept_invalid_certs: Some(cfg.irc.accept_invalid_certs),
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
        r = relay(irc_rx, &discord_sender, |id| channels.get_by_left(id).map(|s| s.as_str()), &user_profiles, "IRC->Discord") => r?,
        r = relay(discord_rx, &irc_sender, |id| channels.get_by_right(id).map(|s| s.as_str()), &user_profiles, "Discord->IRC") => r?,
        _ = tokio::signal::ctrl_c() => {
            log::info!("shutting down...");
        }
    }

    Ok(())
}
