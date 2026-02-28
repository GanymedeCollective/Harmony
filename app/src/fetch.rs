use std::path::Path;

use anyhow::Result;
use bridge_core::PlatformAdapter;

use crate::fetched_data::FetchedData;

pub async fn cmd_fetch(adapters: &[Box<dyn PlatformAdapter>], runtime_dir: &Path) -> Result<()> {
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
