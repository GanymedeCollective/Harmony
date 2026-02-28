use std::collections::HashMap;

use bridge_core::Message;

use crate::user_profile::UserProfile;

pub fn enrich_message(msg: &mut Message, profiles: &HashMap<String, UserProfile>) {
    let key = msg.author.id.as_deref().unwrap_or(&msg.author.name);
    if let Some(profile) = profiles.get(key) {
        if let Some(name) = &profile.display_name {
            msg.author.name = name.clone();
        }
        if let Some(url) = &profile.avatar_url {
            msg.author.avatar_url = Some(url.clone());
        }
    }
}
