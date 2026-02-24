Unnamed IRC<->Discord bridge
============================

Architecture
------------

### app

Parse args, config, create the futures and select send them to tokio

### core

Define types and traits that must be implemented by both sides to make the thing

### discord / irc

Impl of each side of the cargo traits/structs, and side specific stuff (events
etc)

### utils

Self explanatory

### runtime

runtime files, like stuff that needed at runtime, you can see these as the
default" config files. In my vision this will ultimately contain the users
encountered, which would be able to register avatars or stuff like that for
them, or special pretty printing in each side etc something fun and
configurable.

TODO
----

- [ ] Check if this works :zzzz:
