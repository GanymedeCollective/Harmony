//! Discord platform adapter for the bridge.

mod adapter;
mod convert;
mod fetch;
mod handler;
mod proxy;
mod sender;

pub use adapter::DiscordAdapter;
