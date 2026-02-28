use bridge_core::{BoxFuture, Channel, Message, MessageSender};
use irc::client as irc_client;

#[derive(Clone)]
pub struct IrcSender {
    pub(crate) inner: irc_client::Sender,
}

impl MessageSender for IrcSender {
    fn send_message<'a>(
        &'a self,
        target: &'a Channel,
        message: &'a Message,
    ) -> BoxFuture<'a, Result<(), Box<dyn std::error::Error + Send + Sync>>> {
        Box::pin(async {
            let text = format!("<{}> {}", message.author.name, message.content);
            self.inner.send_privmsg(&target.id, text)?;
            Ok(())
        })
    }
}
