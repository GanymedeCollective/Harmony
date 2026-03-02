//! Integration tests for the message relay loop.

use std::time::Duration;

use bridge::config::ChannelLink;
use bridge::fetched_data::FetchedData;
use bridge::run;
use bridge_core::{Channel, Message, User};
use bridge_testing::FakePlatform;

const TIMEOUT: Duration = Duration::from_secs(2);

fn channel_link(pairs: &[(&str, &str)]) -> ChannelLink {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

fn msg(author: &str, channel: &str, content: &str) -> Message {
    Message {
        author: User {
            id: None,
            name: author.to_owned(),
            display_name: None,
            avatar_url: None,
        },
        channel: Channel {
            id: channel.to_owned(),
            name: channel.to_owned(),
        },
        content: content.to_owned(),
        attachments: vec![],
    }
}

#[tokio::test]
async fn message_relayed_between_two_platforms() {
    let (alpha, alpha_ctl) = FakePlatform::new("alpha");
    let (beta, beta_ctl) = FakePlatform::new("beta");

    let handle = run::run(
        vec![alpha, beta],
        vec![channel_link(&[("alpha", "#general"), ("beta", "#general")])],
        vec![],
        FetchedData::default(),
        None,
    )
    .await
    .expect("bridge should start");

    alpha_ctl
        .inject_message(msg("alice", "#general", "hello from alpha"))
        .await;

    let (target_ch, relayed) = beta_ctl
        .next_message(TIMEOUT)
        .await
        .expect("beta should receive the relayed message");

    assert_eq!(target_ch.id, "#general");
    assert_eq!(relayed.content, "hello from alpha");
    assert_eq!(relayed.author.name, "alice");

    handle.shutdown().await;
}

#[tokio::test]
async fn message_not_relayed_to_unlinked_platform() {
    let (alpha, alpha_ctl) = FakePlatform::new("alpha");
    let (beta, beta_ctl) = FakePlatform::new("beta");
    let (gamma, gamma_ctl) = FakePlatform::new("gamma");

    let handle = run::run(
        vec![alpha, beta, gamma],
        vec![channel_link(&[("alpha", "#general"), ("beta", "#general")])],
        vec![],
        FetchedData::default(),
        None,
    )
    .await
    .expect("bridge should start");

    alpha_ctl
        .inject_message(msg("alice", "#general", "only for beta"))
        .await;

    let relayed = beta_ctl.next_message(TIMEOUT).await;
    assert!(relayed.is_some(), "beta should get the message");

    let leaked = gamma_ctl.next_message(Duration::from_millis(200)).await;
    assert!(leaked.is_none(), "gamma should NOT get the message");

    handle.shutdown().await;
}

#[tokio::test]
async fn bidirectional_relay() {
    let (alpha, alpha_ctl) = FakePlatform::new("alpha");
    let (beta, beta_ctl) = FakePlatform::new("beta");

    let handle = run::run(
        vec![alpha, beta],
        vec![channel_link(&[("alpha", "#chat"), ("beta", "#chat")])],
        vec![],
        FetchedData::default(),
        None,
    )
    .await
    .expect("bridge should start");

    alpha_ctl
        .inject_message(msg("alice", "#chat", "alpha->beta"))
        .await;
    let (_, relayed) = beta_ctl
        .next_message(TIMEOUT)
        .await
        .expect("beta should receive");
    assert_eq!(relayed.content, "alpha->beta");

    beta_ctl
        .inject_message(msg("bob", "#chat", "beta->alpha"))
        .await;
    let (_, relayed) = alpha_ctl
        .next_message(TIMEOUT)
        .await
        .expect("alpha should receive");
    assert_eq!(relayed.content, "beta->alpha");

    handle.shutdown().await;
}

#[tokio::test]
async fn three_way_relay() {
    let (alpha, alpha_ctl) = FakePlatform::new("alpha");
    let (beta, beta_ctl) = FakePlatform::new("beta");
    let (gamma, gamma_ctl) = FakePlatform::new("gamma");

    let handle = run::run(
        vec![alpha, beta, gamma],
        vec![channel_link(&[
            ("alpha", "#lobby"),
            ("beta", "#lobby"),
            ("gamma", "#lobby"),
        ])],
        vec![],
        FetchedData::default(),
        None,
    )
    .await
    .expect("bridge should start");

    alpha_ctl
        .inject_message(msg("alice", "#lobby", "broadcast"))
        .await;

    let (_, from_beta) = beta_ctl
        .next_message(TIMEOUT)
        .await
        .expect("beta should receive");
    let (_, from_gamma) = gamma_ctl
        .next_message(TIMEOUT)
        .await
        .expect("gamma should receive");

    assert_eq!(from_beta.content, "broadcast");
    assert_eq!(from_gamma.content, "broadcast");

    handle.shutdown().await;
}
