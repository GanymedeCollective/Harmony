use crate::CoreChannel;
use crate::CoreUser;

/// A message as sent from core to a platform adapter (outbound).
#[derive(Debug, Clone)]
pub struct CoreMessage {
    pub author: CoreUser,
    pub channel: CoreChannel,
    pub content: CoreMessageRope,
}

/// A segment of a message rope.
#[derive(Debug, Clone)]
pub enum CoreMessageSegment {
    Text(String),
    Mention(CoreUser),
}

/// A rope of message segments.
pub type CoreMessageRope = Vec<CoreMessageSegment>;
