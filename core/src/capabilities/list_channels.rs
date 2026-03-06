//! Listing channels known to a platform.

use std::error::Error;

use crate::{BoxFuture, PlatformChannel};

pub trait ListChannels: Send + Sync {
    fn list_channels(
        &self,
    ) -> BoxFuture<'_, Result<Vec<PlatformChannel>, Box<dyn Error + Send + Sync>>>;
}
