//! Cached channel/user lists discovered during IRC startup.

use harmony_core::{BoxFuture, ListChannels, ListUsers, PlatformChannel, PlatformUser};

#[derive(Clone)]
pub struct IrcLister {
    pub(crate) channels: Vec<PlatformChannel>,
    pub(crate) users: Vec<PlatformUser>,
}

impl ListUsers for IrcLister {
    fn list_users(
        &self,
    ) -> BoxFuture<'_, Result<Vec<PlatformUser>, Box<dyn std::error::Error + Send + Sync>>> {
        Box::pin(async { Ok(self.users.clone()) })
    }
}

impl ListChannels for IrcLister {
    fn list_channels(
        &self,
    ) -> BoxFuture<'_, Result<Vec<PlatformChannel>, Box<dyn std::error::Error + Send + Sync>>> {
        Box::pin(async { Ok(self.channels.clone()) })
    }
}
