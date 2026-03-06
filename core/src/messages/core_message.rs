use crate::CoreChannel;
use crate::CoreUser;

/// A message as sent from core to a platform adapter (outbound).
#[derive(Debug, Clone)]
pub struct CoreMessage {
    pub author: CoreUser,
    pub channel: CoreChannel,
    pub content: String,
}
