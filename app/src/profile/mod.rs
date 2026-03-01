//! Cross-platform user identity and display metadata.

pub mod build;
pub mod enrich;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserRef {
    pub platform: String,
    pub user_id: String,
}

#[derive(Debug, Clone)]
pub struct UserMeta {
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}
