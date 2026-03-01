//! Lifecycle events (joins, leaves, renames, channel changes).

use crate::{PlatformId, User};

#[derive(Debug, Clone)]
pub enum MetaEvent {
    UserJoined {
        platform: PlatformId,
        user: User,
    },
    UserLeft {
        platform: PlatformId,
        id: String,
    },
    UserUpdated {
        platform: PlatformId,
        user: User,
    },
    UserRenamed {
        platform: PlatformId,
        old_id: String,
        new_id: String,
        new_name: String,
    },
    UsersDiscovered {
        platform: PlatformId,
        users: Vec<User>,
    },
    ChannelCreated {
        platform: PlatformId,
        id: String,
        name: String,
    },
    ChannelDeleted {
        platform: PlatformId,
        id: String,
    },
    ChannelUpdated {
        platform: PlatformId,
        id: String,
        name: String,
    },
}
