# Status

Updated 2026-10-16. How far jig's code is from its design, and which
plans take it there. The designer keeps this file (skein's
development.md, section 3.3).

## 1. Plans

| Plan | State | Takes the design to | Where |
|---|---|---|---|
| client-api | active on carbon-forge since 2026-10-16 | `STATUS.md`'s "jig's client API" (section 3), its six steps: the client crates renamed, with a check of the naming rule (`README.md`, 15); the engine's plumbing (`domain/`, the rows naming "(jig's client API)"); the wire collapsed; `jig-client`, `jig-client-domain`, `jig-client-protocol-http` and the reference UI, `jig-client-view`, on the public client (`client/domain/`); the server (`jig-protocol-engine-clients` with no listener, `jig-protocol-engine-clients-http`, `jig-test-service` mounting it at `/jig`); the browser (`protocol/browser.md`); in skein, the browser harness's other origins (skein's `ui-web.md`, 14.2) | `docs/development/plans/client-api/` of the repository downstream of jig, which it also changes |
| skein-ui | done 2026-10-15 | `STATUS.md`'s "jig on skein-ui", steps (1) and (2): `client/domain/view.md` 2, 3, 3.1, 7, 9, 10 and 15.3 (`jig-view` on `skein-ui`, jig's owner mapped onto its opaque owner, the provisional state gone, jig's outline rules gone, the chip's hole the reference hole, jig's constructors of skein-ui's limits, the text shell and the fake person skein's, with jig's settings); in skein, `ui.md`, `ui-text.md`, `ui-web.md`, `examples.md` 9, `testing.md` 6, `browser.md` 5, 6 and 9, `lib.md` 2 (the wasm32 build); jig's `client/domain/client.md` 12 (a sign-out's answer kept to its sign-in, in the client's joint world). jig's new primitives wait (the plan's `later.md`) | `docs/development/plans/skein-ui/` |
| client-composition | done 2026-10-14 | `client/domain/` README 2 and 5, `client.md` 3, 4.1, 5.1, 9, 14 and 17, `contract.md` 1, 3.3, 5.3, 6, 7.1 and 7.2, `pages.md` 2, 9 and 12, `root.md` 2, 6, 7, 9 and 15, `testing.md` 2, 4, 6, 8 and 9, `view.md` 3, 3.1, 4, 4.1, 5, 10 to 13 and 15; `README.md` 10 and 15; `examples.md` 8 | `docs/development/plans/client-composition/` |
| test-budgets | done 2026-10-14 | no section of jig's design: the focused suite back within its 15 s budget (`docs/development/workflow.md`, 2; skein's `testing-strategy.md`, 8), and skein's read of what has arrived in jig's readers of unframed bytes (skein's `lib.md`, 7) | `docs/development/plans/test-budgets/` |
| jig-protocol | done 2026-10-13 | `README.md` 15 (the crates' new names); `protocol/` but `llm.md`'s calls in a host's own process and `client.md` 12's browser; the `domain/` and `client/domain/` sections that section 2 names as owed to the protocol layer, but retention (`protocol/store.md`, 11.1) | `docs/development/plans/jig-protocol/` |
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
| `README.md` | partly | 8: the store; 9: agents on workers over the workers' channel, smith's payloads by hand-out; 10: the client's wire and the native shell (jig-protocol); 9: `smith-inline-agent` in the engine; 1, 2, 3, 10, 11, 12, 15 (jig's client API): the layers, one `jig-client` with no view, plumbing and sugar, the application's server, transport and sign-in, jig's credentials, the optional reference UI, every client crate under the `<project>-client` prefix (section 15): client-api; 14: versioning, open: not planned |
| `domain/README.md` | partly | 5: a task's proposal that nobody may accept, refused to its proposer (jig-protocol); 4, 5 (jig's client API): the names and decisions for the operations, templates, flags, credentials and the escalation holder: client-api |
| `domain/core.md` | partly | 6.1: a project's homes, named at adoption: not planned; 6.1: a project's resources, each adopted with its role: the core keeps only the roles live tasks' resources narrowed, and drops an owned one (`jig-domain-core/src/resources.rs`, `role`), so the project's document (`client/domain/pages.md`, 6.5) lists no owned resource and an adopted one only once a live task names it: not planned; 12: open questions only; 6.2, 7.3, 9, 11 (jig's client API): a goal as a tracked task, its defaults the goal template; shapes of work as templates, helpers, facts and models; the goal of a task the nearest tracked task at or above it: client-api |
| `domain/tasks.md` | partly | 5.5: a budget held for its reserve and one most costly completion; 8.2, 8.3: the start carries the inbox, a message refused or left unread stays and wakes the task again; 9: a task's proposal that nobody may accept, refused (jig-protocol); 1, 3, 4, 7, 9, 11, 12 (jig's client API): a task's label and flags (conversation, listed, retention class), last activity, waiting on, `AsAllowed`; a party's `Create` as an agent's batch, its reach, its root's escalation holder; tracked and priority amended, cancel with a grace, `Move`, the compound request; parties' `Propose` and `Withdraw`, and person tasks in their batches; a rejection reaching a party who proposed; goals per project, a grace's span: client-api |
| `domain/authority.md` | partly | 7, 8.3: a run placed only when its budget holds its reserve and one most costly completion; the reserve's fields are built; 6, 7: a project's current period and budget from the root, a party's pool from their role's allotment, a chat's authority from its person's role, where the core keeps one setting of each for the deployment (jig-protocol); 1, 6, 7, 8.1, 8.4, 8.5, 9 (jig's client API): one check for parties and agents, the escalation role, templates' overrides in the policy, `AsAllowed`, explicit proposals, a connector topic's subscription by a read grant: client-api |
| `domain/connectors.md` | partly | 10: the application's credentials a workspace needs (jig-protocol); 12: an adoption naming the project's home: not planned; 13: a system's request budget (fresh reads first, reserved shares, a stop until the reset): not planned; 1, 2, 5.3 to 5.5, 6.2, 6.3, 8 (jig's client API): procedures' amend, cancel with a grace and message, topics offered to agents, the party's own scope and windows, untracking: client-api; 6.3: a procedure's step's effects within its one decision: not planned (client-api's `later.md`) |
| `domain/engine.md` | partly | 6.1: stopping; 7.1: several charters, the start's messages, the conversation's first attempt; 7.2: windows; 8: grants pushed and resent, a hello's agent bounds checked; 9: no inbox section in the brief; 11: text by retry, a gap marked cut; 12: accounts on skein's driver; 12.1: an application's credentials (jig-protocol); 8: a run's text from workers' hosts, waiting for smith's channel: not planned. Models' windows and outputs and budgets' reserves are built; 1, 4.1, 4.4, 5.3, 5.4, 6, 7.2 to 7.4, 9, 11 to 11.5, 13 (jig's client API): `me`, task queries and task lines, waiting on and last activity, a conversation known by its flag, ended reads through the indexes, offers and effects as data, the compound request's worst case, the agents' tools naming revisions: client-api; 9: the brief's Lineage, Calls and Waiting sections: not planned (client-api's `later.md`) |
| `domain/hosts.md` | partly | 2, 6.1: the hub's hello with its agent's bounds, notices; 2, 6.2, 6.4: refusals carried with turns, the unread fence, windows; 6.5: activity from smith's facts; 6.8: smith's payloads by hand-out; 8: host tools decoded outside the root, the reserve (jig-protocol); 5.2: `smith-inline-agent` as the engine's agent; 6.5: text and named calls from agent processes: not planned; 1, 8, 9 (jig's client API): the host tools in the operations' vocabulary, `amend` and `release` naming revisions, `withdraw`, cancel with a grace, connectors' topics: client-api |
| `domain/people.md` | partly | 3: a reply naming its own sign-in; 5.1: a decision on an escalation no longer waiting, refused as not waiting; words within the run's bound (jig-protocol); 1 to 12 (jig's client API): credentials minted on the application's word (`Mint`), the operations with the compound request and operations per request, explicit proposing, the root's escalation holder, read positions kept for conversations only and ended ones in the store, the retention class, a chat as jig's template: client-api; 6: a task forgotten once read that its party's inbox no longer holds, by retention's pass: not planned (retention); 3: a service's credential reissued, a service removed: not planned |
| `domain/root.md` | partly | 5, 6: smith's payloads handed out to the protocol layer; 11, 12: host tools' calls, a call's key and a run's result decoded outside the root (jig-protocol) |
| `domain/testing.md` | partly | 7: the real store on skein-kv's in-memory mode in jig's worlds (jig-protocol); in `ops`'s worlds: not planned; 3 to 7, 9 (jig's client API): the stories and scenarios for the operations, fields and indexes: client-api |
| `client/domain/README.md` | partly | 2: the client's wire, link and native shell (jig-protocol); 1 to 6 (jig's client API): the client, `jig-client`, with no view, and the reference UI, `jig-client-view`, on it; the client's crates by the naming rule; the names and decisions: client-api; 6: several applications in one client, who may see what: not planned |
| `client/domain/client.md` | partly | 1 to 4, 6 to 8, 10 to 12, 14 to 20 (jig's client API): `jig-client`, from the client half of `jig-domain-client` and from `jig-protocol-client-link`: its surface, handles and keys with their askers, the request table held by the client, watches by handle and their latest documents, a run's text by handle, the small store of the client's own records, `SignIn { with }`, versions per owner, the helpers `shapes`, `facts`, `thread` and `words` (19), the modes (20): client-api; 20: the service and one-shot modes, built when an application needs them; 18: open questions: not planned |
| `client/domain/contract.md` | partly | 1 to 10 (jig's client API): the plumbing operations, the compound request, explicit proposing, templates, `me`, the task line, offers and a decision's effect as data, task queries, the application's scopes and windows, the typed version refusal, and what the engine's domain owes for them (section 9): client-api; 7.4: an application's streams: not planned; 5.3: `me`'s creating bounds for a `Create` of any executor: not planned (client-api's `later.md`) |
| `client/domain/objects.md` | partly | 1, 2, 4 to 7, 9 to 11 (jig's client API): jig's cards as the reference UI's, each act its operation, a refusal for authority offering a proposal, an application presenting jig's objects from their documents with `facts`: client-api; 13: open questions only |
| `client/domain/pages.md` | partly | 1 to 4, 6, 8, 9, 12 (jig's client API): the reference UI's pages on the public client: its places and their codec in `jig-client-view`, lists as task queries, chats and goals made by the helpers, compound requests, what each section watches and reads (12.5): client-api; 6.5: the policy section showing a project's template overrides: not planned (client-api's `later.md`); 7: a connector's request budget, and what its system last said: not planned |
| `client/domain/root.md` | partly | 1 to 12, 15 (jig's client API): the application's client root holding `jig-client` as a component, routing by asker, families as bytes, the request table the client's, the reference client root `jig-client-test-domain`: client-api; 11: the service mode; 15.1: an application's own child domains: not planned |
| `client/domain/testing.md` | partly | 1 to 9 (jig's client API): the client tested through its surface, with a scripted application; jig's check in `jig-client-test-kit` (6.2); the reference UI's worlds with skein's fake person; the browser tier, jig's stories for a browser shell and its attack stories (`jig-client-fake-person-browser`, `../../protocol/browser.md`, section 15): client-api |
| `client/domain/view.md` | partly | 1, 2, 4, 4.1, 5, 7, 9 to 11, 15, 18 (jig's client API): the reference UI, `jig-client-view`, from `jig-view` and the page half of `jig-domain-client`, on `jig-client` and `skein-ui` alone: its state, `jig_client_view::Ui` (18), its frame, sections, cards, confirmation, drafts and notices as primitives with a default arrangement, nothing in the tree mandated: client-api; 16: jig's own drawing kinds: not planned |
| `protocol/README.md` | partly | 5: the components but the charter's translation and turns' decoding; 7: versions; 8: the secrets' tables; 9: the store; 12: what the domain owes (jig-protocol); 1 to 8, 10 to 13 (jig's client API): the components by the naming rule (`jig-client-protocol-wire`, `jig-client-protocol-link`, `jig-client-protocol-http`; `jig-protocol-engine-clients` with no listener, `jig-protocol-engine-clients-http`), what an application writes (its server, transport, sign-in, presentation and templates), versions with the typed refusal: client-api |
| `protocol/channel.md` | no | All: the workers' wire, the engine's and the worker's sides, grants, its world (jig-protocol); 6.4: text from agent processes: not planned |
| `protocol/client.md` | partly | 1 to 17 (jig's client API): jig's messages, transport-agnostic; the clients' component's messages, `Mint` and `Link` among them, and no provider route; the link with no HTTP; the wire with its shape-specific records collapsed; the typed version refusal; the small store with the client's records only; the default HTTP mapping at both ends (17): client-api; 12, 17: a browser's client end over fetch, and wasm32 builds: client-api |
| `protocol/configuration.md` | no | All: jig's part of a configuration, profiles, secrets from private files, first start, operators' formats (jig-protocol); 1, 3, 3.1, 6 to 8, 10 (jig's client API): the `templates` member and policies' overrides, with their JSON, `clients` with no listener and no bundle, `providers` as the kinds identities name, operations per request checked at start: client-api |
| `protocol/browser.md` | no | 1 to 19 (jig's client API): jig's browser parts on the application's server: `jig-client-shell-web` (the client's adapter over skein's fetch, the printed link's sign-in), the example guards in `jig-protocol-engine-clients-http`, the reference UI's browser shell `jig-client-test-shell-web` with its page's policy, the reference UI's codec, jig's stories for a browser shell and its attack stories in `jig-client-fake-person-browser`, jig's browser tier and the client's wasm32 builds: client-api; 12.2: the Host check's certificate name between hosts: not planned (with the listener off loopback); 15: jig's tier calling skein's two-person stories for a browser shell: not planned (client-api's `later.md`) |
| `protocol/llm.md` | no | 5, 6: accounts on skein's driver, handed-in accounts, grants to workers (jig-protocol); 1–4, 7–9: LLM calls in a host's own process, for `smith-inline-agent`: not planned |
| `protocol/service.md` | no | All: the engine's and a worker's services, `iterate`, start and end, the reference service and binary (`jig-test-service`, `jig-test`), its world, one process with agents as child processes (jig-protocol); 1 to 3, 7, 8, 10 (jig's client API): the application's service owning the clients' listener and mounting the component, `jig-test-service` mounting it at `/jig` and serving its own bundle and policy, the printed link's URL printed on loopback: client-api; 7: the clients' listener off loopback, with its certificate and key files: not planned (it waits for skein's TLS server side); 10: a client in the engine's process, by an in-process transport: not planned (client-api's `later.md`) |
| `protocol/store.md` | no | All but retention: jig's store component on skein-kv, keys and intervals, codecs and the format row, commits, loads and parties' reads, transcripts' files, the secrets' store, fill, the real store in jig's worlds and the store's world; 15: what the domain owes it (jig-protocol); 1, 4.4, 7.2 to 7.4, 9, 15 (jig's client API): the ended indexes by requesting party and by project, ended read positions, task queries' reads, minted credentials' digests: client-api; 11.1: retention, by class, and its horizons: not planned |
| `examples.md` | partly | 4: `ops`'s agent, `smith-inline-agent` (its world scripts its runs' agents); 7.3: a queue for the pool, policy narrowed, a month of alerts (no world found, not confirmed); the conformance world crashes three stories, not all; 8: its error rate's drawing code, a scene; 1, 2, 8, 9.1 (jig's client API): `ops` on `jig-client` with the reference UI, `jig-ops-client-domain`, `jig-ops-client-view` and `jig-ops-client-protocol-wire`, its server mounting the component at `/jig`, its terminal on skein's text shell over the same client: client-api; 9.1: `ops` standalone: not planned |

## 3. Not planned

- **Retention** (`protocol/store.md`, 11.1; `domain/people.md`, section
  6): summaries of ended tasks, transcripts dropped past a horizon,
  answered keys expired, tasks forgotten once read erased, and their
  remains collected. The store's mechanisms are jig-protocol's; the
  domain's timers wait for a deployment whose store nears its budget.
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
- **smith's parity changes, for jig's next repin of smith** (smith's
  parity plan, its `later.md`; skein's development.md, 5.7): the tools
  domain's new `Call` and `Outcome` variants, for jobs, in
  `smith-domain-tools`, which jig's users of that crate follow. jig pins
  smith `29fa97c9`, before them. Waits for the plan that moves jig's pin
  of smith.
- **`ops` standalone** (`examples.md`, 9.1; `protocol/service.md`,
  section 7): its systems' APIs, its connectors' protocols, its service
  and its binary, and its terminal client's loop over a real terminal,
  skein's text shell with jig's settings (`examples.md`, section 8).
  Waits for the protocol layer, then a plan of its own.
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
- **jig's client API** (`README.md`, sections 1, 10, 11 and 15;
  `client/domain/`; `protocol/client.md`, `browser.md`, `service.md`,
  `configuration.md`, `store.md` and `README.md`; `domain/people.md`,
  `tasks.md`, `authority.md`, `core.md`, `engine.md`, `hosts.md` and
  `connectors.md`; `examples.md`, section 8; it supersedes six rows of
  skein-ui's `later.md`): jig's client presentation-agnostic, its API
  jig's primitives as plumbing with shapes of work as sugar, and the
  server and the URL space the application's. Designed 2026-10-15. In
  order, each step under jig's gate:
  1. *the crates renamed* by the client naming rule (`README.md`,
     section 15), a rename and a crate move with no change of
     behaviour: `jig-protocol-client-wire` to
     `jig-client-protocol-wire`, `jig-protocol-client-link` to
     `jig-client-protocol-link`, `jig-shell-native` to
     `jig-client-shell-native`, `jig-view` to `jig-client-view`,
     `jig-scripted-engine` to `jig-client-scripted-engine`,
     `jig-test-domain-client` to `jig-client-test-domain`,
     `jig-test-view` to `jig-client-test-view`, `jig-test-client-kit`
     to `jig-client-test-kit`, `jig-test-client-records` to
     `jig-client-test-records`, `jig-test-protocol-client-wire` to
     `jig-client-test-protocol-wire`; `ops`'s `jig-ops-domain-client` to
     `jig-ops-client-domain` and `jig-ops-view` to
     `jig-ops-client-view`. `jig-client-world`,
     `jig-client-protocol-world` and `jig-client-wire-world` keep their
     names; the engine's crates (`jig-protocol-engine-clients`,
     `jig-engine-clients-world`, `jig-fake-parties`) and
     `jig-protocol-transcript` are not client crates and keep theirs;
  2. *the engine's plumbing:* parties' `Create`, `Propose` and
     `Withdraw`, person tasks among their batches, cancel with a grace,
     `Move`, the compound request and operations per request; a task's
     label, flags, last activity and waiting on; `AsAllowed`; goals per
     project; the root's escalation holder; agents' `amend` and
     `release` naming revisions, `withdraw`, connectors' topics;
     procedures' amend, cancel and message; `me`, task queries, the two
     ended indexes and the ended read positions; templates in the
     configuration and policies' overrides; the engine sheds its
     shape-specific code;
  3. *the wire:* its shape-specific records collapsed into the
     operations, the compound request, `me` and the task line; the typed
     version refusal; the application's party scope and windows;
  4. *the client:* `jig-domain-client`'s client half becomes
     `jig-client-domain`, under `jig-client` with its helpers; the
     link's HTTP half becomes `jig-client-protocol-http`; the page half
     joins `jig-client-view`, the reference UI, on the public client
     alone; the client's small store keeps no place, address or draft;
  5. *the server:* `jig-protocol-engine-clients` owns no listener and
     takes `Mint`; its routes become `jig-protocol-engine-clients-http`,
     with the example guards; `jig-test-service` mounts it at `/jig` and
     serves its own bundle;
  6. *the browser,* last: `jig-client-shell-web`,
     `jig-client-test-shell-web`, jig's stories for a browser shell and
     its attack stories in `jig-client-fake-person-browser` (whose
     framed and cross-site stories serve their other origin's page
     themselves, and expect the policy's report as trouble, under
     skein's `ui-web.md`, 14.2: an increment in skein first if its
     harness cannot), jig's browser tier and the client's wasm32 builds.

  What the code does against the design, which these steps change: a
  conversation's and a goal's lines send their change time and runtime
  empty; conversations are ordered by number, ended goals by number; an
  ended chat is erased once read, so a list of closed chats shows only
  unread ones; a party's amendment proposal names the task as its
  proposer; the goal of a task is computed as the root in one document
  and as the nearest tracked ancestor in another (`domain/core.md`, 6.2,
  says the nearest tracked task at or above it); parties cannot create
  person tasks or withdraw their proposals; agents' `release` and
  `amend` name no revision; the client stores a home it never reads.
  Measured 2026-10-15: `jig-domain-client` about 10,000 lines of client
  work and 7,000 of presentation; the engine's shape-specific domain
  code about 3,500 lines (1,300 for chats only, 1,450 for goals'
  machinery that stays in general form, the rest for lists); 51 of the
  client wire's records go (41 shape-specific, 10 the small store's
  places, addresses and drafts). Planned: client-api (section 1).
- **Service and one-shot clients, and a bot relaying decisions**
  (`client/domain/client.md`, section 20; `client/domain/root.md`,
  section 11; `domain/people.md`, section 3): the client's service and
  one-shot modes, and a bot holding each person's own credential. Wait
  for an application whose services act as parties, or which pages a
  person on call.
- **The client's generic half in skein** (`README.md`, section 10;
  `protocol/README.md`, section 10): keyed requests sent again until
  durable, level-triggered watches, the link and generic guards, moved
  to skein by a rename and a crate move, as skein-ui was. Waits for a
  second user.
- **Several people on one deployment** (`client/domain/README.md`,
  sections 5 and 6): who may see what, a role's entries shared and taken.
  The client is built for one person first. Waits for such a deployment.
- **`ops`'s remaining stories** (`examples.md`, 7.3): a queue for the
  pool, policy narrowed, a month of alerts, every story crashed at every
  commit. Waits for a plan.
- **An application's own child domains** (`client/domain/root.md`,
  15.1): families that belong to no connector, their owners numbered as
  connectors are, routed by the engine's root to a child that is not a
  connector. Waits for an application with such a domain.
- **An application's streams** (`client/domain/contract.md`, 7.4): its
  data followed from an offset within a window, provisional, never
  committed state, in the pattern of a run's text. `jig-client` carries
  them as any watch. Waits for an application whose people follow its
  data as it comes.
- **jig's own drawing kinds** (`client/domain/view.md`, section 16): a
  chart every application shares, as jig's kind, with its data's encoding
  and its drawing code. Waits for a second application drawing the same
  kind.
- **What the browser's layer leaves** (`protocol/browser.md`, sections
  16 and 17; `protocol/client.md`, section 15): tabs of one person
  sharing one stream, and HTTP/2, which are jig's; what skein's UI kit
  leaves for later (skein's `ui.md`, 4.2 and section 18; skein's
  `ui-web.md`, section 15), among them the bundle served compressed,
  other browsers than Chromium, right-to-left and the file picker; and,
  not jig's, graphics beyond a scene, which an application draws in a
  component of its own (skein's `ui.md`, section 12). Each waits for the
  application that needs it.
- **Several applications in one client** (`client/domain/README.md`,
  section 6): one person's inbox across applications that share their
  people. Waits for such applications.
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

None. `client-api.md` became the design on 2026-10-15 ("jig's client
API", section 3).
