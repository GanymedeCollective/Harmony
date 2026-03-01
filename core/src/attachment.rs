//! A file attached to a message (not actionnable yet)

use http::Uri;

#[derive(Debug, Clone)]
pub struct Attachment {
    pub url: Uri,
    pub filename: String,
}
