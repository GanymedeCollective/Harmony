//! This will be changed soon, I realize this is not the right approach

use std::collections::HashMap;

use crate::config::ChannelLink;
use crate::fetched_data::FetchedData;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChannelRef {
    pub platform: String,
    pub channel: String,
}

/// Routes messages between channels across platforms
pub struct ChannelRouter {
    routes: HashMap<ChannelRef, Vec<ChannelRef>>,
}

impl ChannelRouter {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    pub fn add_bidirectional(&mut self, p1: &str, c1: &str, p2: &str, c2: &str) {
        let ref1 = ChannelRef {
            platform: p1.to_owned(),
            channel: c1.to_owned(),
        };
        let ref2 = ChannelRef {
            platform: p2.to_owned(),
            channel: c2.to_owned(),
        };

        let targets = self.routes.entry(ref1.clone()).or_default();
        if !targets.contains(&ref2) {
            targets.push(ref2.clone());
        }

        let targets = self.routes.entry(ref2).or_default();
        if !targets.contains(&ref1) {
            targets.push(ref1);
        }
    }

    pub fn targets(&self, platform: &str, channel: &str) -> Vec<ChannelRef> {
        let key = ChannelRef {
            platform: platform.to_owned(),
            channel: channel.to_owned(),
        };
        self.routes.get(&key).cloned().unwrap_or_default()
    }

    pub fn pair_count(&self) -> usize {
        self.routes.len() / 2
    }

    /// Build a ChannelRouter from config-defined channel links
    pub fn from_config(links: &[ChannelLink]) -> Self {
        let mut router = Self::new();
        for link in links {
            let entries: Vec<(&String, &String)> = link.iter().collect();
            for i in 0..entries.len() {
                for j in (i + 1)..entries.len() {
                    let (p1, c1) = entries[i];
                    let (p2, c2) = entries[j];
                    router.add_bidirectional(p1, c1, p2, c2);
                }
            }
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
                        self.add_bidirectional(p1, &ch1.id, p2, &ch2.id);
                    }
                }
            }
        }
    }
}

fn normalize_channel_name(name: &str) -> String {
    name.trim_start_matches('#').to_lowercase()
}
