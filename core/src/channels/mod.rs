//! Channel identity types: platform-specific, cross-platform, and indexed
//! collection.

mod core_channel;
mod platform_channel;

pub use core_channel::CoreChannel;
pub use platform_channel::PlatformChannel;

use crate::peers::Peers;

pub type Channels = Peers<CoreChannel>;
