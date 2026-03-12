//! Platform-agnostic types and traits to use as a base for adapters.

mod adapter;
pub mod capabilities;
mod channels;
mod event;
mod messages;
pub(crate) mod peers;
mod platform;
pub mod run;
mod users;

pub use adapter::{PlatformAdapter, PlatformHandle};
pub use capabilities::{ListChannels, ListUsers, SendMessage};
pub use channels::{Channels, CoreChannel, PlatformChannel};
pub use event::MetaEvent;
pub use futures::future::BoxFuture;
pub use messages::{CoreMessage, PlatformMessage};
pub use peers::{Peered, Peers};
pub use platform::PlatformId;
pub use users::{CoreUser, PlatformUser, Users};

// TODO: better
pub const DEFAULT_CHANNEL_BUFFER: usize = 256;
