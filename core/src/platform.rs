//! Opaque identifier for a chat platform (e.g. "irc", "discord").

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlatformId(String);

impl PlatformId {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

impl fmt::Display for PlatformId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::ops::Deref for PlatformId {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}
