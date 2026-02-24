use std::path::Path;

use anyhow::{Context, Result};
use bridge_utils::BiMap;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub irc: IrcSection,
    pub discord: DiscordSection,
    pub channels: Vec<ChannelPair>,
}

#[derive(Deserialize)]
pub struct IrcSection {
    pub server: String,
    pub port: u16,
    #[serde(default = "default_true")]
    pub use_tls: bool,
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

impl Config {
    pub fn channel_map(&self) -> BiMap<String, String> {
        let mut map = BiMap::with_capacity(self.channels.len());
        for pair in &self.channels {
            map.insert(pair.irc.clone(), pair.discord.clone());
        }
        map
    }
}

pub fn load(path: &Path) -> Result<Config> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;
    toml::from_str(&contents).with_context(|| "Failed to parse config file")
}
