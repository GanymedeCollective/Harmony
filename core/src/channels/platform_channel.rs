//! Platform-specific channel identity.

use crate::PlatformId;

#[derive(Debug, Clone)]
pub struct PlatformChannel {
    pub platform: PlatformId,
    pub id: String,
    pub name: String,
}
