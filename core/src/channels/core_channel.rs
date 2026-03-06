//! Cross-platform channel that aggregates aliases from multiple platforms.

use std::collections::HashMap;

use crate::PlatformId;
use crate::peers::Peered;

use super::PlatformChannel;

#[derive(Debug, Clone)]
pub struct CoreChannel {
    pub alias: HashMap<PlatformId, PlatformChannel>,
    pub name_override: Option<String>,
}

impl Peered for CoreChannel {
    type Platform = PlatformChannel;

    fn aliases(&self) -> &HashMap<PlatformId, PlatformChannel> {
        &self.alias
    }

    fn aliases_mut(&mut self) -> &mut HashMap<PlatformId, PlatformChannel> {
        &mut self.alias
    }

    fn platform_of(item: &PlatformChannel) -> &PlatformId {
        &item.platform
    }

    fn id_of(item: &PlatformChannel) -> &str {
        &item.id
    }

    fn from_single_alias(platform: PlatformId, item: PlatformChannel) -> Self {
        let mut alias = HashMap::new();
        alias.insert(platform, item);
        Self {
            alias,
            name_override: None,
        }
    }

    fn match_key(item: &PlatformChannel) -> Option<String> {
        Some(item.name.trim_start_matches('#').to_lowercase())
    }
}

impl CoreChannel {
    #[must_use]
    pub fn get_platform_channel(&self, platform: &PlatformId) -> Option<&PlatformChannel> {
        self.alias.get(platform)
    }

    /// Best name: override, then first alias name.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name_override
            .as_deref()
            .or_else(|| self.alias.values().next().map(|pc| pc.name.as_str()))
    }
}
