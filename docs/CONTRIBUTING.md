HARMONY
=======

Vocabulary
----------

The documents in this project use a few points of vocabulary that are specified
here:

- *harmony*: *Harmony* is the name of the project. It's both the idea that will
  be described throughout the documentation, and the technical solution it
  implements and provides.
- *platform*: An existing implementation of a chatting server protocol/
  application/ecosystem. Like irc, discord, zulip, matrix, rocketchat, whatsapp,
  signal, and even social media group chats or internet forums.
- *harmony*'s implementation is separated into 2 distinct "objects" in natures:
  - *core*: the internal representation of what a *platform* **is**, and what a
    *platform* **can do**.
  - *adapter*: implementation of *core* for a *platform*

Also, the documents of the project aims to respect
[ISO/IEC Directives, Part 2, clause 7](https://www.iso.org/sites/directives/current/part2/index.xhtml#_idTextAnchor078)
regarding verbal forms.

The project
-----------

### Genesis

<!-- TODO: make sentences -->

blabla proprietary software and corporations being the backbone of our social
circles and communication is bad

blabla not everyone wants to go through the pain of switching platform every
other day, and even people who are willing to make the sacrifice can't (and
even shouldn't) force their social circle to do it, and our generation is
lonely enough to not create silos artificially or for moral reasons.

blabla closed gardens are artificial anyway for most chatting protocols

<!-- !TODO: make sentences -->

### Implications

The implication for the product is agnosticism.

Agnosticism first towards the platforms, a chatting server can be easiely
represented with a few simple structures, as what we want is just sending text
from users.

It's pretty much a solved problem since the very early days of the internet.
Some QOL that are pretty much ubiquotous can be represented in *core*, like
threads, or attachments, but platform specific options **ought not** exist in
*core*. *Core* should be just the internal logic to make things work.

*Adapters* **should not** "belong" to the project, it should be easy to BYOA
bring your own adapter). Adapters **may** be provided by this project, but only
for convenience, and alleviating the burden of implementation.

Nothing outside core should be necessary to roll out your instance.

A side effect of this is that adapters **must** be stateless. Everything goes
through core, and core dispatch back everything to the adapters. But we surely
will talk more in depth about this in the implementation section.

Secondly agnosticism towards the users. We **ought** make our concept of what a
user *is* independant to platforms, even for us. There should be no "central"
authority for identification. Currently the way it works is through correlation,
if on different *platforms* two users share enough characteristics, they **may**
be correlated into a single *user*. That will be practical to enrich messages
on platforms on which the user with absent with qol things like a profile
picture, a display name, or even a user description.


### Distribution

It's not yet clear to me what distribution will look like. Currently you
**need** to have things compiled together and modify code to enable adapters by
name. It's not practical. In my mind it should just be `harmony-core` and
`*-adapter` in the dependancies, you call `harmony-core::run()`, and everything
works. I don't know how to make that work, maybe a `build.rs` script can
instrument things, maybe we move the burden a little and in the
`harmony-core::run(...)` you have to provide the adapters, idk. But as things
stands currently, it's not a happy distribution path.

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
