use std::error::Error;

use crate::BoxFuture;
use crate::Channel;
use crate::Message;

pub trait MessageSender: Send + Sync + 'static {
    fn send_message<'a>(
        &'a self,
        target: &'a Channel,
        message: &'a Message,
    ) -> BoxFuture<'a, Result<(), Box<dyn Error + Send + Sync>>>;
}
