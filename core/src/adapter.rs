//! Lifecycle contract for platform crates

use exn::Exn;
use tokio::sync::{mpsc, oneshot};

use crate::messages::PlatformMessage;
use crate::{BoxFuture, HarmonyError, ListChannels, ListUsers, MetaEvent, PlatformId, SendMessage};

pub struct PlatformHandle {
    pub id: PlatformId,
    // TODO: Maybe have the capabilities boxed into one thing
    //       Once we have many of them it will become clumbersome
    pub sender: Box<dyn SendMessage>,
    pub user_lister: Box<dyn ListUsers>,
    pub channel_lister: Box<dyn ListChannels>,
    pub shutdown_tx: oneshot::Sender<()>,
}

pub trait PlatformAdapter: Send {
    fn platform_id(&self) -> &PlatformId;

    fn start(
        self: Box<Self>,
        msg_tx: mpsc::Sender<(PlatformId, PlatformMessage)>,
        event_tx: mpsc::Sender<MetaEvent>,
    ) -> BoxFuture<'static, Result<PlatformHandle, Exn<HarmonyError>>>;
}
