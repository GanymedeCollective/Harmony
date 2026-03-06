use crate::PlatformChannel;
use crate::PlatformUser;

/// A message as produced by a platform adapter (inbound to core).
#[derive(Debug, Clone)]
pub struct PlatformMessage {
    pub author: PlatformUser,
    pub channel: PlatformChannel,
    pub content: String,
}
