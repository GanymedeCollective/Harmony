//! Queries Discord guilds for text channels and members.

use anyhow::Result;
use bridge_core::{PlatformChannel, PlatformId, PlatformUser};
use serenity::all::ChannelType;

pub async fn fetch_guild_data(
    http: &serenity::http::Http,
    platform_id: &PlatformId,
) -> Result<(Vec<PlatformChannel>, Vec<PlatformUser>)> {
    let guilds = http.get_guilds(None, Some(100)).await?;
    log::info!("discord: found {} guild(s)", guilds.len());

    let mut channels = Vec::new();
    let mut users = Vec::new();
    let mut seen_users = std::collections::HashSet::new();

    for guild_info in &guilds {
        let guild_id = guild_info.id;
        log::debug!(
            "discord: fetching data for guild \"{}\" ({})",
            guild_info.name,
            guild_id
        );

        let guild_channels = http.get_channels(guild_id).await?;
        for ch in guild_channels {
            if ch.kind == ChannelType::Text {
                channels.push(PlatformChannel {
                    platform: platform_id.clone(),
                    id: ch.id.get().to_string(),
                    name: ch.name.clone(),
                });
            }
        }

        let mut after = None;
        loop {
            let members = http.get_guild_members(guild_id, Some(1000), after).await?;
            if members.is_empty() {
                break;
            }
            let last_id = members.last().map(|m| m.user.id.get());
            for member in &members {
                if member.user.bot || !seen_users.insert(member.user.id) {
                    continue;
                }
                users.push(PlatformUser {
                    platform: platform_id.clone(),
                    id: member.user.id.get().to_string(),
                    display_name: member
                        .nick
                        .clone()
                        .or_else(|| member.user.global_name.clone())
                        .or_else(|| Some(member.user.name.clone())),
                    avatar_url: member.user.avatar_url(),
                });
            }
            if members.len() < 1000 {
                break;
            }
            after = last_id;
        }
    }

    log::info!(
        "discord: fetched {} channel(s) and {} user(s)",
        channels.len(),
        users.len()
    );

    Ok((channels, users))
}
