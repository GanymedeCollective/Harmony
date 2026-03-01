use std::path::PathBuf;

use anyhow::Result;
use bridge::config;
use bridge::fetched_data::FetchedData;
use bridge_core::PlatformAdapter;
use clap::Parser;

#[derive(Parser)]
#[command(about = "Fetch channels/users from all platforms")]
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

    cmd_fetch(&adapters, &runtime_dir).await
}

async fn cmd_fetch(
    adapters: &[Box<dyn PlatformAdapter>],
    runtime_dir: &std::path::Path,
) -> Result<()> {
    log::info!("fetching channel and user data from all platforms...");

    let mut data = FetchedData::default();

    for adapter in adapters {
        let platform = adapter.platform_id().to_string();
        log::info!("fetching from {platform}...");

        match adapter.fetch().await {
            Ok((channels, users)) => {
                println!(
                    "{platform}: {} channel(s), {} user(s)",
                    channels.len(),
                    users.len()
                );
                let pd = data.platform_mut(&platform);
                pd.channels = channels.into_iter().map(Into::into).collect();
                pd.users = users.into_iter().map(Into::into).collect();
            }
            Err(e) => {
                log::error!("failed to fetch from {platform}: {e}");
            }
        }
    }

    let out_path = runtime_dir.join("fetched_data.toml");
    data.save(&out_path)?;
    println!("Saved to {}", out_path.display());

    Ok(())
}
