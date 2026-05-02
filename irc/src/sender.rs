//! Sends bridged messages as IRC PRIVMSG.

use std::fmt::Write as _;

use exn::{Exn, OptionExt as _};
use harmony_core::{
    BoxFuture, CoreMessage, CoreMessageSegment, HarmonyError, PlatformId, SendMessage,
};
use irc::client as irc_client;

use crate::convert::format_mention;

#[derive(Clone)]
pub struct IrcSender {
    pub(crate) inner: irc_client::Sender,
    pub(crate) platform_id: PlatformId,
}

fn render_msg(platform_id: &PlatformId, msg: &CoreMessage) -> String {
    msg.content
        .iter()
        .fold(String::new(), |mut acc, seg| {
            match seg {
                CoreMessageSegment::Text(text) => acc.push_str(text),
                CoreMessageSegment::Mention(user) => {
                    if let Some(pu) = user.get_platform_user(platform_id) {
                        acc.push_str(&format_mention(&pu.id));
                    } else {
                        let name = user.display_name().unwrap_or("unknown");
                        let _ = write!(acc, "@{name}");
                    }
                }
                CoreMessageSegment::MessageRef(inner) => {
                    let body = render_msg(platform_id, inner);
                    let _ = write!(acc, "\nQuoting ");
                    if let Some(pu) = inner.author.get_platform_user(platform_id) {
                        acc.push_str(&format_mention(&pu.id));
                    } else {
                        let name = inner.author.display_name().unwrap_or("unknown");
                        let _ = write!(acc, "@{name}");
                    }
                    acc.push(':');
                    for line in body.lines() {
                        let _ = write!(acc, "\n> {line}");
                    }
                    acc.push('\n');
                }
            }
            acc
        })
        .trim_matches('\n')
        .to_owned()
}

fn format_message_from_core(
    platform_id: &PlatformId,
    display_name: &str,
    message: &CoreMessage,
) -> Vec<String> {
    render_msg(platform_id, message)
        .split('\n')
        .filter_map(|l| (!l.is_empty()).then(|| format!("<{display_name}> {l}")))
        .collect()
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

            for line in format_message_from_core(&self.platform_id, display_name, message) {
                self.inner
                    .send_privmsg(&channel.id, line)
                    .map_err(|e| HarmonyError::send(e.to_string()).temporary())?;
            }
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use harmony_core::{CoreChannel, CoreUser, Peered, PlatformChannel, PlatformUser};

    fn pid() -> PlatformId {
        PlatformId::new("irc")
    }

    fn user(name: &str) -> CoreUser {
        CoreUser::from_single_alias(
            pid(),
            PlatformUser {
                platform: pid(),
                id: name.to_owned(),
                display_name: Some(name.to_owned()),
                avatar_url: None,
            },
        )
    }

    fn channel() -> CoreChannel {
        CoreChannel::from_single_alias(
            pid(),
            PlatformChannel {
                platform: pid(),
                id: "#test".to_owned(),
                name: "#test".to_owned(),
            },
        )
    }

    fn msg(author: &str, content: Vec<CoreMessageSegment>) -> CoreMessage {
        CoreMessage {
            author: user(author),
            channel: channel(),
            content,
        }
    }

    #[test]
    fn netiquette_layout_emits_one_privmsg_per_quote_line() {
        let alice = msg(
            "alice",
            vec![CoreMessageSegment::Text("67 67 67".to_owned())],
        );
        let bob = msg(
            "bob",
            vec![
                CoreMessageSegment::MessageRef(Box::new(alice)),
                CoreMessageSegment::Text("Haha".to_owned()),
            ],
        );
        let charlie = msg(
            "charlie",
            vec![
                CoreMessageSegment::MessageRef(Box::new(bob)),
                CoreMessageSegment::Text("What am I looking at?".to_owned()),
            ],
        );
        let mine = msg(
            "me",
            vec![
                CoreMessageSegment::MessageRef(Box::new(charlie)),
                CoreMessageSegment::Text("It's a joke".to_owned()),
            ],
        );

        let lines = format_message_from_core(&pid(), "me", &mine);
        assert_eq!(
            lines,
            vec![
                "<me> Quoting @charlie:".to_owned(),
                "<me> > Quoting @bob:".to_owned(),
                "<me> > > Quoting @alice:".to_owned(),
                "<me> > > > 67 67 67".to_owned(),
                "<me> > > Haha".to_owned(),
                "<me> > What am I looking at?".to_owned(),
                "<me> It's a joke".to_owned(),
            ],
        );
    }

    #[test]
    fn ref_only_message_emits_no_trailing_empty_privmsg() {
        let alice = msg("alice", vec![CoreMessageSegment::Text("hi".to_owned())]);
        let mine = msg("me", vec![CoreMessageSegment::MessageRef(Box::new(alice))]);

        let lines = format_message_from_core(&pid(), "me", &mine);
        assert_eq!(
            lines,
            vec!["<me> Quoting @alice:".to_owned(), "<me> > hi".to_owned()],
        );
    }
}
