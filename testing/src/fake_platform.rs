//! A controllable platform adapter for integration tests.
//!
//! `FakePlatform` implements `PlatformAdapter` and `MessageSender` so tests can
//! inject messages/events on one side and assert what comes out the other.

use std::error::Error;
use std::time::Duration;

use bridge_core::{
    BoxFuture, Channel, Message, MessageSender, MetaEvent, PlatformAdapter, PlatformHandle,
    PlatformId, User,
};
use tokio::sync::{mpsc, oneshot};

/// Controllable adapter for integration tests.
///
/// Use [`FakePlatform::new`] for a quick default or [`FakePlatform::builder`]
/// to pre-configure channels/users returned by `fetch()`.
pub struct FakePlatform {
    id: PlatformId,
    inject_msg_rx: mpsc::Receiver<Message>,
    inject_event_rx: mpsc::Receiver<MetaEvent>,
    captured_tx: mpsc::UnboundedSender<(Channel, Message)>,
    channels: Vec<Channel>,
    users: Vec<User>,
}

impl FakePlatform {
    pub fn new(name: &str) -> (Box<dyn PlatformAdapter>, FakeControl) {
        Self::builder(name).build()
    }

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
        msg_tx: mpsc::Sender<(PlatformId, Message)>,
        event_tx: mpsc::Sender<MetaEvent>,
    ) -> BoxFuture<'static, Result<PlatformHandle, Box<dyn Error + Send + Sync>>> {
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
                            let _ = msg_tx.send((task_id.clone(), msg)).await;
                        }
                        Some(event) = inject_event_rx.recv() => {
                            let _ = event_tx.send(event).await;
                        }
                        _ = &mut shutdown_rx => break,
                    }
                }
            });

            let sender = FakeSender {
                captured_tx: self.captured_tx,
            };

            Ok(PlatformHandle {
                id,
                sender: Box::new(sender),
                shutdown_tx,
            })
        })
    }

    fn fetch(
        &self,
    ) -> BoxFuture<'_, Result<(Vec<Channel>, Vec<User>), Box<dyn Error + Send + Sync>>> {
        let channels = self.channels.clone();
        let users = self.users.clone();
        Box::pin(async move { Ok((channels, users)) })
    }
}

struct FakeSender {
    captured_tx: mpsc::UnboundedSender<(Channel, Message)>,
}

impl MessageSender for FakeSender {
    fn send_message<'a>(
        &'a self,
        target: &'a Channel,
        message: &'a Message,
    ) -> BoxFuture<'a, Result<(), Box<dyn Error + Send + Sync>>> {
        let _ = self.captured_tx.send((target.clone(), message.clone()));
        Box::pin(async { Ok(()) })
    }
}

/// Test-side handle for injecting messages/events and reading captured output
pub struct FakeControl {
    platform_id: PlatformId,
    inject_msg_tx: mpsc::Sender<Message>,
    inject_event_tx: mpsc::Sender<MetaEvent>,
    captured_rx: tokio::sync::Mutex<mpsc::UnboundedReceiver<(Channel, Message)>>,
}

impl FakeControl {
    pub fn platform_id(&self) -> &PlatformId {
        &self.platform_id
    }

    pub async fn inject_message(&self, msg: Message) {
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
    pub async fn next_message(&self, timeout: Duration) -> Option<(Channel, Message)> {
        let mut rx = self.captured_rx.lock().await;
        tokio::time::timeout(timeout, rx.recv())
            .await
            .ok()
            .flatten()
    }
}

/// Builder for [`FakePlatform`] when you need to pre-configure `fetch()` data
pub struct FakePlatformBuilder {
    id: PlatformId,
    channels: Vec<Channel>,
    users: Vec<User>,
}

impl FakePlatformBuilder {
    pub fn with_channels(mut self, channels: Vec<Channel>) -> Self {
        self.channels = channels;
        self
    }

    pub fn with_users(mut self, users: Vec<User>) -> Self {
        self.users = users;
        self
    }

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
