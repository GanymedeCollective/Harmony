use http::Uri;

#[derive(Debug, Clone)]
pub struct Attachment {
    pub url: Uri,
    pub filename: String,
}
