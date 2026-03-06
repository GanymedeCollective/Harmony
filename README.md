Bridge
======

Relays messages between chat servers. Channels are auto-matched by name across
platforms. Users are auto-correlated by display name so messages look nice on
all sides.

Usage
-----

```
bridge                        # run the bridge
bridge -c path/to/config.toml # custom config (defaults to runtime/config.toml or $BRIDGE_RUNTIME_DIR)
bridge -v/-vv                 # debug / trace logging
bridge --log-path bridge.log  # log to file instead of stderr
```

Config
------

Lives in `runtime/config.toml`. Just platform credentials:

```toml
[irc]
server = "irc.example.com"
port = 6697
use_tls = true
nickname = "MY-BRIDGE"

[discord]
token = "your-bot-token"
```

At startup the bridge connects to each platform, discovers channels and users
automatically, and links them by name. IRC channels are discovered via `LIST`
and joined; Discord channels come from the bot's guild. No manual channel or
user mapping is needed.

Architecture
------------

```
core/      platform-agnostic types, traits, relay loop, and collections
             types:  PlatformUser, CoreUser, PlatformChannel, CoreChannel, PlatformMessage, CoreMessage
             traits: PlatformAdapter, MessageSender, ListUsers, ListChannels
             collections: Users, Channels (indexed, with transitive auto-correlation)
irc/       IRC adapter (irc crate, LIST+JOIN at startup, stream handling)
discord/   Discord adapter (serenity + webhooks)
app/       composition root: config parsing, adapter wiring, CLI
testing/   fake platforms + test DSL for integration tests
```

Each platform crate implements `PlatformAdapter`, `MessageSender`, `ListUsers`,
and `ListChannels`. The app doesn't care what's behind them: adding a new
platform is a new crate + one line in `create_adapters()`.

At startup, each adapter is queried for its channels and users. Core builds
`Channels` and `Users` collections from discovered data, with transitive
auto-correlation by name (e.g. `#general` on IRC matches `general` on Discord;
if three platforms each have a `general` channel, they all end up linked).
Messages flow through channels (`mpsc`), get resolved to cross-platform
identities (`CoreUser`, `CoreChannel`), and are relayed to targets. Meta-events
(joins, leaves, renames) update the collections in memory.

Docker
------

```sh
docker build -t bridge .
docker run -v /path/to/config.toml:/config.toml -d bridge -c /config.toml
```

TODO
----

- [ ] Detect meta references (reference to other user, reference to channel ...)
- [ ] Attachment relay (download + reupload instead of just URLs)
- [ ] Reply/edit awareness
- [ ] Mechanism for threads: either ircv3, or create `#channel-name--thread-name` automagically
- [ ] Automations (slash commands) to link a user in discord and in irc
