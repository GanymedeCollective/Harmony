//! Platform-agnostic types and traits for the bridge

mod adapter;
mod attachment;
mod channel;
mod event;
mod message;
mod message_sender;
mod platform;
mod user;

pub use adapter::{PlatformAdapter, PlatformHandle};
pub use attachment::Attachment;
pub use channel::Channel;
pub use event::MetaEvent;
pub use message::Message;
pub use message_sender::MessageSender;
pub use platform::PlatformId;
pub use user::User;

pub type BoxFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;

pub const DEFAULT_CHANNEL_BUFFER: usize = 256;
