use http::Uri;

#[derive(Debug, Clone)]
pub struct User {
    pub id: Option<String>,
    pub name: String,
    pub avatar_url: Option<Uri>,
    pub colour: Option<u32>,
}
