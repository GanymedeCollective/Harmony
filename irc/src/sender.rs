//! Sends bridged messages as IRC PRIVMSG.

use std::fmt::Write as _;

use exn::{Exn, OptionExt as _};
use harmony_core::{
    BoxFuture, CoreMessage, CoreMessageSegment, HarmonyError, PlatformId, SendMessage,
};
use irc::client as irc_client;

#[derive(Clone)]
pub struct IrcSender {
    pub(crate) inner: irc_client::Sender,
    pub(crate) platform_id: PlatformId,
}

fn format_message_from_core(
    platform_id: &PlatformId,
    display_name: &str,
    message: &CoreMessage,
) -> String {
    let result = message
        .content
        .iter()
        .fold(String::new(), |mut result, segment| {
            match segment {
                CoreMessageSegment::Text(text) => {
                    result.push_str(text);
                }
                CoreMessageSegment::Mention(core_user) => {
                    if let Some(pu) = core_user.get_platform_user(platform_id) {
                        let _ = write!(result, "@{}", pu.id);
                    } else {
                        let name = core_user.display_name().unwrap_or("unknown");
                        let _ = write!(result, "@{name}");
                    }
                }
                _ => panic!("Unimplemented CoreMessageSegment variant encountered"),
            }
            result
        });

    format!("<{display_name}> {result}")
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

            let text = format_message_from_core(&self.platform_id, display_name, message);
            self.inner
                .send_privmsg(&channel.id, text)
                .map_err(|e| HarmonyError::send(e.to_string()).temporary())?;
            Ok(())
        })
    }
}
