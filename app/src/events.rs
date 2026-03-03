//! Applies `MetaEvent`s to the fetched-data store

use bridge_core::MetaEvent;

use crate::fetched_data::FetchedData;

/// Process a `MetaEvent` by updating the fetched data store
/// Returns whether the fetched data was modified
pub fn handle_meta_event(fetched: &mut FetchedData, event: &MetaEvent) -> bool {
    match event {
        MetaEvent::UserLeft { platform, id } => fetched.remove_user(platform, id),
        MetaEvent::UserJoined { platform, user } | MetaEvent::UserUpdated { platform, user } => {
            let id = user.id.as_deref().unwrap_or(&user.name);
            fetched.upsert_user(
                platform,
                id,
                user.name.clone(),
                user.display_name.clone(),
                user.avatar_url.clone(),
            )
        }
        MetaEvent::UserRenamed {
            platform,
            old_id,
            new_id,
            new_name,
        } => fetched.rename_user(platform, old_id, new_id, new_name),
        MetaEvent::UsersDiscovered { platform, users } => {
            fetched.merge_users(platform, users.clone())
        }
        MetaEvent::ChannelCreated { platform, id, name }
        | MetaEvent::ChannelUpdated { platform, id, name } => {
            fetched.upsert_channel(platform, id, name.clone())
        }
        MetaEvent::ChannelDeleted { platform, id } => fetched.remove_channel(platform, id),
    }
}
