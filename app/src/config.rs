//! Deserializes `config.toml` into typed configuration

use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub irc: IrcSection,
    pub discord: DiscordSection,
    #[serde(default)]
    pub channels: Vec<ChannelLink>,
    #[serde(default)]
    pub users: Vec<UserLink>,
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

impl IrcSection {
    #[must_use]
    pub fn to_irc_config(&self) -> bridge_irc::IrcConfig {
        bridge_irc::IrcConfig {
            nickname: Some(self.nickname.clone()),
            server: Some(self.server.clone()),
            port: Some(self.port),
            use_tls: Some(self.use_tls),
            dangerously_accept_invalid_certs: Some(self.accept_invalid_certs),
            channels: self.channels.clone(),
            ..Default::default()
        }
    }
}

const fn default_true() -> bool {
    true
}

#[derive(Deserialize)]
pub struct DiscordSection {
    pub token: String,
}

/// A channel link maps channel identifiers across platforms.
/// Example: `{ "irc": "#general", "discord": "123456" }`
pub type ChannelLink = HashMap<String, String>;

/// A user link maps a user's identity across platforms
#[derive(Deserialize, Clone)]
pub struct UserLink {
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(flatten)]
    pub identities: HashMap<String, String>,
}

pub fn load(path: &Path) -> Result<Config> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;
    toml::from_str(&contents).with_context(|| "Failed to parse config file")
}
