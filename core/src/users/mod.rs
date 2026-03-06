//! User identity types: platform-specific, cross-platform, and indexed collection.

mod core_user;
mod platform_user;

pub use core_user::CoreUser;
pub use platform_user::PlatformUser;

use crate::PlatformId;
use crate::peers::{Peered, Peers};

pub type Users = Peers<CoreUser>;

impl Users {
    /// Rename a user: update the primary index, the match-key index, and
    /// the alias entry.
    pub fn rename(
        &mut self,
        platform: &PlatformId,
        old_id: &str,
        new_id: &str,
        new_display_name: Option<String>,
    ) {
        let old_key = (platform.clone(), old_id.to_owned());
        let new_key = (platform.clone(), new_id.to_owned());

        let Some(idx) = self.find_index(platform, old_id) else {
            return;
        };

        self.reindex(&old_key, new_key);

        if let Some(core_user) = self.item_mut(idx) {
            let old_match = core_user
                .alias
                .get(platform)
                .and_then(CoreUser::match_key);

            let old_avatar = core_user
                .alias
                .remove(platform)
                .and_then(|pu| pu.avatar_url);

            let new_alias = PlatformUser {
                platform: platform.clone(),
                id: new_id.to_owned(),
                display_name: new_display_name,
                avatar_url: old_avatar,
            };
            let new_match = CoreUser::match_key(&new_alias);
            core_user.alias.insert(platform.clone(), new_alias);

            self.update_match_key(old_match.as_deref(), new_match, idx);
        }
    }
}
