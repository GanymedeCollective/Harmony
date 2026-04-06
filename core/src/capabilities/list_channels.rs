//! Listing channels known to a platform.
use std::sync::Arc;

use exn::Exn;

use crate::{BoxFuture, HarmonyError, PlatformChannel};

pub trait ListChannels: Send + Sync {
    fn list_channels(&self) -> BoxFuture<'_, Result<Vec<PlatformChannel>, Exn<HarmonyError>>>;
}

impl<T: ListChannels> ListChannels for Arc<T> {
    fn list_channels(&self) -> BoxFuture<'_, Result<Vec<PlatformChannel>, Exn<HarmonyError>>> {
        (**self).list_channels()
    }
}
