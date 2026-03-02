//! Bridge lifecycle: start adapters, relay messages, handle events.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use bridge_core::{
    Channel, DEFAULT_CHANNEL_BUFFER, Message, MessageSender, MetaEvent, PlatformAdapter, PlatformId,
};
use bridge_utils::PeerGroups;
use tokio::sync::{RwLock, mpsc, oneshot};
use tokio::task::JoinHandle;

use crate::config::{ChannelLink, UserLink};
use crate::fetched_data::FetchedData;
use crate::profile::{UserMeta, UserRef};
use crate::router::ChannelRouter;
use crate::{events, profile};

pub struct BridgeHandle {
    task: JoinHandle<()>,
    shutdown_txs: Vec<(String, oneshot::Sender<()>)>,
}

impl BridgeHandle {
    pub async fn shutdown(self) {
        for (name, tx) in self.shutdown_txs {
            log::info!("stopping {name}...");
            let _ = tx.send(());
        }
        let _ = self.task.await;
    }
}

/// Start all adapters, build routes/profiles, and spawn the relay loop.
///
/// Returns a [`BridgeHandle`] that can shut everything down
pub async fn run(
    adapters: Vec<Box<dyn PlatformAdapter>>,
    channel_links: Vec<ChannelLink>,
    user_links: Vec<UserLink>,
    fetched: FetchedData,
    persist_path: Option<PathBuf>,
) -> Result<BridgeHandle> {
    let (msg_tx, mut msg_rx) = mpsc::channel::<(PlatformId, Message)>(DEFAULT_CHANNEL_BUFFER);
    let (event_tx, mut event_rx) = mpsc::channel::<MetaEvent>(DEFAULT_CHANNEL_BUFFER);

    let mut senders: HashMap<String, Box<dyn MessageSender>> = HashMap::new();
    let mut shutdown_txs: Vec<(String, oneshot::Sender<()>)> = Vec::new();

    for adapter in adapters {
        let name = adapter.platform_id().to_string();
        let handle = adapter
            .start(msg_tx.clone(), event_tx.clone())
            .await
            .map_err(|e| anyhow::anyhow!("failed to start {name}: {e}"))?;
        log::info!("started platform: {}", handle.id);
        let id_str = handle.id.to_string();
        senders.insert(id_str.clone(), handle.sender);
        shutdown_txs.push((id_str, handle.shutdown_tx));
    }

    drop(msg_tx);
    drop(event_tx);

    let (initial_routes, initial_profiles) = rebuild_all(&fetched, &channel_links, &user_links);

    log::info!(
        "bridge ready: {} channel bridge(s), {} user group(s)",
        initial_routes.bridge_count(),
        initial_profiles.group_count(),
    );

    let routes = Arc::new(RwLock::new(initial_routes));
    let profiles = Arc::new(RwLock::new(initial_profiles));
    let fetched = Arc::new(RwLock::new(fetched));

    let task = tokio::spawn(async move {
        loop {
            tokio::select! {
                Some((source_id, mut msg)) = msg_rx.recv() => {
                    let targets = {
                        let r = routes.read().await;
                        r.targets(&source_id, &msg.channel.id)
                    };
                    {
                        let p = profiles.read().await;
                        profile::enrich::enrich_message(&mut msg, &source_id, &p);
                    }
                    for target in &targets {
                        if let Some(sender) = senders.get(target.platform.as_str()) {
                            let channel = Channel {
                                id: target.channel.clone(),
                                name: target.channel.clone(),
                            };
                            if let Err(e) = sender.send_message(&channel, &msg).await {
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
                            rebuild_all(&f, &channel_links, &user_links);
                        *routes.write().await = new_routes;
                        *profiles.write().await = new_profiles;

                        if let Some(path) = &persist_path {
                            if let Err(e) = f.save(path) {
                                log::error!("failed to save fetched data: {e}");
                            }
                        }
                    }
                }
                else => break,
            }
        }
    });

    Ok(BridgeHandle { task, shutdown_txs })
}

fn rebuild_all(
    fetched: &FetchedData,
    config_channels: &[ChannelLink],
    config_users: &[UserLink],
) -> (ChannelRouter, PeerGroups<UserRef, UserMeta>) {
    let mut router = ChannelRouter::from_config(config_channels);
    router.auto_correlate(fetched);
    router.compact();

    let mut profiles = profile::build::build_from_config(config_users);
    profile::build::auto_correlate(fetched, &mut profiles);
    profiles.compact();

    (router, profiles)
}
