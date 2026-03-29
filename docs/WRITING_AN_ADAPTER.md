Writing an adapter
==================

This guide walks through the steps of implementing a new adapter for Harmony.
By the end you will have a crate that connects to a platform, discovers its
channels and users, relays messages into and out of the core, and emits
lifecycle events.

Before starting, read the *Core* and *Adapters* sections of
[CONTRIBUTING.md](CONTRIBUTING.md) to understand the adapter contract, the data
model, and the invariants your adapter must respect.

## 1. Create the crate

Create a new directory at the workspace root (e.g. `matrix/`) and add it to the
workspace members in the root `Cargo.toml`.

The adapter's `Cargo.toml` should depend on `harmony-core` (path dependency),
`tokio` (with at least the `sync` and `rt` features), `exn` (workspace), and
`log`. Add whichever client library your platform requires.

## 2. Define the adapter struct

The adapter struct holds whatever configuration and state is needed to connect
to the platform. Assign it a stable `PlatformId`:

```rust
pub struct MatrixAdapter {
    homeserver: String,
    token: String,
    platform_id: PlatformId,
}

impl MatrixAdapter {
    pub fn new(homeserver: String, token: String) -> Self {
        Self {
            homeserver,
            token,
            platform_id: PlatformId::new("matrix"),
        }
    }
}
```

## 3. Implement `PlatformAdapter`

`PlatformAdapter::start` is the heart of the adapter. It consumes `self`,
receives two `mpsc::Sender` channels from the core, and must:

1. Connect to the platform.
2. Discover channels and users.
3. Spawn background tasks for reading the platform's event stream.
4. Return a `PlatformHandle` with the adapter's capabilities and a shutdown
   handle.

## 4. Implement capability traits

### `SendMessage`

Receives a `CoreMessage` and delivers it to the platform. Resolve the target
channel and author identity from the core types:

The author resolution pattern (platform alias first, then fallback) is
important: it ensures the relayed message uses the name and avatar that are
most recognizable on the receiving platform.

### `ListChannels` and `ListUsers`

These return the snapshot of channels and users discovered during `start`. A
common pattern is to cache the discovery results in a simple `Clone` struct:

## 5. Convert platform types

Write conversion functions that translate the platform library's native types
into core types. Each inbound message must become a `PlatformMessage`:

## 6. Emit meta-events

Your event stream processing function should emit `MetaEvent` variants for
every lifecycle change the platform exposes. At minimum:

- **User joins**: `MetaEvent::UserJoined`
- **User leaves**: `MetaEvent::UserLeft`
- **User profile changes**: `MetaEvent::UserUpdated`
- **User renames** (if the platform has mutable identifiers): `MetaEvent::UserRenamed`
- **Channel created/deleted/renamed**: the corresponding `MetaEvent` variant

Always filter out events caused by the bot itself to avoid feedback loops.

## 7. Wire it into the app

Add the new adapter crate as a dependency in `app/Cargo.toml`, then construct
it in `create_adapters` alongside the existing adapters in `app/src/harmony.rs`.
Add the platform's configuration to `app/src/config.rs`.

## 8. Write integration tests

Use the `testing` crate's `FakePlatform` to write integration tests that
exercise the relay loop with your adapter's patterns in mind. You do not need
to connect to the real platform in integration tests; `FakePlatform` validates
that the core routes messages and events correctly for any adapter that
respects the contract.

See the *Testing* section of [CONTRIBUTING.md](CONTRIBUTING.md) for the full
testing DSL and conventions.
