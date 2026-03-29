HARMONY
=======

## Vocabulary

The documents in this project use the following terminology:

- *harmony*: *Harmony* refers to both the project itself and the technical
  solution it implements.
- *platform*: An existing implementation of a chat protocol, application, or
  ecosystem (e.g. IRC, Discord, Zulip, Matrix, Rocket.Chat, WhatsApp, Signal,
  or similar systems).
- The implementation of *harmony* is divided into two distinct components:
  - *core*: the internal representation of what a *platform* is and what a
    *platform* can do.
  - *adapter*: an implementation of *core* for a specific *platform*.

The key words "shall", "shall not", "should", "should not", and "may" are to be
interpreted as described in ISO/IEC Directives, Part 2, clause 7.

Vision
------

### Genesis

Modern communication is shaped by closed systems. While large corporations are
a visible part of this, the issue goes beyond them: most platforms, protocols,
and ecosystems are designed as isolated gardens. Even when they solve the same
problems and rely on similar underlying concepts, they are unable to
communicate with each other without intermediaries. These boundaries are often
presented as inherent, but in many cases they are simply the result of design
choices.

This raises a deeper question about freedom in communication. Choosing how and
where to communicate remains a personal decision. Adopting a specific platform,
protocol, or standard can be meaningful, but that choice does not extend to
others. Communication does not require alignment.

In practice, however, interaction is only possible when people share the same
space. This creates a subtle pressure: to communicate, one must conform. The
problem is not only that moving between platforms is difficult, but that it
becomes expected. Asking or convincing others to switch introduces friction
that does not originate from the nature of communication itself.

The consequence is a fragmented landscape where connections are limited by
incompatible systems. These limitations are not always technical necessities;
more often, they reflect how systems are designed and operated.

This project exists to challenge that fragmentation. Instead of requiring
people to converge on a single platform, it aims to make platforms
interoperable. Communication is able to flow regardless of where it originates,
allowing individuals to choose their tools freely without losing the ability to
reach others.

### Implications

The primary consequence of this vision is agnosticism.

First, agnosticism towards platforms. At their core, most chat systems revolve
around a small set of concepts: users, conversations, and messages. This
problem space is well understood. The core **shall** focus exclusively on these
shared primitives and the minimal structures required to enable communication.

Widely adopted quality-of-life features, such as threads or attachments,
**may** be represented in the core model. Platform-specific features **shall
not** be represented in the core. The core **shall not** attempt to replicate
platform-specific capabilities and **shall** remain a stable and minimal
foundation for interoperability.

This separation extends to adapters. Adapters **shall not** be tightly coupled
to the project. It **shall** be possible to provide, replace, or omit adapters
independently of the core. Adapters **may** be provided by the project for
convenience, but they are not required for a functional system. A working
instance **shall** depend only on the core.

Adapters **should** be stateless. They act as translators between external
platforms and the core and **should not** own business logic or persistence. All
processing **shall** flow through the core, which receives events and dispatches
actions.

Agnosticism also applies to users. The system **shall not** depend on a central,
authoritative identity provider. Users are represented independently of any
platform. When sufficient similarities exist across platforms, identities
**may** be correlated, allowing the system to treat them as a single user.

This enables a more consistent experience across platforms. Information such as
display names, profile pictures, or descriptions **may** be shared across
platforms where it is otherwise unavailable, without introducing a central
authority.

### Distribution

The distribution model is still evolving.

At present, enabling adapters requires compiling them together with the core
and modifying code to activate them. This approach is not practical and does
not align with the project’s goals.

The system **shall** support a modular distribution model in which the core and
adapters are independent components. A user **shall** be able to depend on
`harmony-core` alongside one or more `*-adapter` crates and start the system
through a single entry point.

The mechanism used to achieve this is not yet defined. It **may** involve
build-time integration, explicit registration of adapters at runtime, or a
combination of both.

The current distribution model does not meet these requirements and remains an
area of active development.

Implementation
--------------

### Architecture

```
.
├── app/       # Logic for the executable (config, args, logging, lunch, etc)
├── core/      # Harmony Core
├── discord/   # Adapter for discord
├── docs/      # Documentations (you are here)
├── irc/       # Adapter for irc
├── testing/   # Testing utils
├── Cargo.lock
├── Cargo.toml
├── Dockerfile
├── LICENSE
└── README.md  # Documentation for *how to use* and not *how it works*
```

### Core

The core (`harmony-core`) is the platform-agnostic foundation of the project.
It defines how platforms, users, channels, and messages are represented, how
entities are correlated across platforms, and how messages are relayed between
them. The core virtually does not know about any specific platform.

#### Data model

The data model uses a duality between platform-specific and cross-platform
types.

Platform types (`PlatformUser`, `PlatformChannel`, `PlatformMessage`)
represent a single platform's view of an entity. They carry a `PlatformId`
identifying which platform they belong to, a platform-specific `id`, and
whichever metadata the platform provides (display name, avatar URL, channel
name, etc.).

Core types (`CoreUser`, `CoreChannel`, `CoreMessage`) are cross-platform
aggregates. A `CoreUser` holds a `HashMap<PlatformId, PlatformUser>` of
aliases: one entry per platform the user has been observed on. `CoreChannel`
follows the same structure. These aggregates allow the core to reason about
entities independently of any single platform.

The direction of data determines which type is used. Inbound data (platform to
core) is always platform-typed: adapters produce `PlatformMessage` values
containing a `PlatformUser` and a `PlatformChannel`. Outbound data (core to
platform) is always core-typed: the relay builds a `CoreMessage` with a
`CoreUser` and `CoreChannel`, so the receiving adapter can look up its own
platform alias and render the message accordingly.

#### Error model

`HarmonyError` classifies errors along two axes: *kind* (what failed) and
*status* (whether it is safe to retry).

Kinds include `ConnectionFailed`, `SendFailed`, `DiscoveryFailed`,
`ConfigInvalid`, and `Internal`. Each constructor sets a sensible default
status (for instance, `connection` defaults to `Temporary`, `config` defaults
to `Permanent`). The `temporary()` and `permanent()` builder methods override
the status when the default is not appropriate.

The `is_temporary()` method drives the relay's retry logic: only temporary
errors are retried. Adapters **should** mark transient failures (rate limits,
network hiccups) as temporary and structural failures (invalid configuration,
missing permissions) as permanent.

#### Cross-platform correlation

Entities are automatically correlated across platforms through the `Peered`
trait and the `Peers<T>` collection.

`Peered` is implemented by `CoreChannel` and `CoreUser`. Its central method is
`match_key`, which produces a normalized string from a platform item used to
detect equivalence across platforms. For channels, the match key is the channel
name stripped of a leading `#` and lowercased (so `#General` on IRC matches
`general` on Discord). For users, the match key is the display name lowercased
(so `Alice` on Discord matches `alice` on IRC). Items that return `None` from
`match_key` are never auto-correlated.

`Peers<T>` is a slot-based arena with two hash-map indices: a primary index
mapping `(PlatformId, id)` to a slot for O(1) lookups, and a match-key index
mapping the normalized string to a slot for O(1) correlation. The `upsert`
method encodes the correlation logic:

1. If a core entity already exists for this `(platform, id)`, the alias is
   updated in place.
2. Otherwise, if an entity on a *different* platform shares the same match key,
   the item is merged into that entity (a new alias is added).
3. As a last resort, a new standalone entity is created.

This logic applies both at startup (via `auto_correlate`, which runs pairwise
across all discovered platforms) and at runtime (via `upsert`, called when
events arrive). The `detach` method removes a single platform alias from a core
entity; the entity is destroyed only when its last alias is removed.

#### Adapter contract

A platform integrates with the core through two components: the
`PlatformAdapter` trait (lifecycle) and a set of capability traits
(operations).

`PlatformAdapter` defines `start`, which consumes the adapter and receives two
`mpsc::Sender` channels: one for `(PlatformId, PlatformMessage)` and one for
`MetaEvent`. The adapter spawns its own background tasks and returns a
`PlatformHandle` containing:

- A `Box<dyn SendMessage>` for delivering relayed messages to the platform.
- A `Box<dyn ListChannels>` for discovering the platform's channels.
- A `Box<dyn ListUsers>` for discovering the platform's users.
- A `oneshot::Sender<()>` for signaling shutdown.

The three capability traits (`SendMessage`, `ListChannels`, `ListUsers`) are
object-safe and return `BoxFuture` values. They are stored as trait objects in
the core and called during the relay loop and initial discovery.

All communication between the core and adapters flows through these channels
and trait objects. Adapters push inbound data (messages and events) into the
core via the `mpsc` channels. The core pushes outbound data (relayed messages)
to adapters via `SendMessage`.

#### Relay loop

The `run` function in `core::run` orchestrates the entire lifecycle:

1. Two `mpsc` channels are created: one for messages, one for events.
2. All adapters are started concurrently via `join_all`, each receiving clones
   of the senders.
3. `PlatformHandle`s are collected and the original senders are dropped (so the
   channels close naturally when all adapters drop theirs).
4. Discovery runs: `list_channels` and `list_users` are called on every
   platform, and the results are fed into `Peers::build`, which runs
   `auto_correlate` to link entities across platforms.
5. State is wrapped in `Arc<RwLock<...>>` and a relay loop is spawned on a
   dedicated Tokio task.

The relay loop uses `tokio::select!` to multiplex two sources:

- **Messages** (`msg_rx`): routed through `dispatch`, which resolves the source
  channel to a `CoreChannel` (read lock), resolves or registers the author as
  a `CoreUser` (write lock, via `upsert` which also handles auto-correlation
  for previously unknown users), builds a `CoreMessage`, and fans out to every
  other platform in the channel's alias map via `SendMessage`. Failed sends are
  retried up to three times with exponential backoff, but only when the error
  is marked as temporary.
- **Events** (`event_rx`): routed through `handle_event`, which mutates the
  in-memory `Peers` collections. `UserJoined`, `UserUpdated`,
  `UsersDiscovered`, `ChannelCreated`, and `ChannelUpdated` are handled via
  `upsert` (which triggers auto-correlation if a match key hits). `UserLeft`
  and `ChannelDeleted` are handled via `detach`. `UserRenamed` uses a dedicated
  `rename` method that reindexes both the primary and match-key indices.

#### Meta-events

`MetaEvent` is an enum representing lifecycle changes that adapters observe at
runtime. Adapters **shall** emit the appropriate variant whenever they observe
any of the following:

| Event               | Meaning                                            |
|---------------------|----------------------------------------------------|
| `UserJoined`        | A user became visible on the platform              |
| `UserLeft`          | A user is no longer visible                        |
| `UserUpdated`       | A user's metadata (display name, avatar) changed   |
| `UserRenamed`       | A user's platform-specific identifier changed      |
| `UsersDiscovered`   | A batch of users was discovered at once            |
| `ChannelCreated`    | A new channel appeared                             |
| `ChannelDeleted`    | A channel was removed                              |
| `ChannelUpdated`    | A channel's metadata (name) changed                |

These events are the only mechanism for keeping the core's view of the world
up to date after initial discovery.

#### Invariants

- The core **shall not** contain any platform-specific logic. All
  platform-specific behavior belongs in adapters.
- `match_key` implementations **shall** be deterministic, case-insensitive,
  and strip platform-specific syntax (such as the `#` prefix on IRC channel
  names).
- The core **shall not** silently lose messages due to transient failures
  without retrying. Temporary send failures **shall** be retried with bounded
  backoff.
- Adapters **shall** emit `MetaEvent`s for every user and channel lifecycle
  change they can observe. Failure to do so results in stale state in the core.
- The relay loop **shall** remain responsive: a slow or failing adapter
  **shall not** block relay for other platforms. The retry mechanism **shall**
  be bounded.

### Testing

The `testing` crate (`harmony-testing`) provides test doubles and a concise DSL
for integration testing the relay loop. All integration tests run against the
real core relay (via `core::run::run`) with fake adapters standing in for
actual platforms.

#### `FakePlatform`

`FakePlatform` implements `PlatformAdapter`. It is wired with internal `mpsc`
channels to give tests two-sided control:

- **Inject side** (`FakeControl`): the test pushes `PlatformMessage` and
  `MetaEvent` values into the core through `inject_message` and
  `inject_event`.
- **Capture side** (`FakeControl`): the test reads `CoreMessage` values that
  the core routed to this platform through `next_message`, which waits up to a
  caller-specified timeout.

`FakePlatform::builder(name)` allows pre-configuring the channels and users
returned by `ListChannels` / `ListUsers`, so that discovery and
auto-correlation work as expected in tests.

#### `TestWorld`

A `TestWorld` declaratively defines a test scenario: which platforms exist,
which channels each platform has, and which users are present (with per-platform
identities). `TestWorld::start()` instantiates the world by creating one
`FakePlatform` per platform spec, passing the configured channels and users,
and calling `core::run::run` with the resulting adapters. It returns a
`TestContext` holding all `FakeControl` handles.

#### DSL macros

Four macros form the test DSL:

- `test_world!` defines platforms and users declaratively. Channels with
  matching names across platforms are automatically linked by the core's
  auto-correlation. The `users` section is optional.

```rust
let world = test_world! {
    platforms {
        alpha: ["#general", "#chat"],
        beta: ["#general"],
    }
    users {
        alice: { alpha: "4l1c3", beta: "Alice" },
    }
};
```

- `send!(ctx, platform, author, channel, content)` injects a message. The
  author name is resolved through the test world's user identity map: if it
  matches a known canonical name, the platform-specific identity is used;
  otherwise the string is used as-is.
- `expect!(ctx, platform, channel, { field == value, ... })` waits up to two
  seconds for a relayed message and asserts the target channel and arbitrary
  fields on the `CoreMessage`.
- `expect_none!(ctx, platform)` asserts that no message arrives within 200
  milliseconds.

#### Writing a test

The pattern for an integration test is:

1. Define a world with `test_world!`.
2. Call `.start().await` to get a `TestContext`.
3. Inject messages with `send!`.
4. Assert outcomes with `expect!` or `expect_none!`.
5. Call `ctx.shutdown().await`.

```rust
#[tokio::test]
async fn message_relayed_between_two_platforms() {
    let ctx = test_world! {
        platforms {
            alpha: ["#general"],
            beta: ["#general"],
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
```

#### Invariants

- Integration tests **shall** use the real core relay loop via `FakePlatform`,
  not mock or bypass it.
- Tests **should** cover both the positive case (message is relayed to the
  expected platforms) and the negative case (message is *not* relayed where it
  should not be, via `expect_none!`).
- Tests **shall** call `ctx.shutdown().await` to cleanly tear down the relay
  and all fake adapters.

### Adapters

An adapter is a crate that implements the core's `PlatformAdapter` trait and
capability traits for a specific platform. The project currently ships two
adapters (`discord-adapter` and `irc-adapter`) which serve as reference
implementations. This section documents the shared patterns and
platform-specific details that are important to understand when working on or
reviewing adapter code.

#### General pattern

Every adapter follows the same lifecycle:

1. **Construction**: the adapter is created from platform-specific
   configuration (a token, a server address, etc.) and assigned a `PlatformId`.
2. **Start**: `PlatformAdapter::start` is called. The adapter connects to the
   platform, performs initial discovery, spawns one or more background tasks
   for reading the platform's event stream, and returns a `PlatformHandle`.
3. **Steady state**: the background tasks forward inbound messages as
   `PlatformMessage` values and lifecycle changes as `MetaEvent` values into
   the core's channels. The core calls `SendMessage` on the adapter whenever
   a message needs to be relayed to this platform.
4. **Shutdown**: the core sends `()` on the `oneshot` channel. The adapter
   cleans up and disconnects.

Discovery happens inside `start`, before the `PlatformHandle` is returned.
The channels and users discovered at this point are made available through
`ListChannels` and `ListUsers`, which the core calls immediately after all
adapters have started.

#### Discord

The Discord adapter (`discord-adapter`) uses the
[Serenity](https://github.com/serenity-rs/serenity) library.

**Connection.** A Serenity `Client` is built with gateway intents for guild
messages, direct messages, message content, guilds, and guild members. A
`Handler` implementing Serenity's `EventHandler` is registered to receive
gateway events. The client is spawned on a background task; shutdown is
handled by calling `shard_manager.shutdown_all()` when the shutdown signal
arrives.

**Discovery.** Guild channels and members are fetched via the HTTP API (not
the gateway) through `fetch_guild_data`, which paginates through all guilds the
bot is in, collects text channels, and paginates through members 1000 at a
time, deduplicating by user ID.

**Inbound messages.** The `Handler` receives `message` gateway events. Bot
messages and self-messages are filtered out. The Serenity `Message` is
converted to a `PlatformMessage` via `discord_to_core`, which extracts the
best display name (guild nickname, then global name, then username), appends
attachment URLs to the content, and resolves the author's avatar URL.

**Meta-events.** The `Handler` maps Serenity gateway events to `MetaEvent`
variants: `guild_member_addition` to `UserJoined`, `guild_member_removal` to
`UserLeft`, `guild_member_update` to `UserUpdated`, and the `channel_create`,
`channel_delete`, `channel_update` events to their `MetaEvent` equivalents.
Non-text channels are filtered out.

**Outbound messages.** `DiscordSender` implements `SendMessage` using
per-channel webhooks. For each target channel, it lazily creates (or reuses) a
webhook named "Bridge" via a double-checked locking pattern. Messages are sent
by executing the webhook with the original author's display name and avatar
URL, so relayed messages appear as if sent by the original user rather than the
bot.

#### IRC

The IRC adapter (`irc-adapter`) uses the
[irc](https://github.com/aatxe/irc) library.

**Connection.** A `Client` is created from configuration with an empty channel
list (channels are discovered, not preconfigured). The client registers with
the server via `identify`.

**Discovery.** The `discover_and_join` function waits for the end-of-MOTD
reply (indicating registration is complete), sends `LIST` to discover all
channels on the server, joins every discovered channel on `RPL_LISTEND`, then
sends a sentinel `PING` and collects all `RPL_NAMREPLY` nicknames until the
matching `PONG` returns. This gives a complete snapshot of channels and users,
cached in `IrcLister` for `ListChannels` / `ListUsers`. The entire discovery
phase has a 10-second timeout.

**Inbound messages.** The `process_stream` function reads the IRC message
stream. `PRIVMSG` commands are converted to `PlatformMessage` via
`irc_to_core`, which extracts the source nickname and channel name. Self-
messages (matching the bot's nickname) are filtered out.

**Meta-events.** `JOIN` emits `UserJoined`, `QUIT` emits `UserLeft`, `NICK`
emits `UserRenamed` (or updates the bot's own nickname if it is the one being
renamed), and `RPL_NAMREPLY` emits `UsersDiscovered`. The bot itself is always
filtered from user events.

**Outbound messages.** `IrcSender` implements `SendMessage` by formatting the
content as `<display_name> message` and sending a `PRIVMSG` to the target
channel.

#### Invariants

- Adapters **can** filter or transform message content if structurally
  necessary (e.g. appending attachment URLs that would otherwise be lost).
- Adapters **should** skip messages originating from the bot itself to prevent
  echo loops.
- Adapters **shall** emit `MetaEvent`s for all observable lifecycle changes on
  their platform. The core relies on these events to keep its state current.
- Adapters **should** be stateless with respect to business logic. They act as
  translators and **should not** make routing or correlation decisions; those
  belong in the core.
- An adapter's `SendMessage` implementation **should** resolve the author's
  display name and avatar from the `CoreMessage`, preferring the platform-
  specific alias when available, and falling back to the first available alias.
  This ensures relayed messages are attributed consistently.

For a step-by-step guide on implementing a new adapter from scratch, see
[WRITING_AN_ADAPTER.md](WRITING_AN_ADAPTER.md).
