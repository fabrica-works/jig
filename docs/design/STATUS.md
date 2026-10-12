# Status

Updated 2026-10-12. How far jig's code is from its design, and which
plans take it there. The designer keeps this file (skein's
development.md, section 3.3).

## 1. Plans

| Plan | State | Takes the design to | Where |
|---|---|---|---|
| jig-protocol-implementation | drafting | `README.md` 15 (the crates' new names); `protocol/` but the store; `domain/engine.md` 6.1, 7.1, 8, 11, 12; `domain/hosts.md` 2, 6.1, 6.5, 8; `domain/people.md` 3, 5.1, 6; `domain/tasks.md` 9; `domain/root.md` 11; `client/domain/contract.md` 5.1, 6; `client/domain/client.md` 12 | `~/src/rust/plans/jig-protocol-implementation/` (not versioned) |
| jig-client-implementation | done 2026-10-11 | `client/domain/` above the protocol line; `client/domain/contract.md` 9 (the engine's changes); `examples.md` 8 (`ops`'s client) | `~/src/rust/plans/jig-client-implementation/` (not versioned) |
| jig-protocol-design | done 2026-10-10 | a design pass, with no code: `protocol/`, `draft/store.md` | `~/src/rust/plans/jig-protocol-design/` (not versioned) |
| jig-move | done 2026-10-09 | no section: jig in a repository of its own, with its workspace, lints and gate | `~/src/rust/plans/jig-move/` (not versioned) |
| jig-extraction | done 2026-10-09 | `domain/`: the core and its children, the hub, the inline agent, the conformance world; `README.md` 6.2 (the journal, in skein-lib); `examples.md` 2–6, 9, 10 and two stories of 7 (`ops`) | `~/src/rust/plans/jig-extraction/` (not versioned) |

jig-protocol-implementation has not landed. It waits for skein's and
smith's `design/jig-standalone` on their `main` (skein's may land from
any machine since 2026-10-12; smith's waits for the reliability plan's
end, or the user's exception), and for the user's answers in its
`decisions.md`. Section 2 says "not planned" for what it would build.

## 2. The design

| Document | Built | Left |
|---|---|---|
| `README.md` | partly | 8: the store, beyond its fake; 9: agents on workers, over the workers' channel; 10: the client's wire and shells; 15: the crates' new names; 14: versioning, open: not planned |
| `domain/README.md` | partly | 5: a task's proposal that nobody may accept, refused to its proposer (a party's is): not planned |
| `domain/core.md` | yes | 12: open questions only |
| `domain/tasks.md` | partly | 9: a task's proposal that nobody may accept, refused rather than kept waiting: not planned |
| `domain/authority.md` | yes | 12: open questions only |
| `domain/connectors.md` | partly | 13: a system's request budget (fresh reads first, reserved shares, a stop until the reset): not planned |
| `domain/engine.md` | partly | 6.1: stopping; 7.1: several charters; 8: grants pushed, and resent at a hello and an adoption; a run's text from hosts through the fleet; 11: text by retry, a gap marked cut; 12: accounts on skein's driver: not planned |
| `domain/hosts.md` | partly | 2, 6.1: the hub's hello and notices; 5.2: the inline agent on smith's completion contract and text; 6.5: typed tool and check facts; 8: host tools decoded outside the root: not planned |
| `domain/people.md` | partly | 3: a reply naming its own sign-in; 5.1: a decision on an escalation no longer waiting, refused as not waiting; 6: a chat forgotten once ended and read: not planned |
| `domain/root.md` | partly | 11, 12: the reference root decodes host tools' calls, a call's key and a run's result itself: not planned |
| `domain/testing.md` | partly | 7: the store's fake on skein-kv's in-memory mode (`jig-fake-store` is plain Rust): not planned |
| `client/domain/README.md` | partly | 2: the client's wire, link and shells (`protocol/client.md`): not planned |
| `client/domain/client.md` | partly | 12: signing out from the frame; 16: the protocol layer below the domain, but turns' decoding: not planned |
| `client/domain/contract.md` | partly | 5.1: a turn's bytes in its activity line; 6: reads refused as signed out; 8: a range of versions served; 10: the wire, one connection per client, the browser's client served by the engine: not planned |
| `client/domain/objects.md` | yes | 13: open questions only |
| `client/domain/pages.md` | partly | 7: a connector's request budget, and what its system last said: not planned |
| `client/domain/root.md` | partly | 11: a service's root, its small store in a file, its world: not planned |
| `client/domain/testing.md` | partly | 2: the protocol world, the real loop, the browser tier; 3: the fake person's browser face: not planned |
| `client/domain/view.md` | partly | 11: the browser's and the native shells, drawings in a shell; 12: roles mapped by the browser's shell; 13: the browser's stylesheet: not planned |
| `protocol/README.md` | partly | 5: every component but the charter's translation and turns' decoding (`jig-charter`, `jig-client-transcript`); 7: versions; 8: the secrets' tables; 9: the store; 12: what the domain owes, as the `domain/` rows say: not planned |
| `protocol/channel.md` | no | All: the workers' wire, the engine's and the worker's sides, grants, its world. The hub and the fleet meet without bytes: not planned |
| `protocol/client.md` | no | All: the client's wire, the engine's and the client's sides, sign-ins, the shells, wasm32 builds, its world, the browser tier: not planned |
| `protocol/configuration.md` | no | All: jig's part of a configuration, profiles, secrets from private files, first start, operators' formats: not planned |
| `protocol/llm.md` | no | All: smith's LLM component in the engine, the accounts' translation and table, grants, its world. The worlds answer the inline agent with a scripted LLM: not planned |
| `protocol/service.md` | no | All: the engine's and a worker's services, `iterate`, start and end, the reference test service, its world, one process: not planned |
| `examples.md` | partly | 7.3: a queue for the pool, policy narrowed, a month of alerts (no world found, not confirmed); the conformance world crashes three stories, not all; 8: a terminal client; 9.1: `ops` standalone: not planned |

## 3. Not planned

- **smith's latest `main`** (`README.md`, section 9;
  `protocol/README.md`, section 11): `jig-inline-agent`, `jig-charter`
  and `jig-client-transcript` link smith at `6f5b03c`, 338 commits behind
  its `main`. Since then smith changed its host protocol, and dropped
  charter version 1 and transcript version 2. Taking it is a repin and a
  change to those three crates. Waits for a plan: the drafted
  jig-protocol-implementation repins smith in its jig 02, but was written
  before these changes.
- **jig's protocol layer** (`protocol/README.md`, section 5; `README.md`,
  section 15): the components, the reference test service, the protocol
  worlds, and the crates' new names. Drafted as
  jig-protocol-implementation. Waits for skein's and smith's designs on
  their `main`: smith's is held back until the reliability plan ends.
- **What the domain owes the protocol layer** (`protocol/README.md`,
  section 12): the engine stopping, several charters, grants pushed,
  text by retry, accounts on skein's driver, the hub's hello, typed tool
  facts, host tools outside the root, people's and tasks' refusals,
  signing out (section 2's rows). Drafted as the same plan's jig 03 to
  10. Typed facts and the inline agent's contract wait for smith's
  `main`.
- **The store** (`protocol/README.md`, section 9; `domain/testing.md`,
  section 7): jig's records on skein-kv, transcripts, the secrets' store,
  and the worlds' fake on skein-kv. Waits for a design from
  `draft/store.md`. The worlds use `jig-fake-store` until then.
- **`ops` standalone** (`examples.md`, 9.1; `protocol/service.md`,
  section 7): its systems' APIs, its connectors' protocols, its service
  and its binary. Waits for the protocol layer, then a plan of its own.
- **An application's binary and real loop** (`protocol/service.md`,
  sections 3 and 8; `protocol/configuration.md`, sections 7 and 10):
  waits for the first application's standalone mode.
- **Between hosts** (`protocol/channel.md`, section 10;
  `protocol/client.md`, section 4; `domain/hosts.md`, section 10): TLS
  for the engine's listeners, and contained process trees for workers'
  agents. Waits for skein's TLS server side and skein's
  `draft/process.md`.
- **A connector's request budget** (`domain/connectors.md`, section 13;
  `client/domain/pages.md`, section 7): and the deployment's page showing
  it. Waits for a system that limits its rate.
- **A service's client root** (`client/domain/root.md`, section 11):
  waits for a service that acts as a party.
- **Several people on one deployment** (`client/domain/README.md`,
  sections 5 and 6): who may see what, a role's entries shared and taken.
  The client is built for one person first. Waits for such a deployment.
- **`ops`'s remaining stories** (`examples.md`, 7.3 and 8): a queue for
  the pool, policy narrowed, a month of alerts, every story crashed at
  every commit, a terminal client. Waits for a plan.
- **LLM accounts' later work** (`protocol/llm.md`, section 10): signing
  in through a client, API keys. Waits for a decision.
- **Versioning** (`README.md`, section 14): how jig is released and
  pinned, and how a change reaches every application. Waits for a
  decision.
- **The documents' open questions** (each document's last section):
  wait for a decision, or for a deployment's measurements.
- **The client plan's code debts** (`../development/tech-debt.md`,
  sections 3 to 7): legacy and compatibility paths, worlds' tests in bare
  numbers, 203 `unreachable!` arms, files of 1,600 to 3,000 lines. The
  client plan's jig 47 took up section 4. The rest waits for a
  consolidation pass.

## 4. Drafts

| Draft | About | Next |
|---|---|---|
| `draft/store.md` | jig's store on skein-kv in the engine: its families, keys and codecs, transcripts, secrets, opening and restart, retention and fill; six questions, each with a leaning | A design pass of its own, whose content moves into `protocol/store.md`; then a plan (the drafted jig-protocol-implementation's jig 34) |
