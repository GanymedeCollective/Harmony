//! Lifecycle contract for platform crates

use std::error::Error;

use tokio::sync::{mpsc, oneshot};

use crate::{BoxFuture, Channel, Message, MessageSender, MetaEvent, PlatformId, User};

pub struct PlatformHandle {
    pub id: PlatformId,
    pub sender: Box<dyn MessageSender>,
    pub shutdown_tx: oneshot::Sender<()>,
}

pub trait PlatformAdapter: Send {
    fn platform_id(&self) -> &PlatformId;

    fn start(
        self: Box<Self>,
        msg_tx: mpsc::Sender<(PlatformId, Message)>,
        event_tx: mpsc::Sender<MetaEvent>,
    ) -> BoxFuture<'static, Result<PlatformHandle, Box<dyn Error + Send + Sync>>>;

    #[allow(clippy::type_complexity)]
    fn fetch(
        &self,
    ) -> BoxFuture<'_, Result<(Vec<Channel>, Vec<User>), Box<dyn Error + Send + Sync>>>;
}
