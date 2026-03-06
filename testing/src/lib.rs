//! Test doubles and DSL for integration testing the bridge.
//!
//! Define a reusable [`TestWorld`] with [`test_world!`], instantiate it per test
//! with [`TestWorld::start()`](TestWorld::start), then script scenarios with
//! [`send!`], [`expect!`], and [`expect_none!`].

mod context;
mod fake_platform;
mod macros;
mod world;

pub use bridge_core::{
    CoreMessage, MetaEvent, PlatformChannel, PlatformId, PlatformMessage, PlatformUser,
};
pub use context::TestContext;
pub use fake_platform::{FakeControl, FakePlatform, FakePlatformBuilder};
pub use world::{PlatformSpec, TestWorld, TestWorldBuilder, UserSpec};
