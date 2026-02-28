mod args;
mod config;
mod enrich;
mod events;
mod fetch;
mod fetched_data;
mod logger;
mod profiles;
mod router;
mod user_profile;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use bridge_core::{
    Channel, DEFAULT_CHANNEL_BUFFER, Message, MetaEvent, PlatformAdapter, PlatformHandle,
    PlatformId,
};
use clap::Parser;
use tokio::sync::{RwLock, mpsc};

use crate::config::{ChannelLink, UserLink};
use crate::fetched_data::FetchedData;
use crate::router::ChannelRouter;
use crate::user_profile::UserProfile;

fn resolve_paths(config_arg: Option<&Path>) -> (PathBuf, PathBuf) {
    if let Some(config_path) = config_arg {
        let runtime_dir = config_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        (runtime_dir, config_path.to_path_buf())
    } else {
        let runtime_dir = std::env::var("BRIDGE_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("runtime"));
        let config_path = runtime_dir.join("config.toml");
        (runtime_dir, config_path)
    }
}

/// The only place that knows about specific platform crates
fn create_adapters(cfg: &config::Config) -> Vec<Box<dyn PlatformAdapter>> {
    vec![
        Box::new(bridge_irc::IrcAdapter::new(
            cfg.irc.to_irc_config(),
            cfg.irc.nickname.clone(),
        )),
        Box::new(bridge_discord::DiscordAdapter::new(
            cfg.discord.token.clone(),
        )),
    ]
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = args::Args::parse();
    logger::init(args.verbose, args.log_path.as_deref());

    let (runtime_dir, config_path) = resolve_paths(args.config.as_deref());

    log::info!("config: {}", config_path.display());

    let cfg = config::load(&config_path)?;

    let adapters = create_adapters(&cfg);

    match args.command {
        Some(args::Command::Fetch) => fetch::cmd_fetch(&adapters, &runtime_dir).await,
        None => cmd_run(cfg, adapters, &runtime_dir).await,
    }
}

async fn cmd_run(
    cfg: config::Config,
    adapters: Vec<Box<dyn PlatformAdapter>>,
    runtime_dir: &Path,
) -> Result<()> {
    let fetched_path = runtime_dir.join("fetched_data.toml");
    let fetched = fetched_data::FetchedData::load(&fetched_path)?;

    let (msg_tx, mut msg_rx) = mpsc::channel::<(PlatformId, Message)>(DEFAULT_CHANNEL_BUFFER);
    let (event_tx, mut event_rx) = mpsc::channel::<MetaEvent>(DEFAULT_CHANNEL_BUFFER);

    let mut handles: HashMap<String, PlatformHandle> = HashMap::new();

    for adapter in adapters {
        let name = adapter.platform_id().to_string();
        let handle = adapter
            .start(msg_tx.clone(), event_tx.clone())
            .await
            .map_err(|e| anyhow::anyhow!("failed to start {name}: {e}"))?;
        log::info!("started platform: {}", handle.id);
        handles.insert(handle.id.to_string(), handle);
    }

    // Drop our copies so channels close when all platforms stop
    drop(msg_tx);
    drop(event_tx);

    let (initial_routes, initial_profiles) = rebuild_all(&fetched, &cfg.channels, &cfg.users);

    log::info!(
        "bridge ready: {} channel route(s), {} user profile(s)",
        initial_routes.pair_count(),
        initial_profiles.len(),
    );

    let routes = Arc::new(RwLock::new(initial_routes));
    let profiles = Arc::new(RwLock::new(initial_profiles));
    let fetched = Arc::new(RwLock::new(fetched));

    log::info!("bridge is running, ctrl+c to stop");

    loop {
        tokio::select! {
            Some((source_id, mut msg)) = msg_rx.recv() => {
                let targets = {
                    let r = routes.read().await;
                    r.targets(&source_id, &msg.channel.id)
                };
                {
                    let p = profiles.read().await;
                    enrich::enrich_message(&mut msg, &p);
                }
                for target in &targets {
                    if let Some(handle) = handles.get(target.platform.as_str()) {
                        let channel = Channel {
                            id: target.channel.clone(),
                            name: target.channel.clone(),
                        };
                        if let Err(e) = handle.sender.send_message(&channel, &msg).await {
                            log::error!(
                                "{source_id} -> {}: relay failed: {e}",
                                target.platform
                            );
                        }
                    }
                }
            }
            Some(event) = event_rx.recv() => {
                let mut f = fetched.write().await;
                if events::handle_meta_event(&mut f, &event) {
                    let (new_routes, new_profiles) =
                        rebuild_all(&f, &cfg.channels, &cfg.users);
                    *routes.write().await = new_routes;
                    *profiles.write().await = new_profiles;

                    if let Err(e) = f.save(&fetched_path) {
                        log::error!("failed to save fetched data: {e}");
                    }
                }
            }
            _ = tokio::signal::ctrl_c() => {
                log::info!("shutting down...");
                for (name, handle) in handles {
                    log::info!("stopping {name}...");
                    let _ = handle.shutdown_tx.send(());
                }
                break;
            }
        }
    }

    Ok(())
}

fn rebuild_all(
    fetched: &FetchedData,
    config_channels: &[ChannelLink],
    config_users: &[UserLink],
) -> (ChannelRouter, HashMap<String, UserProfile>) {
    let mut router = ChannelRouter::from_config(config_channels);
    router.auto_correlate(fetched);

    let mut profs = profiles::build_from_config(config_users);
    profiles::auto_correlate(fetched, &mut profs);

    (router, profs)
}
