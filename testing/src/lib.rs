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

pub use context::TestContext;
pub use fake_platform::{FakeControl, FakePlatform, FakePlatformBuilder};
pub use world::{PlatformSpec, TestWorld, TestWorldBuilder, UserSpec};

/// Render a [`CoreMessageRope`] as a plain string, formatting mentions as `@name`.
pub fn rope_to_text(rope: &[CoreMessageSegment]) -> String {
    use std::fmt::Write as _;

    rope.iter().fold(String::new(), |mut result, seg| {
        match seg {
            CoreMessageSegment::Text(t) => result.push_str(t),
            CoreMessageSegment::Mention(u) => {
                let _ = write!(result, "@{}", u.display_name().unwrap_or("unknown"));
            }
            _ => {}
        }
        result
    })
}
