//! Lifecycle: start adapters, discover data, relay messages, handle events.

use std::collections::HashMap;
use std::sync::Arc;

use exn::{Exn, ResultExt as _};
use tokio::sync::{RwLock, mpsc, oneshot};
use tokio::task::JoinHandle;

use crate::error::HarmonyError;
use crate::{
    Channels, CoreChannel, CoreMessage, CoreUser, DEFAULT_CHANNEL_BUFFER, ListChannels, ListUsers,
    MetaEvent, Peers, PlatformAdapter, PlatformId, PlatformMessage, PlatformUser, SendMessage,
    Users,
};

/// A simple context used for dependency injection across core.
pub(crate) struct CoreCtx {
    pub(crate) channels: Arc<RwLock<Peers<CoreChannel>>>,
    pub(crate) users: Arc<RwLock<Peers<CoreUser>>>,
    pub(crate) senders: HashMap<PlatformId, Box<dyn SendMessage>>,
}

impl CoreCtx {
    pub(crate) fn new(
        channels: Arc<RwLock<Peers<CoreChannel>>>,
        users: Arc<RwLock<Peers<CoreUser>>>,
        senders: HashMap<PlatformId, Box<dyn SendMessage>>,
    ) -> Self {
        Self {
            channels,
            users,
            senders,
        }
    }
}

pub struct AdapterHandle {
    task: JoinHandle<()>,
    shutdown_txs: Vec<(PlatformId, oneshot::Sender<()>)>,
}

impl AdapterHandle {
    pub async fn shutdown(self) {
        for (name, tx) in self.shutdown_txs {
            log::info!("stopping {name}...");
            if tx.send(()).is_err() {
                log::error!("Failed to send shutdown signal for {name}");
            }
        }
        if let Err(e) = self.task.await {
            log::error!("JoinHandle failed with: {e}");
        }
    }
}

/// Start all adapters, discover users/channels, and spawn the relay loop.
///
/// Returns a [`AdapterHandle`] that can shut everything down.
///
/// # Errors
///
/// Returns an error if any adapter fails to start.
pub async fn run(
    adapters: Vec<Box<dyn PlatformAdapter>>,
) -> Result<AdapterHandle, Exn<HarmonyError>> {
    let (msg_tx, mut msg_rx) =
        mpsc::channel::<(PlatformId, PlatformMessage)>(DEFAULT_CHANNEL_BUFFER);
    let (event_tx, mut event_rx) = mpsc::channel::<MetaEvent>(DEFAULT_CHANNEL_BUFFER);

    let mut senders: HashMap<PlatformId, Box<dyn SendMessage>> = HashMap::new();
    let mut user_listers: HashMap<PlatformId, Box<dyn ListUsers>> = HashMap::new();
    let mut channel_listers: HashMap<PlatformId, Box<dyn ListChannels>> = HashMap::new();
    let mut shutdown_txs: Vec<(PlatformId, oneshot::Sender<()>)> = Vec::new();

    let start_futures: Vec<_> = adapters
        .into_iter()
        .map(|adapter| {
            let name = adapter.platform_id().to_string();
            let tx = msg_tx.clone();
            let etx = event_tx.clone();
            async move {
                let handle = adapter.start(tx, etx).await.or_raise(|| {
                    HarmonyError::connection(format!("failed to start platform {name}"))
                })?;
                log::info!("started platform: {}", handle.id);
                Ok::<_, Exn<HarmonyError>>(handle)
            }
        })
        .collect();

    let results = futures::future::join_all(start_futures).await;
    for result in results {
        let handle = result?;
        senders.insert(handle.id.clone(), handle.sender);
        user_listers.insert(handle.id.clone(), handle.user_lister);
        channel_listers.insert(handle.id.clone(), handle.channel_lister);
        shutdown_txs.push((handle.id, handle.shutdown_tx));
    }

    drop(msg_tx);
    drop(event_tx);

    let (channels, users) = discover_and_build(&channel_listers, &user_listers).await;

    log::info!(
        "Harmony ready: {} channel bridge(s), {} user group(s)",
        channels.len(),
        users.len(),
    );

    let channels = Arc::new(RwLock::new(channels));
    let users = Arc::new(RwLock::new(users));
    let ctx = Arc::new(CoreCtx::new(channels, users, senders));

    let task = tokio::spawn(async move {
        loop {
            tokio::select! {
                Some((source_id, msg)) = msg_rx.recv() => {
                    dispatch(ctx.clone(), &source_id, msg).await;
                }
                Some(event) = event_rx.recv() => {
                    handle_event(ctx.clone(), &event).await;
                }
                else => break,
            }
        }
    });

    Ok(AdapterHandle { task, shutdown_txs })
}

/// Dispatches a platform message to all the registered platforms.
async fn dispatch(ctx: Arc<CoreCtx>, source_id: &PlatformId, msg: PlatformMessage) {
    let core_channel = {
        let ch = ctx.channels.read().await;
        ch.find(source_id, &msg.channel.id).cloned()
    };

    let Some(core_channel) = core_channel else {
        log::debug!("{source_id}: no route for channel {}", msg.channel.id);
        return;
    };

    let core_author = {
        let mut u = ctx.users.write().await;
        resolve_or_register(&mut u, source_id, &msg.author)
    };

    let core_msg = CoreMessage {
        author: core_author,
        channel: core_channel.clone(),
        content: msg.content.clone(),
    };

    for platform in core_channel.alias.keys() {
        if platform == source_id {
            continue;
        }
        if let Some(sender) = ctx.senders.get(platform)
            && let Err(e) = sender.send_message(&core_msg).await
        {
            if e.is_temporary() {
                log::warn!("{source_id} -> {platform}: relay failed (retryable): {e:?}");
            } else {
                log::error!("{source_id} -> {platform}: relay failed (permanent): {e:?}");
            }
        }
    }
}

/// Query all adapters for their channels/users, then build the collections.
async fn discover_and_build(
    channel_listers: &HashMap<PlatformId, Box<dyn ListChannels>>,
    user_listers: &HashMap<PlatformId, Box<dyn ListUsers>>,
) -> (Channels, Users) {
    let mut discovered_channels = Vec::new();
    for (pid, lister) in channel_listers {
        match lister.list_channels().await {
            Ok(chs) => {
                log::info!("{pid}: discovered {} channel(s)", chs.len());
                discovered_channels.push((pid.clone(), chs));
            }
            Err(e) => log::error!("{pid}: failed to list channels: {e:?}"),
        }
    }

    let mut discovered_users = Vec::new();
    for (pid, lister) in user_listers {
        match lister.list_users().await {
            Ok(us) => {
                log::info!("{pid}: discovered {} user(s)", us.len());
                discovered_users.push((pid.clone(), us));
            }
            Err(e) => log::error!("{pid}: failed to list users: {e:?}"),
        }
    }

    let channels = Channels::build(&discovered_channels);
    let users = Users::build(&discovered_users);

    (channels, users)
}

/// Look up the `CoreUser` for a message author. If unknown, register them
/// via `upsert` (which also handles auto-correlation) so they become a
/// first-class entity.
fn resolve_or_register(
    users: &mut Users,
    source_platform: &PlatformId,
    author: &PlatformUser,
) -> CoreUser {
    if let Some(core_user) = users.find(source_platform, &author.id) {
        return core_user.clone();
    }
    users.upsert(author.clone());
    users
        .find(source_platform, &author.id)
        .expect("just upserted")
        .clone()
}

/// Handle a runtime event by directly updating the in-memory collections.
async fn handle_event(ctx: Arc<CoreCtx>, event: &MetaEvent) {
    let users = ctx.users.clone();
    let channels = ctx.channels.clone();
    match event {
        MetaEvent::UserJoined { user, .. } | MetaEvent::UserUpdated { user, .. } => {
            users.write().await.upsert(user.clone());
        }
        MetaEvent::UserLeft { platform, id } => {
            users.write().await.detach(platform, id);
        }
        MetaEvent::UserRenamed {
            platform,
            old_id,
            new_id,
            new_display_name,
        } => {
            users
                .write()
                .await
                .rename(platform, old_id, new_id, new_display_name.clone());
        }
        MetaEvent::UsersDiscovered {
            users: new_users, ..
        } => {
            let mut u = users.write().await;
            for user in new_users {
                u.upsert(user.clone());
            }
        }
        MetaEvent::ChannelCreated { channel, .. } | MetaEvent::ChannelUpdated { channel, .. } => {
            channels.write().await.upsert(channel.clone());
        }
        MetaEvent::ChannelDeleted { platform, id } => {
            channels.write().await.detach(platform, id);
        }
    }
}
