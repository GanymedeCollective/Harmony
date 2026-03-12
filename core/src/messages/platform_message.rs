use crate::PlatformChannel;
use crate::PlatformUser;

/// A message as produced by a platform adapter (inbound to core).
#[derive(Debug, Clone)]
pub struct PlatformMessage {
    pub author: PlatformUser,
    pub channel: PlatformChannel,
    pub content: PlatformMessageRope,
}

/// A segment of a message rope.
#[derive(Debug, Clone)]
pub enum PlatformMessageSegment {
    Text(String),
    Mention(PlatformUser),
}

/// A rope of message segments.
pub type PlatformMessageRope = Vec<PlatformMessageSegment>;
