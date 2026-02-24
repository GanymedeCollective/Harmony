use std::error::Error;

use async_trait::async_trait;

use crate::Channel;
use crate::Message;

#[async_trait]
pub trait MessageSender: Send + Sync + 'static {
    async fn send_message(
        &self,
        target: &Channel,
        message: &Message,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
}
