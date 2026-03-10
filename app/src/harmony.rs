//! CLI entry point: parse args, load config, wire adapters, delegate to core.

mod args;
mod config;
mod logger;

use std::path::PathBuf;

use anyhow::Result;
use args::Args;
use harmony_core::PlatformAdapter;
use clap::Parser;

#[must_use]
fn create_adapters(cfg: &config::Config) -> Vec<Box<dyn PlatformAdapter>> {
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

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    logger::init(args.verbose, args.log_path.as_deref());

    let runtime_dir = PathBuf::from(
        std::env::var("HARMONY_RUNTIME_DIR").unwrap_or_else(|_| "runtime".to_string()),
    );
    let config_path = args
        .config
        .unwrap_or_else(|| runtime_dir.join("config.toml"));

    log::info!("config: {}", config_path.display());

    let cfg = config::load(&config_path)?;
    let adapters = create_adapters(&cfg);

    let handle = harmony_core::run::run(adapters).await?;

    log::info!("Harmony is running, ctrl+c to stop");
    tokio::signal::ctrl_c().await?;

    log::info!("shutting down... (press ctrl+c again to force)");
    tokio::select! {
        () = handle.shutdown() => {}
        _ = tokio::signal::ctrl_c() => {
            log::warn!("forced shutdown");
            std::process::exit(1);
        }
    }

    Ok(())
}
