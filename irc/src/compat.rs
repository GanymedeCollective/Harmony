use bridge_core::{Channel, Message, MessageSender, User};
use irc::client as irc_client;
use irc::proto::Command;

#[derive(Clone)]
pub struct IrcSender {
    pub(crate) inner: irc_client::Sender,
}

#[async_trait::async_trait]
impl MessageSender for IrcSender {
    async fn send_message(
        &self,
        target: &Channel,
        message: &Message,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let text = format!("<{}> {}", message.author.name, message.content);
        self.inner.send_privmsg(&target.id, text)?;
        Ok(())
    }
}

pub(crate) fn irc_to_core(msg: &irc::proto::Message) -> Option<Message> {
    match &msg.command {
        Command::PRIVMSG(channel, text) => {
            let nickname = msg.source_nickname().unwrap_or("unknown").to_owned();

            Some(Message {
                author: User {
                    name: nickname,
                    avatar_url: None,
                },
                channel: Channel {
                    id: channel.clone(),
                    name: channel.clone(),
                },
                content: text.clone(),
                attachments: Vec::new(),
            })
        }
        _ => None,
    }
}
