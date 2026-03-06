//! Sending a message to a target channel.

use std::error::Error;

use crate::BoxFuture;
use crate::CoreMessage;

pub trait SendMessage: Send + Sync + 'static {
    fn send_message<'a>(
        &'a self,
        message: &'a CoreMessage,
    ) -> BoxFuture<'a, Result<(), Box<dyn Error + Send + Sync>>>;
}
