Harmony
=======

Relays messages between chat servers. Channels are auto-matched by name across
platforms. Users are auto-correlated by display name so messages look nice on
all sides.

Usage
-----

```
harmony                         # run Harmony
harmony -c path/to/config.toml  # custom config (defaults to runtime/config.toml or $HARMONY_RUNTIME_DIR/config.toml)
harmony -v/-vv                  # debug / trace logging
harmony --log-path harmony.log  # log to file instead of stderr
```

Config
------

Lives in `${HARMONY_RUNTIME_DIR:-runtime}/config.toml`. Just platform
credentials:

```toml
[irc]
server = "irc.cool-url.org"
port = 6697
use_tls = true
accept_invalid_certs = false # "" Dangerous ""
nickname = "HARMONY"

[discord]
token = "your-bot-token"
```

At startup, Harmony connects to each platform, discovers channels and users
automatically, and links them by name. IRC channels are discovered via `LIST`
and joined; Discord channels come from the bot's guild. No manual channel or
user mapping is needed. That means channels/users that ought be linked together
must have the same name (display name, nick, whatever your platform calls this).

Docker
------

```sh
docker build -t harmony .
docker run -v /path/to/config.toml:/config.toml -d harmony -c /config.toml
```
