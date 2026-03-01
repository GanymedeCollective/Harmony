//! Stamps outgoing messages with resolved user metadata (display name, avatar).

use bridge_core::Message;
use bridge_utils::PeerGroups;

use super::{UserMeta, UserRef};

pub fn enrich_message(msg: &mut Message, platform: &str, profiles: &PeerGroups<UserRef, UserMeta>) {
    let user_ref = UserRef {
        platform: platform.to_owned(),
        user_id: msg
            .author
            .id
            .clone()
            .unwrap_or_else(|| msg.author.name.clone()),
    };
    if let Some(meta) = profiles.metadata(&user_ref) {
        if let Some(name) = &meta.display_name {
            msg.author.name = name.clone();
        }
        if let Some(url) = &meta.avatar_url {
            msg.author.avatar_url = Some(url.clone());
        }
    }
}
