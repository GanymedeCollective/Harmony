//! Sending a message to a target channel.

use {exn::Exn, std::sync::Arc};

use crate::{BoxFuture, CoreMessage, HarmonyError};

pub trait SendMessage: Send + Sync + 'static {
    fn send_message<'a>(
        &'a self,
        message: &'a CoreMessage,
    ) -> BoxFuture<'a, Result<(), Exn<HarmonyError>>>;
}

impl<T: SendMessage> SendMessage for Arc<T> {
    fn send_message<'a>(
        &'a self,
        message: &'a CoreMessage,
    ) -> BoxFuture<'a, Result<(), Exn<HarmonyError>>> {
        (**self).send_message(message)
    }
}
