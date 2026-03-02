//! Composition root: path resolution and platform adapter wiring

pub mod config;
pub mod events;
pub mod fetched_data;
pub mod logger;
pub mod profile;
pub mod router;
pub mod run;

use std::path::{Path, PathBuf};

use bridge_core::PlatformAdapter;

pub fn resolve_paths(config_arg: Option<&Path>) -> (PathBuf, PathBuf) {
    if let Some(config_path) = config_arg {
        let runtime_dir = config_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        (runtime_dir, config_path.to_path_buf())
    } else {
        let runtime_dir = std::env::var("BRIDGE_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("runtime"));
        let config_path = runtime_dir.join("config.toml");
        (runtime_dir, config_path)
    }
}

/// The only place that knows about specific platform crates.
pub fn create_adapters(cfg: &config::Config) -> Vec<Box<dyn PlatformAdapter>> {
    vec![
        Box::new(bridge_irc::IrcAdapter::new(
            cfg.irc.to_irc_config(),
            cfg.irc.nickname.clone(),
        )),
        Box::new(bridge_discord::DiscordAdapter::new(
            cfg.discord.token.clone(),
        )),
    ]
}
