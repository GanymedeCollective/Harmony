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
#[non_exhaustive]
pub enum PlatformMessageSegment {
    Text(String),
    //        ID
    Mention(String),
}

/// A rope of message segments.
pub type PlatformMessageRope = Vec<PlatformMessageSegment>;
