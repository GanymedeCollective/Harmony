use http::Uri;

#[derive(Debug, Clone)]
pub struct User {
    pub name: String,
    pub avatar_url: Option<Uri>,
}
