mod compat;

use anyhow::Result;
use bridge_core::Message;
use futures::prelude::*;
use irc::client::Client;
use tokio::sync::mpsc;

pub use compat::IrcSender;
pub use irc::client::data::Config as IrcConfig;

pub async fn run(
    config: IrcConfig,
    bot_nickname: String,
) -> Result<(mpsc::Receiver<Message>, IrcSender)> {
    let mut client = Client::from_config(config).await?;
    client.identify()?;

    let sender = IrcSender {
        inner: client.sender(),
    };

    let mut stream = client.stream()?;
    let (tx, rx) = mpsc::channel::<Message>(256);

    tokio::spawn(async move {
        while let Some(result) = stream.next().await {
            match result {
                Ok(msg) => {
                    if let Some(core_msg) = compat::irc_to_core(&msg) {
                        if core_msg.author.name == bot_nickname {
                            continue;
                        }
                        if tx.send(core_msg).await.is_err() {
                            log::warn!("irc: receiver dropped, stopping stream");
                            break;
                        }
                    }
                }
                Err(e) => {
                    log::error!("irc: stream error: {e}");
                    break;
                }
            }
        }
    });

    Ok((rx, sender))
}
