//! Sending a message to a target channel.

use exn::Exn;

use crate::BoxFuture;
use crate::CoreMessage;
use crate::HarmonyError;

pub trait SendMessage: Send + Sync + 'static {
    fn send_message<'a>(
        &'a self,
        message: &'a CoreMessage,
    ) -> BoxFuture<'a, Result<(), Exn<HarmonyError>>>;
}
