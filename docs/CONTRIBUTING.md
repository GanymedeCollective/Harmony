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

The project
-----------

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
