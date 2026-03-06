//! IRC platform adapter for the bridge.

mod adapter;
mod convert;
mod lister;
mod sender;

pub use adapter::{IrcAdapter, IrcConfig};
pub use sender::IrcSender;
