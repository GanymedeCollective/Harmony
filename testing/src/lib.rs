//! Test doubles and DSL for integration testing Harmony.
//!
//! Define a reusable [`TestWorld`] with [`test_world!`], instantiate it per test
//! with [`TestWorld::start()`](TestWorld::start), then script scenarios with
//! [`send!`], [`expect!`], and [`expect_none!`].

mod context;
mod fake_platform;
mod macros;
mod world;

pub use harmony_core::{
    CoreMessage, CoreMessageSegment, MetaEvent, PlatformChannel, PlatformId, PlatformMessage,
    PlatformMessageSegment, PlatformUser,
};

/// Render a [`CoreMessageRope`] as a plain string, formatting mentions as `@name`.
pub fn rope_to_text(rope: &[CoreMessageSegment]) -> String {
    rope.iter()
        .map(|seg| match seg {
            CoreMessageSegment::Text(t) => t.clone(),
            CoreMessageSegment::Mention(u) => {
                format!("@{}", u.display_name().unwrap_or("unknown"))
            }
            _ => panic!("Unimplemented CoreMessageSegment variant encountered"),
        })
        .collect()
}
pub use context::TestContext;
pub use fake_platform::{FakeControl, FakePlatform, FakePlatformBuilder};
pub use world::{PlatformSpec, TestWorld, TestWorldBuilder, UserSpec};
