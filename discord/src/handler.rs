//! Serenity event handler: forwards Discord gateway events as core messages and meta-events.

use async_trait::async_trait;
use bridge_core::{Message, MetaEvent, PlatformId, User};
use serenity::all::{
    ChannelType, Context, EventHandler, GuildChannel, GuildId, GuildMemberUpdateEvent, Member,
    Ready, User as SerenityUser,
};
use serenity::model::channel::Message as SerenityMessage;
use tokio::sync::mpsc;

pub(crate) struct Handler {
    pub msg_tx: mpsc::Sender<(PlatformId, Message)>,
    pub event_tx: mpsc::Sender<MetaEvent>,
    pub platform_id: PlatformId,
    pub bot_user_id: std::sync::OnceLock<u64>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        log::info!("discord: connected as {}", ready.user.name);
        let _ = self.bot_user_id.set(ready.user.id.get());
    }

    async fn message(&self, _ctx: Context, msg: SerenityMessage) {
        let is_self = self
            .bot_user_id
            .get()
            .is_some_and(|&bot_id| msg.author.id.get() == bot_id);
        if is_self || msg.author.bot {
            return;
        }

        let core_msg = crate::convert::discord_to_core(&msg);
        if self
            .msg_tx
            .send((self.platform_id.clone(), core_msg))
            .await
            .is_err()
        {
            log::warn!("discord: receiver dropped, handler will stop forwarding");
        }
    }

    async fn guild_member_addition(&self, _ctx: Context, new_member: Member) {
        if new_member.user.bot {
            return;
        }
        let _ = self
            .event_tx
            .send(MetaEvent::UserJoined {
                platform: self.platform_id.clone(),
                user: User {
                    id: Some(new_member.user.id.get().to_string()),
                    name: new_member.user.name.clone(),
                    display_name: new_member.nick.clone(),
                    avatar_url: new_member.user.avatar_url(),
                },
            })
            .await;
    }

    async fn guild_member_removal(
        &self,
        _ctx: Context,
        _guild_id: GuildId,
        user: SerenityUser,
        _member: Option<Member>,
    ) {
        if user.bot {
            return;
        }
        let _ = self
            .event_tx
            .send(MetaEvent::UserLeft {
                platform: self.platform_id.clone(),
                id: user.id.get().to_string(),
            })
            .await;
    }

    async fn guild_member_update(
        &self,
        _ctx: Context,
        _old: Option<Member>,
        _new: Option<Member>,
        event: GuildMemberUpdateEvent,
    ) {
        if event.user.bot {
            return;
        }
        let _ = self
            .event_tx
            .send(MetaEvent::UserUpdated {
                platform: self.platform_id.clone(),
                user: User {
                    id: Some(event.user.id.get().to_string()),
                    name: event.user.name.clone(),
                    display_name: event.nick.clone(),
                    avatar_url: event.user.avatar_url(),
                },
            })
            .await;
    }

    async fn channel_create(&self, _ctx: Context, channel: GuildChannel) {
        if channel.kind != ChannelType::Text {
            return;
        }
        let _ = self
            .event_tx
            .send(MetaEvent::ChannelCreated {
                platform: self.platform_id.clone(),
                id: channel.id.get().to_string(),
                name: channel.name.clone(),
            })
            .await;
    }

    async fn channel_delete(
        &self,
        _ctx: Context,
        channel: GuildChannel,
        _messages: Option<Vec<SerenityMessage>>,
    ) {
        if channel.kind != ChannelType::Text {
            return;
        }
        let _ = self
            .event_tx
            .send(MetaEvent::ChannelDeleted {
                platform: self.platform_id.clone(),
                id: channel.id.get().to_string(),
            })
            .await;
    }

    async fn channel_update(&self, _ctx: Context, _old: Option<GuildChannel>, new: GuildChannel) {
        if new.kind != ChannelType::Text {
            return;
        }
        let _ = self
            .event_tx
            .send(MetaEvent::ChannelUpdated {
                platform: self.platform_id.clone(),
                id: new.id.get().to_string(),
                name: new.name.clone(),
            })
            .await;
    }
}
