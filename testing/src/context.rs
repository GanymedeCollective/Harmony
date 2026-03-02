//! Per-test runtime: instantiates a test world into live fake platforms and a bridge.

use std::collections::HashMap;

use bridge::config::UserLink;
use bridge::fetched_data::FetchedData;
use bridge::run::BridgeHandle;
use bridge_core::{Channel, User};

use crate::fake_platform::FakePlatform;
use crate::world::{TestWorld, UserSpec};

pub use crate::fake_platform::FakeControl;

pub struct TestContext {
    controls: HashMap<String, FakeControl>,
    users: HashMap<String, UserSpec>,
    handle: BridgeHandle,
}

impl TestWorld {
    /// Instantiate this world into live fake platforms, start the bridge, and
    /// return a [`TestContext`] for scripting scenarios.
    pub async fn start(&self) -> TestContext {
        let mut adapters = Vec::new();
        let mut controls = HashMap::new();

        for spec in &self.platforms {
            let channels: Vec<Channel> = spec
                .channels
                .iter()
                .map(|ch| Channel {
                    id: ch.clone(),
                    name: ch.clone(),
                })
                .collect();

            let users: Vec<User> = self
                .users
                .iter()
                .filter_map(|u| {
                    u.identities.get(&spec.name).map(|name| User {
                        id: Some(name.clone()),
                        name: name.clone(),
                        display_name: u.display_name.clone(),
                        avatar_url: u.avatar_url.clone(),
                    })
                })
                .collect();

            let (adapter, control) = FakePlatform::builder(&spec.name)
                .with_channels(channels)
                .with_users(users)
                .build();

            adapters.push(adapter);
            controls.insert(spec.name.clone(), control);
        }

        let channel_links = self
            .channel_links
            .iter()
            .map(|pairs| pairs.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .collect();

        let user_links: Vec<UserLink> = self
            .users
            .iter()
            .map(|u| UserLink {
                display_name: u.display_name.clone(),
                avatar_url: u.avatar_url.clone(),
                identities: u.identities.clone(),
            })
            .collect();

        let handle = bridge::run::run(
            adapters,
            channel_links,
            user_links,
            FetchedData::default(),
            None,
        )
        .await
        .expect("bridge should start");

        let users = self
            .users
            .iter()
            .map(|u| (u.canonical_name.clone(), u.clone()))
            .collect();

        TestContext {
            controls,
            users,
            handle,
        }
    }
}

impl TestContext {
    pub fn control(&self, platform: &str) -> &FakeControl {
        self.controls
            .get(platform)
            .unwrap_or_else(|| panic!("no platform '{platform}' in test world"))
    }

    /// Look up a user's platform-specific identity. Panics if the user has no
    /// identity on that platform.
    pub fn user_name(&self, canonical: &str, platform: &str) -> &str {
        self.users
            .get(canonical)
            .and_then(|spec| spec.identities.get(platform))
            .map(|s| s.as_str())
            .unwrap_or_else(|| {
                panic!("user '{canonical}' has no identity on platform '{platform}'")
            })
    }

    /// Resolve an author name for a platform: if it matches a known canonical
    /// user name, return the platform-specific identity; otherwise return the
    /// name as-is.
    pub fn resolve_author(&self, canonical_or_raw: &str, platform: &str) -> String {
        self.users
            .get(canonical_or_raw)
            .and_then(|spec| spec.identities.get(platform))
            .cloned()
            .unwrap_or_else(|| canonical_or_raw.to_owned())
    }

    pub async fn shutdown(self) {
        self.handle.shutdown().await;
    }
}
