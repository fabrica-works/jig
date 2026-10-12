# Status

Updated 2026-10-12. How far jig's code is from its design, and which
plans take it there. The designer keeps this file (skein's
development.md, section 3.3).

## 1. Plans

| Plan | State | Takes the design to | Where |
|---|---|---|---|
| jig-protocol | drafting | `README.md` 15 (the crates' new names); `protocol/` but `llm.md`'s calls in a host's own process; the `domain/` and `client/domain/` sections that section 2 names as owed to the protocol layer | `docs/development/plans/jig-protocol/` (not written; `~/src/rust/plans/jig-protocol-implementation/`, unversioned, drafted before this design's revision of 2026-10-12, is an input) |
| jig-client-implementation | done 2026-10-11 | `client/domain/` above the protocol line; `client/domain/contract.md` 9 (the engine's changes); `examples.md` 8 (`ops`'s client) | `~/src/rust/plans/jig-client-implementation/` (not versioned) |
| jig-protocol-design | done 2026-10-10 | a design pass, with no code: `protocol/`, `draft/store.md` | `~/src/rust/plans/jig-protocol-design/` (not versioned) |
| jig-move | done 2026-10-09 | no section: jig in a repository of its own, with its workspace, lints and gate | `~/src/rust/plans/jig-move/` (not versioned) |
| jig-extraction | done 2026-10-09 | `domain/`: the core and its children, the hub, the conformance world; `README.md` 6.2 (the journal, in skein-lib); `examples.md` 2–6, 9, 10 and two stories of 7 (`ops`) | `~/src/rust/plans/jig-extraction/` (not versioned) |

jig-protocol is to be planned from this design, as skein's
development.md, section 5 says. Its upstream needs are on skein's and
smith's `main`: skein's designs of joined sockets, in-process addresses,
private startup reads, `base64`, `Secret`, `CryptoRng` and the
cryptographic seed, and `OpenWrite` and `Cut` (designed, not built);
smith's charter v2, transcript v3, channel v2, host domain and host
half, parent face, `smith-inline-agent` and agent shell (built). Section
2 says "not planned" for what it would build.

## 2. The design

| Document | Built | Left |
|---|---|---|
| `README.md` | partly | 8: the store (`protocol/store.md`); 9: agents on workers over the workers' channel, smith's payloads by hand-out, `smith-inline-agent` in the engine; 10: the client's wire and shells; 15: the crates' new names; 14: versioning, open: not planned |
| `domain/README.md` | partly | 5: a task's proposal that nobody may accept, refused to its proposer (a party's is): not planned |
| `domain/core.md` | yes | 12: open questions only |
| `domain/tasks.md` | partly | 5.5: a budget held for its reserve and one most costly completion; 8.2, 8.3: the start carries the inbox, a message refused or left unread stays and wakes the task again; 9: a task's proposal that nobody may accept, refused rather than kept waiting: not planned |
| `domain/authority.md` | partly | 7, 8.3: a run placed only when its budget holds its reserve and one most costly completion (the reserve's fields are built): not planned |
| `domain/connectors.md` | partly | 10: the application's credentials a workspace needs, named with its items; 13: a system's request budget (fresh reads first, reserved shares, a stop until the reset): not planned |
| `domain/engine.md` | partly | 6.1: stopping; 7.1: several charters, the start's messages, the conversation's first attempt for the run's identity; 7.2: windows, a resume from the current window; 8: grants pushed, and resent at a hello and an adoption; a hello's agent bounds checked; a run's text from hosts through the fleet; 9: no inbox section in the brief; 11: text by retry, a gap marked cut; 12: accounts on skein's driver; 12.1: an application's credentials: not planned. Models' windows and outputs and budgets' reserves are built |
| `domain/hosts.md` | partly | 2, 6.1: the hub's hello (with its agent's bounds) and notices; 2, 6.2, 6.4: refusals carried with turns, the unread fence, windows; 5.2: `smith-inline-agent` as the engine's agent; 6.5: activity from smith's facts; 6.8: smith's payloads by hand-out; 8: host tools decoded outside the root, the reserve: not planned |
| `domain/people.md` | partly | 3: a reply naming its own sign-in; 5.1: a decision on an escalation no longer waiting, refused as not waiting; 6: a chat forgotten once ended and read: not planned |
| `domain/root.md` | partly | 5, 6: smith's payloads handed out to the protocol layer; 11, 12: the reference root decodes host tools' calls, a call's key and a run's result itself: not planned |
| `domain/testing.md` | partly | 7: the real store on skein-kv's in-memory mode in the worlds (`jig-fake-store` stands in): not planned |
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
| `protocol/llm.md` | no | 5, 6: accounts on skein's driver, grants to workers: not planned; 1–4, 7–9: LLM calls in a host's own process, for `smith-inline-agent`: not planned |
| `protocol/service.md` | no | All: the engine's and a worker's services, `iterate`, start and end, the reference service and binary (`jig-test-service`, `jig-test`), its world, one process with agents as child processes: not planned |
| `protocol/store.md` | no | All: jig's store component on skein-kv, keys and intervals, codecs and the format row, commits, loads and parties' reads, transcripts' files, the secrets' store, growth, the real store in the worlds and the store's world; 15: what the domain owes it; skein's `OpenWrite` and `Cut`: not planned |
| `examples.md` | partly | 4: `ops`'s agent, `smith-inline-agent` (its world scripts its runs' agents); 7.3: a queue for the pool, policy narrowed, a month of alerts (no world found, not confirmed); the conformance world crashes three stories, not all; 8: a terminal client; 9.1: `ops` standalone: not planned |

## 3. Not planned

- **jig's protocol layer** (`protocol/README.md`, section 5; `README.md`,
  section 15): the components, the reference service and binary, the
  protocol worlds, and the crates' new names. Waits for its plan,
  jig-protocol (section 1).
- **What the domain owes the protocol layer** (`protocol/README.md`,
  section 12): the engine stopping, several charters, grants pushed and
  resent, an application's credentials, text by retry, accounts on
  skein's driver, the hub's hello with its agent's bounds, refusals
  carried with turns and the unread fence, the start's messages, windows,
  the reserve before a run is placed, smith's payloads and facts by
  hand-out, host tools outside the root, people's and tasks' refusals,
  signing out (section 2's rows); and the store's: live before history in
  each child's keys, indexes for parties' reads, the inbox restored from
  an index, turns kept by window and dropped whole, fill, retention, keys
  for single reads (`protocol/store.md`, section 15). Waits for
  jig-protocol.
- **Agents in the engine** (`README.md`, section 9, the second shape;
  `domain/hosts.md`, 5.2; `protocol/llm.md`, sections 1 to 4 and 7 to
  9): `smith-inline-agent` composed by the engine's root, and smith's LLM
  component in its protocol layer. `ops`'s world scripts its runs' agents
  meanwhile. Waits for an application that runs agents in its engine.
- **What jig asks of smith, later** (`protocol/README.md`, section 11;
  `domain/hosts.md`, 6.5; `protocol/channel.md`, 6.4 and section 13): a
  run's live text and named calls from agent processes, whose channel
  carries facts without content, so they give neither (wanted for the
  chat web interface); and an abort and a closed terminal for smith's
  LLM component. Waits for a smith design and plan.
- **`ops` standalone** (`examples.md`, 9.1; `protocol/service.md`,
  section 7): its systems' APIs, its connectors' protocols, its service
  and its binary. Waits for the protocol layer, then a plan of its own.
- **An application's own binary and real loop** (`protocol/service.md`,
  sections 3 and 8): jig's reference binary, `jig-test`, is
  jig-protocol's; an application's waits for that application.
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

None.
