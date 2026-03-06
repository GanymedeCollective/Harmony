//! Cross-platform user that aggregates aliases from multiple platforms.

use super::PlatformUser;
use crate::PlatformId;
use crate::peers::Peered;
use std::collections::HashMap;

/// A cross-platform user identity, grouping aliases across platforms.
#[derive(Debug, Clone)]
pub struct CoreUser {
    pub alias: HashMap<PlatformId, PlatformUser>,
    pub display_name_override: Option<String>,
    pub avatar_override: Option<String>,
}

impl Peered for CoreUser {
    type Platform = PlatformUser;

    fn aliases(&self) -> &HashMap<PlatformId, PlatformUser> {
        &self.alias
    }

    fn aliases_mut(&mut self) -> &mut HashMap<PlatformId, PlatformUser> {
        &mut self.alias
    }

    fn platform_of(item: &PlatformUser) -> &PlatformId {
        &item.platform
    }

    fn id_of(item: &PlatformUser) -> &str {
        &item.id
    }

    fn from_single_alias(platform: PlatformId, item: PlatformUser) -> Self {
        let mut alias = HashMap::new();
        alias.insert(platform, item);
        Self {
            alias,
            display_name_override: None,
            avatar_override: None,
        }
    }

    fn match_key(item: &PlatformUser) -> Option<String> {
        item.display_name.as_ref().map(|dn| dn.to_lowercase())
    }
}

impl CoreUser {
    #[must_use]
    pub fn get_platform_user(&self, platform: &PlatformId) -> Option<&PlatformUser> {
        self.alias.get(platform)
    }

    /// Best display name: override, then first alias that has one.
    #[must_use]
    pub fn display_name(&self) -> Option<&str> {
        self.display_name_override.as_deref().or_else(|| {
            self.alias
                .values()
                .find_map(|pu| pu.display_name.as_deref())
        })
    }

    /// Best avatar URL: override, then first alias that has one.
    #[must_use]
    pub fn avatar_url(&self) -> Option<&str> {
        self.avatar_override
            .as_deref()
            .or_else(|| self.alias.values().find_map(|pu| pu.avatar_url.as_deref()))
    }
}
