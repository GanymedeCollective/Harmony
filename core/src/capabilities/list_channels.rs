//! Listing channels known to a platform.

use exn::Exn;

use crate::{BoxFuture, HarmonyError, PlatformChannel};

pub trait ListChannels: Send + Sync {
    fn list_channels(&self) -> BoxFuture<'_, Result<Vec<PlatformChannel>, Exn<HarmonyError>>>;
}
