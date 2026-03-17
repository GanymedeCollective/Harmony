//! Sends bridged messages as IRC PRIVMSG.

use exn::{Exn, OptionExt as _, ResultExt as _};
use harmony_core::{BoxFuture, CoreMessage, HarmonyError, PlatformId, SendMessage};
use irc::client as irc_client;

#[derive(Clone)]
pub struct IrcSender {
    pub(crate) inner: irc_client::Sender,
    pub(crate) platform_id: PlatformId,
}

impl SendMessage for IrcSender {
    fn send_message<'a>(
        &'a self,
        message: &'a CoreMessage,
    ) -> BoxFuture<'a, Result<(), Exn<HarmonyError>>> {
        Box::pin(async {
            let display_name = message
                .author
                .get_platform_user(&self.platform_id)
                .and_then(|pu| pu.display_name.as_deref())
                .or_else(|| message.author.display_name())
                .unwrap_or("unknown");

            let channel = message
                .channel
                .get_platform_channel(&self.platform_id)
                .ok_or_raise(|| {
                    HarmonyError::send("no channel alias for this platform").permanent()
                })?;

            let text = format!("<{display_name}> {}", message.content);
            self.inner
                .send_privmsg(&channel.id, text)
                .or_raise(|| HarmonyError::send("irc PRIVMSG failed"))?;
            Ok(())
        })
    }
}
