//! Platform-specific user identity.

use crate::PlatformId;

/// A user's identity on a single platform.
#[derive(Debug, Clone)]
pub struct PlatformUser {
    pub platform: PlatformId,
    pub id: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}
