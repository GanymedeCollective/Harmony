//! Test world definition: platforms, users, and channel links.

use std::collections::HashMap;

pub struct TestWorld {
    pub(crate) platforms: Vec<PlatformSpec>,
    pub(crate) users: Vec<UserSpec>,
    pub(crate) channel_links: Vec<Vec<(String, String)>>,
}

pub struct PlatformSpec {
    pub(crate) name: String,
    pub(crate) channels: Vec<String>,
}

#[derive(Clone)]
pub struct UserSpec {
    pub(crate) canonical_name: String,
    pub(crate) identities: HashMap<String, String>,
    pub(crate) display_name: Option<String>,
    pub(crate) avatar_url: Option<String>,
}

pub struct TestWorldBuilder {
    platforms: Vec<PlatformSpec>,
    users: Vec<UserSpec>,
    channel_links: Vec<Vec<(String, String)>>,
}

impl TestWorld {
    pub fn builder() -> TestWorldBuilder {
        TestWorldBuilder {
            platforms: Vec::new(),
            users: Vec::new(),
            channel_links: Vec::new(),
        }
    }
}

impl TestWorldBuilder {
    pub fn platform(mut self, name: &str, channels: &[&str]) -> Self {
        self.platforms.push(PlatformSpec {
            name: name.to_owned(),
            channels: channels.iter().map(|s| s.to_string()).collect(),
        });
        self
    }

    pub fn user(mut self, canonical: &str, identities: &[(&str, &str)]) -> Self {
        self.users.push(UserSpec {
            canonical_name: canonical.to_owned(),
            identities: identities
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            display_name: None,
            avatar_url: None,
        });
        self
    }

    pub fn user_with_meta(
        mut self,
        canonical: &str,
        identities: &[(&str, &str)],
        display_name: Option<&str>,
        avatar_url: Option<&str>,
    ) -> Self {
        self.users.push(UserSpec {
            canonical_name: canonical.to_owned(),
            identities: identities
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            display_name: display_name.map(|s| s.to_owned()),
            avatar_url: avatar_url.map(|s| s.to_owned()),
        });
        self
    }

    pub fn link(mut self, pairs: &[(&str, &str)]) -> Self {
        self.channel_links.push(
            pairs
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        );
        self
    }

    pub fn build(self) -> TestWorld {
        TestWorld {
            platforms: self.platforms,
            users: self.users,
            channel_links: self.channel_links,
        }
    }
}
