//! Integration tests for the message relay loop.

use std::time::Duration;

use harmony_testing::{
    CoreMessageSegment, PlatformChannel, PlatformId, PlatformMessage, PlatformMessageSegment,
    PlatformUser, expect, expect_none, send, test_world,
};

#[tokio::test]
async fn message_relayed_between_two_platforms() {
    let ctx = test_world! {
        platforms {
            alpha: ["#general", "#chat", "#lobby"],
            beta: ["#general", "#chat", "#lobby"],
        }
    }
    .start()
    .await;

    send!(ctx, alpha, "alice", "#general", "hello from alpha");
    expect!(ctx, beta, "#general", {
        content == "hello from alpha",
    });

    ctx.shutdown().await;
}

#[tokio::test]
async fn message_not_relayed_to_unlinked_platform() {
    let world = test_world! {
        platforms {
            alpha: ["#general"],
            beta: ["#general"],
            gamma: ["#unrelated"],
        }
    };
    let ctx = world.start().await;

    send!(ctx, alpha, "alice", "#general", "only for beta");
    expect!(ctx, beta, "#general", { content == "only for beta" });
    expect_none!(ctx, gamma);

    ctx.shutdown().await;
}

#[tokio::test]
async fn bidirectional_relay() {
    let ctx = test_world! {
        platforms {
            alpha: ["#general", "#chat", "#lobby"],
            beta: ["#general", "#chat", "#lobby"],
        }
    }
    .start()
    .await;

    send!(ctx, alpha, "alice", "#chat", "alpha->beta");
    expect!(ctx, beta, "#chat", { content == "alpha->beta" });

    send!(ctx, beta, "bob", "#chat", "beta->alpha");
    expect!(ctx, alpha, "#chat", { content == "beta->alpha" });

    ctx.shutdown().await;
}

#[tokio::test]
async fn three_way_relay() {
    let world = test_world! {
        platforms {
            alpha: ["#lobby"],
            beta: ["#lobby"],
            gamma: ["#lobby"],
        }
    };
    let ctx = world.start().await;

    send!(ctx, alpha, "alice", "#lobby", "broadcast");
    expect!(ctx, beta, "#lobby", { content == "broadcast" });
    expect!(ctx, gamma, "#lobby", { content == "broadcast" });

    ctx.shutdown().await;
}

// --- mention tests ---

fn inject_msg(
    platform: &str,
    author: &str,
    channel: &str,
    content: Vec<PlatformMessageSegment>,
) -> PlatformMessage {
    PlatformMessage {
        author: PlatformUser {
            platform: PlatformId::new(platform),
            id: author.to_owned(),
            display_name: Some(author.to_owned()),
            avatar_url: None,
        },
        channel: PlatformChannel {
            platform: PlatformId::new(platform),
            id: channel.to_owned(),
            name: channel.to_owned(),
        },
        content,
    }
}

#[tokio::test]
async fn mention_resolved_and_relayed() {
    let ctx = test_world! {
        platforms {
            alpha: ["#general"],
            beta: ["#general"],
        }
        users {
            bob: { alpha: "bob", beta: "bob" },
        }
    }
    .start()
    .await;

    ctx.control("alpha")
        .inject_message(inject_msg(
            "alpha",
            "alice",
            "#general",
            vec![
                PlatformMessageSegment::Text("hello ".to_owned()),
                PlatformMessageSegment::Mention("bob".to_owned()),
            ],
        ))
        .await;

    expect!(ctx, beta, "#general", { content == "hello @bob" });

    ctx.shutdown().await;
}

#[tokio::test]
async fn unresolved_mention_falls_back_to_text() {
    let ctx = test_world! {
        platforms {
            alpha: ["#general"],
            beta: ["#general"],
        }
    }
    .start()
    .await;

    ctx.control("alpha")
        .inject_message(inject_msg(
            "alpha",
            "alice",
            "#general",
            vec![
                PlatformMessageSegment::Text("hey ".to_owned()),
                PlatformMessageSegment::Mention("ghost".to_owned()),
            ],
        ))
        .await;

    expect!(ctx, beta, "#general", { content == "hey @ghost" });

    ctx.shutdown().await;
}

#[tokio::test]
async fn mention_segment_preserved_in_core_message() {
    // Verifies that a resolved mention arrives as a real Mention segment,
    // not just a Text("@bob") fallback.
    let ctx = test_world! {
        platforms {
            alpha: ["#general"],
            beta: ["#general"],
        }
        users {
            bob: { alpha: "bob", beta: "bob" },
        }
    }
    .start()
    .await;

    ctx.control("alpha")
        .inject_message(inject_msg(
            "alpha",
            "alice",
            "#general",
            vec![
                PlatformMessageSegment::Text("ping ".to_owned()),
                PlatformMessageSegment::Mention("bob".to_owned()),
            ],
        ))
        .await;

    let msg = ctx
        .control("beta")
        .next_message(Duration::from_secs(2))
        .await
        .expect("beta should receive a message");

    assert_eq!(msg.content.len(), 2);
    assert!(matches!(&msg.content[0], CoreMessageSegment::Text(t) if t == "ping "));
    assert!(
        matches!(&msg.content[1], CoreMessageSegment::Mention(u) if u.display_name() == Some("bob"))
    );

    ctx.shutdown().await;
}
