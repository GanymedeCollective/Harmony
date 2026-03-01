//! Builds the user peer-groups from config links and cross-platform name matching.

use std::collections::HashMap;

use bridge_utils::PeerGroups;

use crate::config::UserLink;
use crate::fetched_data::FetchedData;

use super::{UserMeta, UserRef};

/// Build user peer-groups from config-defined user links
pub fn build_from_config(links: &[UserLink]) -> PeerGroups<UserRef, UserMeta> {
    let mut groups = PeerGroups::new();
    for link in links {
        let refs: Vec<UserRef> = link
            .identities
            .iter()
            .map(|(platform, user_id)| UserRef {
                platform: platform.clone(),
                user_id: user_id.clone(),
            })
            .collect();

        groups.link(&refs);

        if let Some(first) = refs.first() {
            let meta = UserMeta {
                display_name: link.display_name.clone(),
                avatar_url: link.avatar_url.clone(),
            };
            groups.set_metadata(first, meta);
        }
    }
    groups
}

/// Auto-correlate users across platforms by matching names/display names
pub fn auto_correlate(fetched: &FetchedData, groups: &mut PeerGroups<UserRef, UserMeta>) {
    let platforms: Vec<&String> = fetched.platforms.keys().collect();
    for i in 0..platforms.len() {
        for j in (i + 1)..platforms.len() {
            let p1 = platforms[i];
            let p2 = platforms[j];
            let data1 = &fetched.platforms[p1];
            let data2 = &fetched.platforms[p2];

            let mut by_name: HashMap<String, &crate::fetched_data::FetchedUser> = HashMap::new();
            for u in &data2.users {
                by_name.insert(u.name.to_lowercase(), u);
                if let Some(dn) = &u.display_name {
                    by_name.entry(dn.to_lowercase()).or_insert(u);
                }
            }

            for u1 in &data1.users {
                if let Some(u2) = by_name.get(&u1.name.to_lowercase()) {
                    let ref1 = UserRef {
                        platform: p1.clone(),
                        user_id: u1.id.clone(),
                    };
                    let ref2 = UserRef {
                        platform: p2.clone(),
                        user_id: u2.id.clone(),
                    };

                    groups.link(&[ref1.clone(), ref2]);

                    if groups.metadata(&ref1).is_none() {
                        let display = u2.display_name.clone().unwrap_or_else(|| u2.name.clone());
                        groups.set_metadata(
                            &ref1,
                            UserMeta {
                                display_name: Some(display),
                                avatar_url: u2.avatar_url.clone(),
                            },
                        );
                    }
                }
            }
        }
    }
}
