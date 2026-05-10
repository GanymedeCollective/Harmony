//! Channel-based proxy for [`SendMessage`] that decouples the core relay loop
//! from adapter-internal resources.

use std::sync::Arc;

use exn::Exn;
use harmony_core::{BoxFuture, CoreMessage, HarmonyError, SendMessage};
use tokio::sync::{mpsc, oneshot};

pub struct SendRequest {
    pub message: Arc<CoreMessage>,
    pub response_tx: oneshot::Sender<Result<(), Exn<HarmonyError>>>,
}

pub struct DiscordSendProxy {
    pub tx: mpsc::Sender<SendRequest>,
}

impl SendMessage for DiscordSendProxy {
    fn send_message<'a>(
        &'a self,
        message: &'a Arc<CoreMessage>,
    ) -> BoxFuture<'a, Result<(), Exn<HarmonyError>>> {
        Box::pin(async {
            let (response_tx, response_rx) = oneshot::channel();
            self.tx
                .send(SendRequest {
                    message: Arc::clone(message),
                    response_tx,
                })
                .await
                .map_err(|_| {
                    Exn::from(
                        HarmonyError::send("failed to send message to discord adapter").permanent(),
                    )
                })?;
            response_rx.await.map_err(|_| {
                Exn::from(
                    HarmonyError::send("failed to receive response from discord adapter")
                        .temporary(),
                )
            })?
        })
    }
}
