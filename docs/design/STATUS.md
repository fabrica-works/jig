# Status

Updated 2026-10-12. How far jig's code is from its design, and which
plans take it there. The designer keeps this file (skein's
development.md, section 3.3).

## 1. Plans

| Plan | State | Takes the design to | Where |
|---|---|---|---|
| jig-protocol | active on carbon-forge since 2026-10-12 | `README.md` 15 (the crates' new names); `protocol/` but `llm.md`'s calls in a host's own process and `client.md` 12's browser; the `domain/` and `client/domain/` sections that section 2 names as owed to the protocol layer, but retention (`protocol/store.md`, 11.1) | `docs/development/plans/jig-protocol/` |
| jig-client-implementation | done 2026-10-11 | `client/domain/` above the protocol line; `client/domain/contract.md` 9 (the engine's changes); `examples.md` 8 (`ops`'s client) | `~/src/rust/plans/jig-client-implementation/` (not versioned) |
| jig-protocol-design | done 2026-10-10 | a design pass, with no code: `protocol/`, `draft/store.md` | `~/src/rust/plans/jig-protocol-design/` (not versioned) |
| jig-move | done 2026-10-09 | no section: jig in a repository of its own, with its workspace, lints and gate | `~/src/rust/plans/jig-move/` (not versioned) |
| jig-extraction | done 2026-10-09 | `domain/`: the core and its children, the hub, the conformance world; `README.md` 6.2 (the journal, in skein-lib); `examples.md` 2–6, 9, 10 and two stories of 7 (`ops`) | `~/src/rust/plans/jig-extraction/` (not versioned) |

jig-protocol builds skein's designed pieces it needs (in-process
addresses and joined sockets, private startup reads and the seed,
`Secret`, `base64`, `CryptoRng`, `OpenWrite` and `Cut`), takes them into
smith, and builds what section 2's rows name it for.

## 2. The design

| Document | Built | Left |
|---|---|---|
| `README.md` | partly | 8: the store; 9: agents on workers over the workers' channel, smith's payloads by hand-out; 10: the client's wire and the native shell (jig-protocol); 9: `smith-inline-agent` in the engine; 10: the browser's shell; 14: versioning, open: not planned |
| `domain/README.md` | partly | 5: a task's proposal that nobody may accept, refused to its proposer (jig-protocol) |
| `domain/core.md` | partly | 6.1: a project's homes, named at adoption: not planned; 12: open questions only |
| `domain/tasks.md` | partly | 5.5: a budget held for its reserve and one most costly completion; 8.2, 8.3: the start carries the inbox, a message refused or left unread stays and wakes the task again; 9: a task's proposal that nobody may accept, refused (jig-protocol) |
| `domain/authority.md` | partly | 7, 8.3: a run placed only when its budget holds its reserve and one most costly completion; the reserve's fields are built; 6, 7: a project's current period and budget from the root, a party's pool from their role's allotment, a chat's authority from its person's role, where the core keeps one setting of each for the deployment (jig-protocol) |
| `domain/connectors.md` | partly | 10: the application's credentials a workspace needs (jig-protocol); 12: an adoption naming the project's home: not planned; 13: a system's request budget (fresh reads first, reserved shares, a stop until the reset): not planned |
| `domain/engine.md` | partly | 6.1: stopping; 7.1: several charters, the start's messages, the conversation's first attempt; 7.2: windows; 8: grants pushed and resent, a hello's agent bounds checked; 9: no inbox section in the brief; 11: text by retry, a gap marked cut; 12: accounts on skein's driver; 12.1: an application's credentials (jig-protocol); 8: a run's text from workers' hosts, waiting for smith's channel: not planned. Models' windows and outputs and budgets' reserves are built |
| `domain/hosts.md` | partly | 2, 6.1: the hub's hello with its agent's bounds, notices; 2, 6.2, 6.4: refusals carried with turns, the unread fence, windows; 6.5: activity from smith's facts; 6.8: smith's payloads by hand-out; 8: host tools decoded outside the root, the reserve (jig-protocol); 5.2: `smith-inline-agent` as the engine's agent; 6.5: text and named calls from agent processes: not planned |
| `domain/people.md` | partly | 3: a reply naming its own sign-in; 5.1: a decision on an escalation no longer waiting, refused as not waiting; words within the run's bound (jig-protocol); 6: a chat forgotten once ended and read: not planned (retention); 3: a service's credential reissued, a service removed: not planned |
| `domain/root.md` | partly | 5, 6: smith's payloads handed out to the protocol layer; 11, 12: host tools' calls, a call's key and a run's result decoded outside the root (jig-protocol) |
| `domain/testing.md` | partly | 7: the real store on skein-kv's in-memory mode in jig's worlds (jig-protocol); in `ops`'s worlds: not planned |
| `client/domain/README.md` | partly | 2: the client's wire, link and native shell (jig-protocol); the browser's shell: not planned |
| `client/domain/client.md` | partly | 12: signing out from the frame; 16: the protocol layer below the domain (jig-protocol) |
| `client/domain/contract.md` | partly | 5.1: a turn's bytes in its activity line; 6: reads refused as signed out; 8: a range of versions served; 10: the wire, one connection per client (jig-protocol); 10: the browser's client served by the engine: not planned |
| `client/domain/objects.md` | yes | 13: open questions only |
| `client/domain/pages.md` | partly | 7: a connector's request budget, and what its system last said: not planned |
| `client/domain/root.md` | partly | 11: a service's root, its small store in a file, its world: not planned |
| `client/domain/testing.md` | partly | 2: the protocol world, the real loop (jig-protocol); 2, 3: the browser tier, the fake person's browser face: not planned |
| `client/domain/view.md` | partly | 11: the native shell (jig-protocol); 11–13: the browser's shell, drawings in a shell, roles mapped by the browser, its stylesheet: not planned |
| `protocol/README.md` | partly | 5: the components but the charter's translation and turns' decoding; 7: versions; 8: the secrets' tables; 9: the store; 12: what the domain owes (jig-protocol) |
| `protocol/channel.md` | no | All: the workers' wire, the engine's and the worker's sides, grants, its world (jig-protocol); 6.4: text from agent processes: not planned |
| `protocol/client.md` | no | All but section 12's browser: the client's wire, the engine's and the client's sides, sign-ins, the native shell, its world (jig-protocol); 12: the browser's shell, wasm32 builds, the browser tier: not planned |
| `protocol/configuration.md` | no | All: jig's part of a configuration, profiles, secrets from private files, first start, operators' formats (jig-protocol) |
| `protocol/llm.md` | no | 5, 6: accounts on skein's driver, handed-in accounts, grants to workers (jig-protocol); 1–4, 7–9: LLM calls in a host's own process, for `smith-inline-agent`: not planned |
| `protocol/service.md` | no | All: the engine's and a worker's services, `iterate`, start and end, the reference service and binary (`jig-test-service`, `jig-test`), its world, one process with agents as child processes (jig-protocol) |
| `protocol/store.md` | no | All but retention: jig's store component on skein-kv, keys and intervals, codecs and the format row, commits, loads and parties' reads, transcripts' files, the secrets' store, fill, the real store in jig's worlds and the store's world; 15: what the domain owes it (jig-protocol); 11.1: retention: not planned |
| `examples.md` | partly | 4: `ops`'s agent, `smith-inline-agent` (its world scripts its runs' agents); 7.3: a queue for the pool, policy narrowed, a month of alerts (no world found, not confirmed); the conformance world crashes three stories, not all; 8: a terminal client; 9.1: `ops` standalone: not planned |

## 3. Not planned

- **Retention** (`protocol/store.md`, 11.1; `domain/people.md`, section
  6): summaries of ended tasks, transcripts dropped past a horizon,
  answered keys expired, chats forgotten, and forgotten chats' remains
  collected. The store's mechanisms are jig-protocol's; the domain's
  timers wait for a deployment whose store nears its budget.
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
- **The browser's shell and the client's wasm32 builds**
  (`protocol/client.md`, section 12; `client/domain/view.md`, sections 11
  to 13; `client/domain/testing.md`, the browser tier; skein's `lib.md`,
  section 2): jig-protocol proves the client's protocol through the native
  shell only, by the user's choice (2026-10-12). Waits for a plan of its
  own, once the native client serves.
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
