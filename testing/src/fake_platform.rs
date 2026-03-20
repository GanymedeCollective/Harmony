//! A controllable platform adapter for integration tests.
//!
//! `FakePlatform` implements `PlatformAdapter` so tests can inject
//! messages/events on one side and assert what comes out the other.

use std::time::Duration;

use exn::Exn;
use harmony_core::{
    BoxFuture, CoreMessage, HarmonyError, ListChannels, ListUsers, MetaEvent, PlatformAdapter,
    PlatformChannel, PlatformHandle, PlatformId, PlatformMessage, PlatformUser, SendMessage,
};
use tokio::sync::{mpsc, oneshot};

/// Controllable adapter for integration tests.
///
/// Use [`FakePlatform::new`] for a quick default or [`FakePlatform::builder`]
/// to pre-configure channels/users returned by listing.
pub struct FakePlatform {
    id: PlatformId,
    inject_msg_rx: mpsc::Receiver<PlatformMessage>,
    inject_event_rx: mpsc::Receiver<MetaEvent>,
    captured_tx: mpsc::UnboundedSender<CoreMessage>,
    channels: Vec<PlatformChannel>,
    users: Vec<PlatformUser>,
}

impl FakePlatform {
    #[must_use]
    #[expect(
        clippy::new_ret_no_self,
        reason = "For testing purposes, it is easier to have new return this"
    )]
    pub fn new(name: &str) -> (Box<dyn PlatformAdapter>, FakeControl) {
        Self::builder(name).build()
    }

    #[must_use]
    pub fn builder(name: &str) -> FakePlatformBuilder {
        FakePlatformBuilder {
            id: PlatformId::new(name),
            channels: Vec::new(),
            users: Vec::new(),
        }
    }
}

impl PlatformAdapter for FakePlatform {
    fn platform_id(&self) -> &PlatformId {
        &self.id
    }

    fn start(
        self: Box<Self>,
        msg_tx: mpsc::Sender<(PlatformId, PlatformMessage)>,
        event_tx: mpsc::Sender<MetaEvent>,
    ) -> BoxFuture<'static, Result<PlatformHandle, Exn<HarmonyError>>> {
        Box::pin(async move {
            let id = self.id.clone();
            let mut inject_msg_rx = self.inject_msg_rx;
            let mut inject_event_rx = self.inject_event_rx;

            let (shutdown_tx, mut shutdown_rx) = oneshot::channel();

            let task_id = id.clone();
            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        Some(msg) = inject_msg_rx.recv() => {
                            if msg_tx.send((task_id.clone(), msg)).await.is_err() {
                                log::warn!("fake {}: receiver dropped, stopping", task_id);
                                break;
                            }
                        }
                        Some(event) = inject_event_rx.recv() => {
                            if event_tx.send(event).await.is_err() {
                                log::warn!("fake {}: receiver dropped, stopping", task_id);
                                break;
                            }
                        }
                        _ = &mut shutdown_rx => break,
                    }
                }
            });

            let sender = FakeSender {
                captured_tx: self.captured_tx,
            };

            let lister = FakeLister {
                channels: self.channels,
                users: self.users,
            };

            Ok(PlatformHandle {
                id,
                sender: Box::new(sender),
                user_lister: Box::new(lister.clone()),
                channel_lister: Box::new(lister),
                shutdown_tx,
            })
        })
    }
}

struct FakeSender {
    captured_tx: mpsc::UnboundedSender<CoreMessage>,
}

impl SendMessage for FakeSender {
    fn send_message<'a>(
        &'a self,
        message: &'a CoreMessage,
    ) -> BoxFuture<'a, Result<(), Exn<HarmonyError>>> {
        let _ = self.captured_tx.send(message.clone());
        Box::pin(async { Ok(()) })
    }
}

#[derive(Clone)]
struct FakeLister {
    channels: Vec<PlatformChannel>,
    users: Vec<PlatformUser>,
}

impl ListUsers for FakeLister {
    fn list_users(&self) -> BoxFuture<'_, Result<Vec<PlatformUser>, Exn<HarmonyError>>> {
        let users = self.users.clone();
        Box::pin(async move { Ok(users) })
    }
}

impl ListChannels for FakeLister {
    fn list_channels(&self) -> BoxFuture<'_, Result<Vec<PlatformChannel>, Exn<HarmonyError>>> {
        let channels = self.channels.clone();
        Box::pin(async move { Ok(channels) })
    }
}

/// Test-side handle for injecting messages/events and reading captured output
pub struct FakeControl {
    platform_id: PlatformId,
    inject_msg_tx: mpsc::Sender<PlatformMessage>,
    inject_event_tx: mpsc::Sender<MetaEvent>,
    captured_rx: tokio::sync::Mutex<mpsc::UnboundedReceiver<CoreMessage>>,
}

impl FakeControl {
    pub const fn platform_id(&self) -> &PlatformId {
        &self.platform_id
    }

    pub async fn inject_message(&self, msg: PlatformMessage) {
        self.inject_msg_tx
            .send(msg)
            .await
            .expect("adapter task gone");
    }

    pub async fn inject_event(&self, event: MetaEvent) {
        self.inject_event_tx
            .send(event)
            .await
            .expect("adapter task gone");
    }

    /// Wait for the next relayed message, returning `None` on timeout.
    pub async fn next_message(&self, timeout: Duration) -> Option<CoreMessage> {
        let mut rx = self.captured_rx.lock().await;
        tokio::time::timeout(timeout, rx.recv())
            .await
            .ok()
            .flatten()
    }
}

/// Builder for [`FakePlatform`] when you need to pre-configure listing data
pub struct FakePlatformBuilder {
    id: PlatformId,
    channels: Vec<PlatformChannel>,
    users: Vec<PlatformUser>,
}

impl FakePlatformBuilder {
    #[must_use]
    pub fn with_channels(mut self, channels: Vec<PlatformChannel>) -> Self {
        self.channels = channels;
        self
    }

    #[must_use]
    pub fn with_users(mut self, users: Vec<PlatformUser>) -> Self {
        self.users = users;
        self
    }

    #[must_use]
    pub fn build(self) -> (Box<dyn PlatformAdapter>, FakeControl) {
        let (inject_msg_tx, inject_msg_rx) = mpsc::channel(64);
        let (inject_event_tx, inject_event_rx) = mpsc::channel(64);
        let (captured_tx, captured_rx) = mpsc::unbounded_channel();

        let platform = FakePlatform {
            id: self.id.clone(),
            inject_msg_rx,
            inject_event_rx,
            captured_tx,
            channels: self.channels,
            users: self.users,
        };

        let control = FakeControl {
            platform_id: self.id,
            inject_msg_tx,
            inject_event_tx,
            captured_rx: tokio::sync::Mutex::new(captured_rx),
        };

        (Box::new(platform), control)
    }
}
