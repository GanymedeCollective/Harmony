//! Integration tests for the message relay loop.

use std::time::Duration;

use harmony_testing::{
    CoreMessageSegment, PlatformChannel, PlatformId, PlatformMessage, PlatformMessageSegment,
    PlatformUser, expect, expect_none, rope_to_text, send, test_world,
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

// --- MessageRef / reply tests ---

fn pm(platform: &str, author: &str, channel: &str, text: &str) -> PlatformMessage {
    inject_msg(
        platform,
        author,
        channel,
        vec![PlatformMessageSegment::Text(text.to_owned())],
    )
}

#[tokio::test]
async fn message_ref_resolved_and_relayed() {
    // Bob is correlated across alpha + beta. Alice on alpha sends a reply
    // to a prior bob message; beta receives the reply with bob's beta
    // identity rejoined to the inner ref's author.
    // Both bob identities share the same display name (= identity, given
    // the test world's default), so auto-correlation links them.
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

    let bob_prior = pm("alpha", "bob", "#general", "first");

    ctx.control("alpha")
        .inject_message(inject_msg(
            "alpha",
            "alice",
            "#general",
            vec![
                PlatformMessageSegment::MessageRef(Box::new(bob_prior)),
                PlatformMessageSegment::Text("agreed".to_owned()),
            ],
        ))
        .await;

    let msg = ctx
        .control("beta")
        .next_message(Duration::from_secs(2))
        .await
        .expect("beta should receive a message");

    assert_eq!(msg.content.len(), 2);
    let CoreMessageSegment::MessageRef(inner) = &msg.content[0] else {
        panic!("expected MessageRef, got {:?}", &msg.content[0]);
    };
    let inner_author = &inner.author;
    let beta_alias = inner_author
        .get_platform_user(&PlatformId::new("beta"))
        .expect("inner ref author should have beta alias via auto-correlation");
    assert_eq!(beta_alias.id, "bob");
    assert_eq!(rope_to_text(&inner.content), "first");

    assert!(matches!(&msg.content[1], CoreMessageSegment::Text(t) if t == "agreed"));

    ctx.shutdown().await;
}

#[tokio::test]
async fn message_ref_unknown_author_fabricated() {
    // The ref's author is unknown to the bridge: it must survive as a
    // fabricated single-platform CoreUser carrying the platform display_name.
    let ctx = test_world! {
        platforms {
            alpha: ["#general"],
            beta: ["#general"],
        }
    }
    .start()
    .await;

    let ghost_msg = pm("alpha", "ghost", "#general", "boo");

    ctx.control("alpha")
        .inject_message(inject_msg(
            "alpha",
            "alice",
            "#general",
            vec![
                PlatformMessageSegment::MessageRef(Box::new(ghost_msg)),
                PlatformMessageSegment::Text("ok".to_owned()),
            ],
        ))
        .await;

    let msg = ctx
        .control("beta")
        .next_message(Duration::from_secs(2))
        .await
        .expect("beta should receive a message");

    let CoreMessageSegment::MessageRef(inner) = &msg.content[0] else {
        panic!("expected MessageRef, got {:?}", &msg.content[0]);
    };
    assert_eq!(inner.author.display_name(), Some("ghost"));
    assert!(
        inner
            .author
            .get_platform_user(&PlatformId::new("alpha"))
            .is_some(),
        "fabricated user should retain its source-platform alias",
    );
    assert!(
        inner
            .author
            .get_platform_user(&PlatformId::new("beta"))
            .is_none(),
        "fabricated user should NOT have a beta alias (no auto-correlation)",
    );

    ctx.shutdown().await;
}

#[tokio::test]
async fn multiple_refs_in_rope_relayed() {
    // Non-leading refs survive end-to-end with surrounding text segments.
    let ctx = test_world! {
        platforms {
            alpha: ["#general"],
            beta: ["#general"],
        }
    }
    .start()
    .await;

    let m1 = pm("alpha", "a1", "#general", "msg1");
    let m2 = pm("alpha", "a2", "#general", "msg2");

    ctx.control("alpha")
        .inject_message(inject_msg(
            "alpha",
            "alice",
            "#general",
            vec![
                PlatformMessageSegment::Text("see ".to_owned()),
                PlatformMessageSegment::MessageRef(Box::new(m1)),
                PlatformMessageSegment::Text(" and ".to_owned()),
                PlatformMessageSegment::MessageRef(Box::new(m2)),
            ],
        ))
        .await;

    let msg = ctx
        .control("beta")
        .next_message(Duration::from_secs(2))
        .await
        .expect("beta should receive a message");

    assert_eq!(msg.content.len(), 4);
    assert!(matches!(&msg.content[0], CoreMessageSegment::Text(t) if t == "see "));
    assert!(matches!(&msg.content[1], CoreMessageSegment::MessageRef(_)));
    assert!(matches!(&msg.content[2], CoreMessageSegment::Text(t) if t == " and "));
    assert!(matches!(&msg.content[3], CoreMessageSegment::MessageRef(_)));

    ctx.shutdown().await;
}

fn build_ref_chain(depth: usize) -> PlatformMessage {
    let mut current = inject_msg(
        "alpha",
        &format!("author{depth}"),
        "#general",
        vec![PlatformMessageSegment::Text("leaf".to_owned())],
    );
    for level in (1..depth).rev() {
        current = inject_msg(
            "alpha",
            &format!("author{level}"),
            "#general",
            vec![
                PlatformMessageSegment::MessageRef(Box::new(current)),
                PlatformMessageSegment::Text(format!("body{level}")),
            ],
        );
    }
    current
}

#[tokio::test]
async fn deeply_nested_refs_truncate_at_4() {
    // Build a chain of 7 nested PlatformMessages (each referencing the next).
    // Core conversion must preserve refs at quote levels 1..=4 and replace
    // the level-5 ref with a `> ...` truncation marker living inside the
    // depth-4 ref's content.
    let ctx = test_world! {
        platforms {
            alpha: ["#general"],
            beta: ["#general"],
        }
    }
    .start()
    .await;

    let depth7 = build_ref_chain(7);
    ctx.control("alpha").inject_message(depth7).await;

    let msg = ctx
        .control("beta")
        .next_message(Duration::from_secs(2))
        .await
        .expect("beta should receive a message");

    // The top-level message is author1 (no `>` prefix); refs at quote
    // levels 1..=4 hold author2..=author5. Author6 (would-be quote level 5)
    // is truncated.
    assert_eq!(msg.author.display_name(), Some("author1"));
    let mut cursor = &msg.content;
    for level in 1..=4 {
        let ref_seg = cursor
            .iter()
            .find_map(|s| match s {
                CoreMessageSegment::MessageRef(inner) => Some(inner.as_ref()),
                _ => None,
            })
            .unwrap_or_else(|| panic!("expected MessageRef at quote level {level}"));
        let expected = format!("author{}", level + 1);
        assert_eq!(
            ref_seg.author.display_name(),
            Some(expected.as_str()),
            "quote level {level} author mismatch",
        );
        cursor = &ref_seg.content;
    }

    let any_ref = cursor
        .iter()
        .any(|s| matches!(s, CoreMessageSegment::MessageRef(_)));
    assert!(
        !any_ref,
        "depth-5 MessageRef should have been truncated, not preserved",
    );
    let truncated = cursor
        .iter()
        .any(|s| matches!(s, CoreMessageSegment::Text(t) if t.contains("> ...")));
    assert!(
        truncated,
        "depth-5 ref should be replaced by a `> ...` truncation marker; got: {cursor:?}",
    );

    ctx.shutdown().await;
}
