//! Starts adapters, runs the message-relay and event loops

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use bridge::config::{ChannelLink, UserLink};
use bridge::fetched_data::FetchedData;
use bridge::profile::UserProfile;
use bridge::router::ChannelRouter;
use bridge::{config, events, profile};
use bridge_core::{
    Channel, DEFAULT_CHANNEL_BUFFER, Message, MetaEvent, PlatformAdapter, PlatformHandle,
    PlatformId,
};
use clap::Parser;
use tokio::sync::{RwLock, mpsc};

#[derive(Parser)]
#[command(about = "IRC-Discord bridge")]
struct Args {
    #[arg(short, long, value_name = "PATH")]
    config: Option<PathBuf>,

    #[arg(long)]
    log_path: Option<PathBuf>,

    #[arg(short, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    bridge::logger::init(args.verbose, args.log_path.as_deref());

    let (runtime_dir, config_path) = bridge::resolve_paths(args.config.as_deref());

    log::info!("config: {}", config_path.display());

    let cfg = config::load(&config_path)?;
    let adapters = bridge::create_adapters(&cfg);

    cmd_run(cfg, adapters, &runtime_dir).await
}

async fn cmd_run(
    cfg: config::Config,
    adapters: Vec<Box<dyn PlatformAdapter>>,
    runtime_dir: &Path,
) -> Result<()> {
    let fetched_path = runtime_dir.join("fetched_data.toml");
    let fetched = FetchedData::load(&fetched_path)?;

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
                    profile::enrich::enrich_message(&mut msg, &p);
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

    let mut profs = profile::build::build_from_config(config_users);
    profile::build::auto_correlate(fetched, &mut profs);

    (router, profs)
}
