//! Sends bridged messages as IRC PRIVMSG.

use bridge_core::{BoxFuture, CoreMessage, CoreMessageSegment, PlatformId, SendMessage};
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
    let mut result = String::new();

    for segment in &message.content {
        match segment {
            CoreMessageSegment::Text(text) => {
                result.push_str(text);
            }
            CoreMessageSegment::Mention(core_user) => {
                let platform_user = core_user.get_platform_user(platform_id).unwrap();
                result.push_str(&format!("@{}", platform_user.id));
            }
        }
    }

    format!("<{display_name}> {}", result)
}

impl SendMessage for IrcSender {
    fn send_message<'a>(
        &'a self,
        message: &'a CoreMessage,
    ) -> BoxFuture<'a, Result<(), Box<dyn std::error::Error + Send + Sync>>> {
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
                .ok_or("no channel alias for this platform")?;

            let text = format_message_from_core(&self.platform_id, display_name, message);
            self.inner.send_privmsg(&channel.id, text)?;
            Ok(())
        })
    }
}
