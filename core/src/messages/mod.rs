//! Message types for platform-to-core and core-to-platform communication.

mod core_message;
mod platform_message;

pub use core_message::CoreMessage;
pub use core_message::CoreMessageRope;
pub use core_message::CoreMessageSegment;
pub use platform_message::PlatformMessage;
pub use platform_message::PlatformMessageRope;
pub use platform_message::PlatformMessageSegment;
