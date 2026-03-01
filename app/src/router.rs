//! Channel routing: groups of bridged channels across platforms.

use std::collections::HashMap;

use bridge_utils::PeerGroups;

use crate::config::ChannelLink;
use crate::fetched_data::FetchedData;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChannelRef {
    pub platform: String,
    pub channel: String,
}

/// Routes messages between bridged channel groups
pub struct ChannelRouter {
    groups: PeerGroups<ChannelRef>,
}

impl Default for ChannelRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl ChannelRouter {
    pub fn new() -> Self {
        Self {
            groups: PeerGroups::new(),
        }
    }

    pub fn link(&mut self, refs: &[ChannelRef]) {
        self.groups.link(refs);
    }

    pub fn targets(&self, platform: &str, channel: &str) -> Vec<ChannelRef> {
        let key = ChannelRef {
            platform: platform.to_owned(),
            channel: channel.to_owned(),
        };
        self.groups
            .peers(&key)
            .map(|peers| peers.into_iter().cloned().collect())
            .unwrap_or_default()
    }

    pub fn bridge_count(&self) -> usize {
        self.groups.group_count()
    }

    pub fn compact(&mut self) {
        self.groups.compact();
    }

    /// Build a ChannelRouter from config-defined channel links
    pub fn from_config(links: &[ChannelLink]) -> Self {
        let mut router = Self::new();
        for link in links {
            let refs: Vec<ChannelRef> = link
                .iter()
                .map(|(platform, channel)| ChannelRef {
                    platform: platform.clone(),
                    channel: channel.clone(),
                })
                .collect();
            router.link(&refs);
        }
        router
    }

    /// Auto-correlate channels across platforms by matching normalized names
    pub fn auto_correlate(&mut self, fetched: &FetchedData) {
        let platforms: Vec<&String> = fetched.platforms.keys().collect();
        for i in 0..platforms.len() {
            for j in (i + 1)..platforms.len() {
                let p1 = platforms[i];
                let p2 = platforms[j];
                let data1 = &fetched.platforms[p1];
                let data2 = &fetched.platforms[p2];

                let by_name: HashMap<String, &crate::fetched_data::FetchedChannel> = data2
                    .channels
                    .iter()
                    .map(|ch| (normalize_channel_name(&ch.name), ch))
                    .collect();

                for ch1 in &data1.channels {
                    let norm = normalize_channel_name(&ch1.name);
                    if let Some(ch2) = by_name.get(&norm) {
                        log::debug!(
                            "auto-correlated channel {}/{}  <->  {}/{}",
                            p1,
                            ch1.name,
                            p2,
                            ch2.name
                        );
                        self.link(&[
                            ChannelRef {
                                platform: p1.clone(),
                                channel: ch1.id.clone(),
                            },
                            ChannelRef {
                                platform: p2.clone(),
                                channel: ch2.id.clone(),
                            },
                        ]);
                    }
                }
            }
        }
    }
}

fn normalize_channel_name(name: &str) -> String {
    name.trim_start_matches('#').to_lowercase()
}
