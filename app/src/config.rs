use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use bridge_utils::BiMap;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub irc: IrcSection,
    pub discord: DiscordSection,
    pub channels: Vec<ChannelPair>,
    #[serde(default)]
    pub users: Vec<UserPair>,
}

#[derive(Deserialize)]
pub struct IrcSection {
    pub server: String,
    pub port: u16,
    #[serde(default = "default_true")]
    pub use_tls: bool,
    #[serde(default)]
    pub accept_invalid_certs: bool,
    pub nickname: String,
    #[serde(default)]
    pub channels: Vec<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Deserialize)]
pub struct DiscordSection {
    pub token: String,
}

#[derive(Deserialize)]
pub struct ChannelPair {
    pub irc: String,
    pub discord: String,
}

#[derive(Deserialize)]
pub struct UserPair {
    pub irc: String,
    pub discord: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub colour: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UserProfile {
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub colour: Option<u32>,
}

fn parse_hex_colour(s: &str) -> Option<u32> {
    let hex = s.strip_prefix('#').unwrap_or(s);
    u32::from_str_radix(hex, 16).ok()
}

impl Config {
    pub fn channel_map(&self) -> BiMap<String, String> {
        let mut map = BiMap::with_capacity(self.channels.len());
        for pair in &self.channels {
            map.insert(pair.irc.clone(), pair.discord.clone());
        }
        map
    }

    pub fn user_map(&self) -> BiMap<String, String> {
        let mut map = BiMap::with_capacity(self.users.len());
        for pair in &self.users {
            map.insert(pair.irc.clone(), pair.discord.clone());
        }
        map
    }

    pub fn user_profiles(&self) -> HashMap<String, UserProfile> {
        let mut profiles = HashMap::with_capacity(self.users.len() * 2);
        for pair in &self.users {
            let profile = UserProfile {
                display_name: pair.display_name.clone(),
                avatar_url: pair.avatar_url.clone(),
                colour: pair.colour.as_deref().and_then(parse_hex_colour),
            };
            profiles.insert(pair.irc.clone(), profile.clone());
            profiles.insert(pair.discord.clone(), profile);
        }
        profiles
    }
}

pub fn load(path: &Path) -> Result<Config> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;
    toml::from_str(&contents).with_context(|| "Failed to parse config file")
}
