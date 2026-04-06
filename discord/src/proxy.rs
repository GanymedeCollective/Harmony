//! Channel-based proxy for [`SendMessage`] that decouples the core relay loop
//! from adapter-internal resources.

use {
    exn::Exn,
    harmony_core::{BoxFuture, CoreMessage, HarmonyError, SendMessage},
    tokio::sync::{mpsc, oneshot},
};

pub(crate) struct SendRequest {
    pub message: CoreMessage,
    pub response_tx: oneshot::Sender<Result<(), Exn<HarmonyError>>>,
}

pub(crate) struct DiscordSendProxy {
    pub tx: mpsc::Sender<SendRequest>,
}

impl SendMessage for DiscordSendProxy {
    fn send_message<'a>(
        &'a self,
        message: &'a CoreMessage,
    ) -> BoxFuture<'a, Result<(), Exn<HarmonyError>>> {
        Box::pin(async {
            let (response_tx, response_rx) = oneshot::channel();
            self.tx
                .send(SendRequest {
                    message: message.clone(),
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
                    HarmonyError::send("failed to received response from discord adapter")
                        .permanent(),
                )
            })?
        })
    }
}
