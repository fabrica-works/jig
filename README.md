# jig

*Engine for agentic applications.*

A Rust kit that runs an application's work, done by agents, procedures
and people, durably and within the authority each task was given.

## The idea

- **As flexible as an agent at a terminal, and more.** jig adds what such
  an agent lacks: work that outlives processes and machines, many agents
  at once, rules no agent can loosen, and effects on other systems made
  once.
- **Five primitives.** The task is what is asked; its executor, an
  agent, a procedure or a person, does it; messages are all it hears
  once started; authority is what it may do, narrowed whenever it
  delegates; connectors are the external systems. Plans, coordinators,
  chats, goals and reviews are built from them.
- **A kit, not a framework.** The application owns the top of every
  process: its root composes jig's parts with its own connectors and
  domain logic. jig has no application trait, no plug-in points and no
  callbacks, and decides nothing that is the application's: not its
  server, its URLs or its presentation.
- **The hard parts are jig's.** The code an application writes around jig
  only routes and translates. The commit barrier, restart, authority and
  effects made once are jig's, and its conformance world and referee
  check, in the application's own worlds, that the wiring kept them.
- **Agents are optional.** An application may have none, run them in
  its engine, or run them on workers. jig's core never depends on smith.
- **Built when pulled,** as skein is. temper's needs decide what jig
  holds and when; `ops`, a small second application, keeps jig from
  becoming temper's in disguise.

## Start here

- [`docs/design/README.md`](docs/design/README.md): what jig is, how an
  application is built around it, and the promises it keeps.
- [`docs/design/examples.md`](docs/design/examples.md): `ops`, the
  example application, whose root every application copies.
- [`docs/design/STATUS.md`](docs/design/STATUS.md): how far the code is
  from the design.
- skein's foundation, which jig follows: `programming-model.md`,
  `testing-strategy.md` and `notes.md`.

Before changing code, read [`AGENTS.md`](AGENTS.md). `main` moves only
through skein's `land`, which runs the gate, `scripts/check.sh`.

## The family

These are the repositories of [Fabrig](https://github.com/fabrica-works), an
agent-driven software factory: write, prove, run.

| Repository | | Builds on |
|---|---|---|
| [skein](https://github.com/fabrica-works/skein) | One thread, one loop, no async | |
| [smith](https://github.com/fabrica-works/smith) | Protocol-driven agent loop for AI applications | skein |
| [jig](https://github.com/fabrica-works/jig) | Engine for agentic applications | skein, smith |
| [temper](https://github.com/fabrica-works/temper) | Agent-driven software development on a forge | skein, smith, jig |
| [hearth](https://github.com/fabrica-works/hearth) | Immutable machines, built from a revision | skein |
