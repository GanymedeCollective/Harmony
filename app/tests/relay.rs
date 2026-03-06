//! Integration tests for the message relay loop.

use bridge_testing::{expect, expect_none, send, test_world};

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
