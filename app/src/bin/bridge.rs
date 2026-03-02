//! CLI entry point: parse args, load config, delegate to `run::run`.

use std::path::PathBuf;

use anyhow::Result;
use bridge::{config, run};
use clap::Parser;

#[derive(Parser)]
#[command(about = "IRC-Discord bridge")]
struct Args {
    #[arg(short, long, value_name = "PATH")]
    config: Option<PathBuf>,

    #[arg(long)]
    log_path: Option<PathBuf>,

    #[arg(short, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    bridge::logger::init(args.verbose, args.log_path.as_deref());

    let (runtime_dir, config_path) = bridge::resolve_paths(args.config.as_deref());

    log::info!("config: {}", config_path.display());

    let cfg = config::load(&config_path)?;
    let adapters = bridge::create_adapters(&cfg);

    let fetched_path = runtime_dir.join("fetched_data.toml");
    let fetched = bridge::fetched_data::FetchedData::load(&fetched_path)?;

    let handle = run::run(
        adapters,
        cfg.channels,
        cfg.users,
        fetched,
        Some(fetched_path),
    )
    .await?;

    log::info!("bridge is running, ctrl+c to stop");
    tokio::signal::ctrl_c().await?;

    log::info!("shutting down... (press ctrl+c again to force)");
    tokio::select! {
        _ = handle.shutdown() => {}
        _ = tokio::signal::ctrl_c() => {
            log::warn!("forced shutdown");
            std::process::exit(1);
        }
    }

    Ok(())
}
