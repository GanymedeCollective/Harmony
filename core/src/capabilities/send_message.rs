//! Sending a message to a target channel.

use std::sync::Arc;

use exn::Exn;

use crate::{BoxFuture, CoreMessage, HarmonyError};

pub trait SendMessage: Send + Sync + 'static {
    fn send_message<'a>(
        &'a self,
        message: &'a Arc<CoreMessage>,
    ) -> BoxFuture<'a, Result<(), Exn<HarmonyError>>>;
}
