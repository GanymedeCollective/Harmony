//! Message types for platform-to-core and core-to-platform communication.

mod core_message;
mod platform_message;

pub use core_message::CoreMessage;
pub use platform_message::PlatformMessage;
