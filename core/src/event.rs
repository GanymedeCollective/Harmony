//! Lifecycle events (joins, leaves, renames, channel changes).

use crate::{PlatformChannel, PlatformId, PlatformUser};

#[derive(Debug, Clone)]
pub enum MetaEvent {
    UserJoined {
        platform: PlatformId,
        user: PlatformUser,
    },
    UserLeft {
        platform: PlatformId,
        id: String,
    },
    UserUpdated {
        platform: PlatformId,
        user: PlatformUser,
    },
    UserRenamed {
        platform: PlatformId,
        old_id: String,
        new_id: String,
        new_display_name: Option<String>,
    },
    UsersDiscovered {
        platform: PlatformId,
        users: Vec<PlatformUser>,
    },
    ChannelCreated {
        platform: PlatformId,
        channel: PlatformChannel,
    },
    ChannelDeleted {
        platform: PlatformId,
        id: String,
    },
    ChannelUpdated {
        platform: PlatformId,
        channel: PlatformChannel,
    },
}
