Bridge
======

Relays messages between chat servers. Channels are mapped by config or auto
matched by name. Users can have cross-platform profiles (display name, avatar)
so messages look nice on all sides.

Usage
-----

```
bridge                        # run the bridge
bridge-fetch                  # fetch channels/users from all platforms, writes fetched_data.toml
bridge -c path/to/config.toml # custom config (defaults to runtime/config.toml or $BRIDGE_RUNTIME_DIR)
bridge -v/-vv                 # debug / trace logging
bridge --log-path bridge.log  # log to file instead of stderr
```

Config
------

Lives in `runtime/config.toml`. Define platform credentials, channel links, and
user profiles:

```toml
[irc]
server = "irc.example.com"
port = 6697
use_tls = true
nickname = "MY-BRIDGE"
channels = ["#general", "#random"]

[discord]
token = "your-bot-token"

[[channels]]
irc = "#general"
discord = "123456789"

[[users]]
irc = "someone"
discord = "987654321"
display_name = "Someone Cool"
avatar_url = "https://example.com/avatar.png"
```

Channels not explicitly linked are auto-correlated by name (e.g. `#general` on
IRC matches `general` on Discord). Same for users; matched by nickname across
platforms.

Architecture
------------

```
core/      platform-agnostic types and traits (Message, User, Channel, PlatformAdapter, MessageSender)
irc/       IRC adapter (irc crate + stream handling)
discord/   Discord adapter (serenity + webhooks)
app/       composition root: wires adapters, routes messages, handles events
runtime/   config + fetched data (runtime files)
```

Each platform crate implements `PlatformAdapter` and `MessageSender`. The app
doesn't care what's behind them: adding a new platform is a new crate + one
line in `create_adapters()`.

Messages flow through channels (`mpsc`), get enriched with user profiles, and
are relayed to targets. Meta-events (joins, leaves, renames) update a local
cache (`fetched_data.toml`) that feeds auto-correlation.

TODO
----

- [ ] Detect meta references (reference to other user, reference to channel ...)
- [ ] Attachment relay (download + reupload instead of just URLs)
- [ ] Reply/edit awareness
- [ ] Mecanism for threads: either ircv3, or create `#channel-name--thread-name` automagically
- [ ] Automations (slash commands) to link a user in discord and in irc
