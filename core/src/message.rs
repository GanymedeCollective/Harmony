//! A chat message with metadata

use crate::Attachment;
use crate::Channel;
use crate::User;

#[derive(Debug, Clone)]
pub struct Message {
    pub author: User,
    pub channel: Channel,
    pub content: String,
    pub attachments: Vec<Attachment>,
}
