//! A chat user's identity on a single platform.

#[derive(Debug, Clone)]
pub struct User {
    pub id: Option<String>,
    pub name: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}
