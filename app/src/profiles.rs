use std::collections::HashMap;

use crate::config::UserLink;
use crate::fetched_data::FetchedData;
use crate::user_profile::UserProfile;

/// Build user profiles from config-defined user links
pub fn build_from_config(links: &[UserLink]) -> HashMap<String, UserProfile> {
    let mut profiles = HashMap::new();
    for link in links {
        let profile = UserProfile {
            display_name: link.display_name.clone(),
            avatar_url: link.avatar_url.clone(),
        };
        for id in link.identities.values() {
            profiles.insert(id.clone(), profile.clone());
        }
    }
    profiles
}

/// Auto-correlate users across platforms by matching names/display names
pub fn auto_correlate(fetched: &FetchedData, profiles: &mut HashMap<String, UserProfile>) {
    let platforms: Vec<&String> = fetched.platforms.keys().collect();
    for i in 0..platforms.len() {
        for j in (i + 1)..platforms.len() {
            let data1 = &fetched.platforms[platforms[i]];
            let data2 = &fetched.platforms[platforms[j]];

            let mut by_name: HashMap<String, &crate::fetched_data::FetchedUser> = HashMap::new();
            for u in &data2.users {
                by_name.insert(u.name.to_lowercase(), u);
                if let Some(dn) = &u.display_name {
                    by_name.entry(dn.to_lowercase()).or_insert(u);
                }
            }

            for u1 in &data1.users {
                if let Some(u2) = by_name.get(&u1.name.to_lowercase()) {
                    let display = u2.display_name.clone().unwrap_or_else(|| u2.name.clone());
                    let profile = UserProfile {
                        display_name: Some(display),
                        avatar_url: u2.avatar_url.clone(),
                    };
                    profiles.entry(u1.id.clone()).or_insert(profile.clone());
                    profiles.entry(u2.id.clone()).or_insert(profile);
                }
            }
        }
    }
}
