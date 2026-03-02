//! Integration tests for the message relay loop.

use bridge_testing::{TestWorld, expect, expect_none, send, test_world};

fn two_platform_world() -> TestWorld {
    test_world! {
        platforms {
            alpha: ["#general", "#chat", "#lobby"],
            beta: ["#general", "#chat", "#lobby"],
        }
        channels {
            alpha "#general" = beta "#general",
            alpha "#chat" = beta "#chat",
            alpha "#lobby" = beta "#lobby",
        }
    }
}

#[tokio::test]
async fn message_relayed_between_two_platforms() {
    let ctx = two_platform_world().start().await;

    send!(ctx, alpha, "alice", "#general", "hello from alpha");
    expect!(ctx, beta, "#general", {
        content == "hello from alpha",
        author.name == "alice",
    });

    ctx.shutdown().await;
}

#[tokio::test]
async fn message_not_relayed_to_unlinked_platform() {
    let world = test_world! {
        platforms {
            alpha: ["#general"],
            beta: ["#general"],
            gamma: ["#general"],
        }
        channels {
            alpha "#general" = beta "#general",
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
    let ctx = two_platform_world().start().await;

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
        channels {
            alpha "#lobby" = beta "#lobby" = gamma "#lobby",
        }
    };
    let ctx = world.start().await;

    send!(ctx, alpha, "alice", "#lobby", "broadcast");
    expect!(ctx, beta, "#lobby", { content == "broadcast" });
    expect!(ctx, gamma, "#lobby", { content == "broadcast" });

    ctx.shutdown().await;
}
