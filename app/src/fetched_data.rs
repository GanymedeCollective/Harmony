//! Serializable cache of discovered channels and users per platform

use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use bridge_core::{Channel, User};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FetchedData {
    #[serde(default)]
    pub platforms: HashMap<String, PlatformData>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PlatformData {
    #[serde(default)]
    pub channels: Vec<FetchedChannel>,
    #[serde(default)]
    pub users: Vec<FetchedUser>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchedChannel {
    pub id: String,
    pub name: String,
}

impl From<Channel> for FetchedChannel {
    fn from(ch: Channel) -> Self {
        Self {
            id: ch.id,
            name: ch.name,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchedUser {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
}

impl From<User> for FetchedUser {
    fn from(u: User) -> Self {
        Self {
            id: u.id.unwrap_or_default(),
            name: u.name,
            display_name: u.display_name,
            avatar_url: u.avatar_url,
        }
    }
}

impl FetchedData {
    pub fn load(path: &Path) -> Result<Self> {
        match std::fs::read_to_string(path) {
            Ok(contents) => toml::from_str(&contents)
                .with_context(|| format!("Failed to parse {}", path.display())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                log::info!(
                    "no fetched data found at {}, starting fresh",
                    path.display()
                );
                Ok(Self::default())
            }
            Err(e) => Err(e).with_context(|| format!("Failed to read {}", path.display())),
        }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let contents = toml::to_string_pretty(self).context("Failed to serialize fetched data")?;
        std::fs::write(path, contents)
            .with_context(|| format!("Failed to write {}", path.display()))
    }

    pub fn platform_mut(&mut self, platform: &str) -> &mut PlatformData {
        self.platforms.entry(platform.to_owned()).or_default()
    }

    pub fn upsert_user(
        &mut self,
        platform: &str,
        id: &str,
        name: String,
        display_name: Option<String>,
        avatar_url: Option<String>,
    ) -> bool {
        let data = self.platform_mut(platform);
        if let Some(existing) = data.users.iter_mut().find(|u| u.id == id) {
            let changed = existing.name != name
                || existing.display_name != display_name
                || existing.avatar_url != avatar_url;
            if changed {
                existing.name = name;
                existing.display_name = display_name;
                existing.avatar_url = avatar_url;
            }
            changed
        } else {
            data.users.push(FetchedUser {
                id: id.to_owned(),
                name,
                display_name,
                avatar_url,
            });
            true
        }
    }

    pub fn remove_user(&mut self, platform: &str, id: &str) -> bool {
        let data = self.platform_mut(platform);
        let before = data.users.len();
        data.users.retain(|u| u.id != id);
        data.users.len() != before
    }

    pub fn upsert_channel(&mut self, platform: &str, id: &str, name: String) -> bool {
        let data = self.platform_mut(platform);
        if let Some(existing) = data.channels.iter_mut().find(|c| c.id == id) {
            let changed = existing.name != name;
            if changed {
                existing.name = name;
            }
            changed
        } else {
            data.channels.push(FetchedChannel {
                id: id.to_owned(),
                name,
            });
            true
        }
    }

    pub fn remove_channel(&mut self, platform: &str, id: &str) -> bool {
        let data = self.platform_mut(platform);
        let before = data.channels.len();
        data.channels.retain(|c| c.id != id);
        data.channels.len() != before
    }

    pub fn merge_users(&mut self, platform: &str, users: Vec<User>) -> bool {
        let mut changed = false;
        for u in users {
            let id = u.id.clone().unwrap_or_else(|| u.name.clone());
            if self.upsert_user(platform, &id, u.name, u.display_name, u.avatar_url) {
                changed = true;
            }
        }
        changed
    }

    pub fn rename_user(
        &mut self,
        platform: &str,
        old_id: &str,
        new_id: &str,
        new_name: &str,
    ) -> bool {
        let data = self.platform_mut(platform);
        if let Some(user) = data.users.iter_mut().find(|u| u.id == old_id) {
            user.id = new_id.to_owned();
            user.name = new_name.to_owned();
            true
        } else {
            data.users.push(FetchedUser {
                id: new_id.to_owned(),
                name: new_name.to_owned(),
                display_name: None,
                avatar_url: None,
            });
            true
        }
    }
}
