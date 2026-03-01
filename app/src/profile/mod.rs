//! `UserProfile` — cross-platform display identity for a user.

pub mod build;
pub mod enrich;

#[derive(Debug, Clone)]
pub struct UserProfile {
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}
