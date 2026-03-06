//! Deserializes `config.toml` into typed configuration

use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub irc: IrcConfig,
    pub discord: DiscordConfig,
}

#[derive(Deserialize)]
pub struct IrcConfig {
    pub server: String,
    pub port: u16,
    #[serde(default = "default_true")]
    pub use_tls: bool,
    #[serde(default)]
    pub accept_invalid_certs: bool,
    #[serde(default)]
    pub nickname: String,
}

impl IrcConfig {
    #[must_use]
    pub fn to_irc_config(&self) -> bridge_irc::IrcConfig {
        bridge_irc::IrcConfig {
            nickname: Some(self.nickname.clone()),
            server: Some(self.server.clone()),
            port: Some(self.port),
            use_tls: Some(self.use_tls),
            dangerously_accept_invalid_certs: Some(self.accept_invalid_certs),
            ..Default::default()
        }
    }
}

const fn default_true() -> bool {
    true
}

#[derive(Deserialize)]
pub struct DiscordConfig {
    pub token: String,
}

pub fn load(path: &Path) -> Result<Config> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;
    toml::from_str(&contents).with_context(|| "Failed to parse config file")
}
