# Status

Updated 2026-10-15. How far jig's code is from its design, and which
plans take it there. The designer keeps this file (skein's
development.md, section 3.3).

## 1. Plans

| Plan | State | Takes the design to | Where |
|---|---|---|---|
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
| `README.md` | partly | 8: the store; 9: agents on workers over the workers' channel, smith's payloads by hand-out; 10: the client's wire and the native shell (jig-protocol); 10, 15: jig's pages as a library, owners, the text shell (client-composition); 9: `smith-inline-agent` in the engine; 10, 11, 15: jig on skein-ui: jig's view as primitives on `skein-ui` with jig's default arrangement, nothing in the tree mandated, a decision's facts and jig's check; skein's `skein-ui-text`, `skein-fake-person` and `skein-ui-bundle` in place of `jig-shell-text`, `jig-fake-person` and `jig-bundle`; `jig-shell-web` as jig's sign-in part, link adapter and look over `skein-ui-web`; the reference shell, an application's browser shell, its paths and its steps past the reference, the client host, jig's stories for a browser shell (`protocol/browser.md`); 14: versioning, open: not planned |
| `domain/README.md` | partly | 5: a task's proposal that nobody may accept, refused to its proposer (jig-protocol) |
| `domain/core.md` | partly | 6.1: a project's homes, named at adoption: not planned; 6.1: a project's resources, each adopted with its role: the core keeps only the roles live tasks' resources narrowed, and drops an owned one (`jig-domain-core/src/resources.rs`, `role`), so the project's document (`client/domain/pages.md`, 6.5) lists no owned resource and an adopted one only once a live task names it: not planned; 12: open questions only |
| `domain/tasks.md` | partly | 5.5: a budget held for its reserve and one most costly completion; 8.2, 8.3: the start carries the inbox, a message refused or left unread stays and wakes the task again; 9: a task's proposal that nobody may accept, refused (jig-protocol) |
| `domain/authority.md` | partly | 7, 8.3: a run placed only when its budget holds its reserve and one most costly completion; the reserve's fields are built; 6, 7: a project's current period and budget from the root, a party's pool from their role's allotment, a chat's authority from its person's role, where the core keeps one setting of each for the deployment (jig-protocol) |
| `domain/connectors.md` | partly | 10: the application's credentials a workspace needs (jig-protocol); 12: an adoption naming the project's home: not planned; 13: a system's request budget (fresh reads first, reserved shares, a stop until the reset): not planned |
| `domain/engine.md` | partly | 6.1: stopping; 7.1: several charters, the start's messages, the conversation's first attempt; 7.2: windows; 8: grants pushed and resent, a hello's agent bounds checked; 9: no inbox section in the brief; 11: text by retry, a gap marked cut; 12: accounts on skein's driver; 12.1: an application's credentials (jig-protocol); 8: a run's text from workers' hosts, waiting for smith's channel: not planned. Models' windows and outputs and budgets' reserves are built |
| `domain/hosts.md` | partly | 2, 6.1: the hub's hello with its agent's bounds, notices; 2, 6.2, 6.4: refusals carried with turns, the unread fence, windows; 6.5: activity from smith's facts; 6.8: smith's payloads by hand-out; 8: host tools decoded outside the root, the reserve (jig-protocol); 5.2: `smith-inline-agent` as the engine's agent; 6.5: text and named calls from agent processes: not planned |
| `domain/people.md` | partly | 3: a reply naming its own sign-in; 5.1: a decision on an escalation no longer waiting, refused as not waiting; words within the run's bound (jig-protocol); 6: a chat forgotten once ended and read: not planned (retention); 3: a service's credential reissued, a service removed: not planned |
| `domain/root.md` | partly | 5, 6: smith's payloads handed out to the protocol layer; 11, 12: host tools' calls, a call's key and a run's result decoded outside the root (jig-protocol) |
| `domain/testing.md` | partly | 7: the real store on skein-kv's in-memory mode in jig's worlds (jig-protocol); in `ops`'s worlds: not planned |
| `client/domain/README.md` | partly | 2: the client's wire, link and native shell (jig-protocol); 2, 5: jig's pages as a library, layout as semantics, the closed sets, owners, the text shell (client-composition); 2, 4, 5: jig on skein-ui: `jig-view` on `skein-ui`, jig's primitives and default arrangement, nothing in the tree mandated, the promise held below the tree, a decision's facts and jig's check, jig's owner on skein-ui's; skein's text shell, fake person and bundler in place of jig's crates; jig's browser parts beside skein's, an application's browser shell and its steps past the reference, drawings as scenes or an application's components (`../../protocol/browser.md`); 6: several applications in one client: not planned |
| `client/domain/client.md` | partly | 12: signing out from the frame; 16: the protocol layer below the domain (jig-protocol); 3, 4.1, 5.1, 14: a page's sections, the arrangement, the sections an application's page names, limits counted for them; 9: folds the domain keeps; 17: the arranged and the section-slot stories (client-composition); 11, 16: a tab's session storage, addresses as a browser's paths (`../../protocol/browser.md`); 8.2: jig on skein-ui: the link's state shown by jig's link part where an application places it, on every page of jig's default arrangement: not planned |
| `client/domain/contract.md` | partly | 5.1: a turn's bytes in its activity line; 6: reads refused as signed out; 8: a range of versions served; 10: the wire, one connection per client (jig-protocol); 1, 3.3, 5.3, 6, 7.1, 7.2: owners in parts, requests, reads and subjects (client-composition); 7.4: an application's streams; 10: the browser's client served by the engine: not planned |
| `client/domain/objects.md` | partly | 2: jig on skein-ui: jig's cards as defaults, an application presenting jig's objects itself from jig's documents through jig's binding constructors, a decision's facts as data and checked by jig's check: not planned; 13: open questions only |
| `client/domain/pages.md` | partly | 2, 9, 12: the frame's parts, the sections, their region names, arrangements, an application's arrangement and jig's sections on its pages (client-composition); 2: home, the place the application's codec gives the path `/` in a browser; 1, 2, 9, 12: jig on skein-ui: the frame's parts placed anywhere or nowhere, jig's banner and arrangements as jig's default arrangement, `after_page` split into the owed cards, the confirmation and the notices, no outline rule of jig's; 7: a connector's request budget, and what its system last said: not planned |
| `client/domain/root.md` | partly | 2, 6, 7, 9, 15: owners, numbered alike; the section slot (client-composition); 11: a service's root, its small store in a file, its world; 5, 9, 12: addresses printed and parsed as paths by the client host's codec, jig's default for jig's places; the look as data; the browser shell copied from the reference shell; 7, 9, 15: jig on skein-ui: a family presenting one of jig's objects itself, bound through jig's binding constructors; the view root's tree in the arrangement it chooses, from jig's primitives; jig's owner mapped onto skein-ui's opaque owner, 65536 for jig; 15.1: an application's own child domains: not planned |
| `client/domain/testing.md` | partly | 2: the protocol world, the real loop (jig-protocol); 2, 4, 6, 8, 9: the page's outline, and a delivery taken told unless the engine restarted, in the referee; the text shell's tiers and stories, the section slot in jig's test application (client-composition); 2, 3, 4, 6, 8: jig on skein-ui: skein's fake person and text shell in place of jig's; the referee running skein-ui's outline check and jig's check, jig's outline rules gone; the link is told where the application shows jig's link part or its own presentation of the link's state; an application presenting jig's objects itself running jig's check in its worlds; 1, 2, 3, 4, 8, 9: the browser tier on skein's face and harness, jig's stories for a browser shell, jig's reference shell taking steps past it, an application's browser tier, the tier's budget: not planned |
| `client/domain/view.md` | partly | 11: the native shell (jig-protocol); 3, 3.1, 4, 4.1, 5, 10–13, 15: the outline and its check, landmarks, menus, disclosures, images, region names, shortcuts, the drawing act's place, owners, the parts and sections as functions, the text shell (client-composition); 1–10, 13, 15, 17: jig on skein-ui: `jig-view` on `skein-ui`, whose generic core (the tree, the builder, the diff, the view, Markdown, the phrasebook's machinery, scenes, the outline's rules, limits and their worst case) is skein's; jig's owner mapped onto skein-ui's opaque owner; the provisional state gone, provisional text said in a status; jig's outline rules gone; the frame's parts, the owed cards, the confirmation and the notices as primitives, jig's banner and `after_page` its defaults; an application presenting jig's objects itself, `jig_view::facts` and `jig_view::check`; the chip's hole as skein-ui's reference hole; jig's constructors of skein-ui's limits; the text shell skein's, with jig's settings; 5: a link's address for a shell, printed by the client host's codec; 7: the shells' own words; 11, 12, 13: skein's browser parts beside jig's, holding the order that protects requests whatever the order of the application's loop, drawing code in every shell that draws, an application's elements and components, the look's names as data, jig's stylesheet; 16: jig's own drawing kinds: not planned |
| `protocol/README.md` | partly | 5: the components but the charter's translation and turns' decoding; 7: versions; 8: the secrets' tables; 9: the store; 12: what the domain owes (jig-protocol); 2, 5, 6, 10, 11, 13: jig's browser parts beside skein's, the reference shell, jig's default codec, the engine's serving as data, what an application writes for its browser and what it may change, what it needs from skein (its UI kit, built for wasm32 too) and smith: not planned |
| `protocol/channel.md` | no | All: the workers' wire, the engine's and the worker's sides, grants, its world (jig-protocol); 6.4: text from agent processes: not planned |
| `protocol/client.md` | no | All but section 12's browser: the client's wire, the engine's and the client's sides, sign-ins, the native shell, its world (jig-protocol); 8, 10, 12, 14, 15, 16: a browser's sign-in, its session storage, an application's browser shell over skein's browser parts and jig's, jig's link adapter, wasm32 builds, the engine's serving as data behind its front door, the bundle's digest directory served immutable, the browser tier on skein's harness, a hidden tab's stream (`browser.md`; skein's `ui-web.md`): not planned |
| `protocol/configuration.md` | no | All: jig's part of a configuration, profiles, secrets from private files, first start, operators' formats (jig-protocol); 3: the clients' bundle directory, which the reference startup reads (`browser.md`, section 12): not planned |
| `protocol/browser.md` | no | 1–19: jig's browser layer, skein's browser parts (skein's `ui-web.md`) and what jig adds to them: `jig-shell-web`'s sign-in part (the cookie, the printed link, providers, signing out, a credential shown once), the link's adapter (jig's link requests as the transport's plain data, with the browser header; fetch's reports as link events) and jig's look (`jig.css`, jig's palette, its pictures, its phrases for the parts' words); the reference shell, `jig-test-shell-web`, after skein's, with jig's link, sign-in and client host; what an application writes and may change, step by step; the bundle under `/jig/client/`, its digest directory served immutable and its page revalidated; jig's default codec for jig's places, `/jig/` reserved, history; the engine's serving as data behind its front door, the policy's floor and additions; Escape on jig's confirmation's cancel button; jig's stories for a browser shell on skein's face and harness, jig's browser tier and its wasm32 builds; what stays fixed, and why: not planned |
| `protocol/llm.md` | no | 5, 6: accounts on skein's driver, handed-in accounts, grants to workers (jig-protocol); 1–4, 7–9: LLM calls in a host's own process, for `smith-inline-agent`: not planned |
| `protocol/service.md` | no | All: the engine's and a worker's services, `iterate`, start and end, the reference service and binary (`jig-test-service`, `jig-test`), its world, one process with agents as child processes (jig-protocol); 7, 8: `jig-test`'s startup giving the engine jig's test bundle, built from the reference shell, the browser tier: not planned |
| `protocol/store.md` | no | All but retention: jig's store component on skein-kv, keys and intervals, codecs and the format row, commits, loads and parties' reads, transcripts' files, the secrets' store, fill, the real store in jig's worlds and the store's world; 15: what the domain owes it (jig-protocol); 11.1: retention: not planned |
| `examples.md` | partly | 8: `ops`'s arrangement, its section slot, its owners in its view, its terminal client in its worlds (client-composition); 4: `ops`'s agent, `smith-inline-agent` (its world scripts its runs' agents); 7.3: a queue for the pool, policy narrowed, a month of alerts (no world found, not confirmed); the conformance world crashes three stories, not all; 8: its terminal over a real terminal; 8: its error rate's drawing code, a scene; 8: jig on skein-ui: its terminal client on skein's text shell, its worlds' person skein's fake person, its browser shell over skein's browser parts and jig's; 9.1: `ops` standalone, its browser shell with it: not planned |

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
- **A service's client root** (`client/domain/root.md`, section 11):
  waits for a service that acts as a party.
- **The browser's layer** (`protocol/browser.md`, sections 1 to 19;
  `protocol/client.md`, sections 8, 10, 12, 14 and 15;
  `client/domain/view.md`, sections 1, 11 and 13;
  `client/domain/root.md`, sections 5, 9 and 12;
  `client/domain/testing.md`, the browser tier and jig's stories for a
  browser shell; skein's `lib.md`, section 2, and `testing.md`, section
  6): skein's browser parts (skein's `ui-web.md`) and what jig adds to
  them, `jig-shell-web`'s sign-in part, the link's adapter and jig's
  look; the reference shell, `jig-test-shell-web`, written after
  skein's and adding jig's link, sign-in and client host, which an
  application copies to own its start and its loop; the engine's
  serving as data the application's startup gives, the bundle under
  `/jig/client/` with its digest directory immutable, behind a fixed
  front door and the policy's floor; signing in by cookie; jig's
  default codec for jig's places; jig's browser tier and the client's
  wasm32 builds, jig's stories for a browser shell on skein's face and
  harness, and the reference shell taking jig's steps an application
  may take past it.
  Designed 2026-10-14, on skein's UI kit 2026-10-15. Waits for skein's
  UI kit, then the last stage of jig on skein-ui (below).
- **jig on skein-ui** (`README.md`, sections 10, 11 and 15;
  `protocol/README.md`, sections 5, 10 and 13; `protocol/browser.md`;
  `protocol/client.md`, section 12; `client/domain/README.md`, sections
  2, 4 and 5; `client/domain/view.md`; `client/domain/pages.md`,
  sections 1, 2, 9 and 12; `client/domain/objects.md`, section 2;
  `client/domain/client.md`, 8.2; `client/domain/testing.md`, sections
  2, 3, 4, 6 and 8; `client/domain/root.md`, sections 7, 9 and 15;
  `examples.md`, section 8; skein's `ui.md`, `ui-text.md` and
  `ui-web.md`; skein's `STATUS.md`, **skein-ui**): `jig-view` on
  `skein-ui`, keeping jig's primitives, jig's default arrangement,
  phrases, region names, tokens, bindings, limits' constructors and
  owner mapping; nothing in the tree mandated, jig's outline rules gone,
  `jig_view::facts` and `jig_view::check` in their place; skein's
  `skein-ui-text`, `skein-fake-person` and `skein-ui-bundle` in place of
  `jig-shell-text`, `jig-fake-person` and `jig-bundle`; `jig-shell-web`
  as jig's sign-in part, link adapter and look over `skein-ui-web`. The
  user's rule for its plan: refactor jig's code in place until the
  generic parts can be extracted by a mechanical rename and crate move,
  never rewrite them in skein. In order: (1) generalize in jig, under
  jig's gate: the owner made opaque with jig's mapping; the provisional
  state dropped; `Value.written` and the view's count named generically;
  `binding.rs` split (`Act`, `Binding`, `BoundAct`, `number` apart from
  jig's meanings); `limits.rs` split (the struct and `worst_case` apart
  from jig's constructors and jig's page scratch); `outline.rs` split
  (the generic rules apart from jig's, which then go); `phrases.rs`
  apart from jig's tables, its chip hole the reference hole;
  `jig-shell-text`'s `JigRegion::Link` a setting, its output byte for
  byte as before; the roles jig's view binds checked against skein's
  `ui.md`, section 6's act table, before skein-ui's builder asserts it;
  (2) move as is, a
  rename and a crate move: `tree`, `builder`, `diff`, `render`,
  `markdown`, the generic halves above, with their tests, into
  `skein-ui`; `jig-shell-text` into `skein-ui-text`; `jig-fake-person`
  into `skein-fake-person`; skein lands first, then jig repins; (3) new,
  in skein: the vocabulary added now, the act refusal, scenes, the
  browser parts, the bundler, the face and harness, the UI example and
  its browser tier, the gate's wasm32 builds; then in jig: jig's frame
  and `after_page` primitives, `jig_view::facts` and `check`,
  `jig-shell-web`'s sign-in part and link adapter, `jig-test-shell-web`,
  jig's browser tier. Measured 2026-10-15: jig-view's generic core about
  2,700 production lines (tree 405, builder 447, diff 352, render 96,
  markdown 506, phrases 597, and parts of binding, limits and outline);
  `jig-shell-text` 992 lines; `jig-fake-person` 645; their tests about
  2,400. Two decisions wait for the user: a JavaScript snippet of at
  most 10 lines in skein's UI example, called by its component (default
  yes: it proves the bundler and the policy carry an application's
  JavaScript), and with it whether jig's test application carries one
  too (default no, `protocol/browser.md`, section 17); and the native
  shell split into parts (default later: `jig-shell-native` stays one
  crate, since it carries jig's link, not the tree). Waits for its plan,
  across skein and jig, which comes next.
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
  committed state. skein's browser parts and jig's link adapter carry
  them as any watch, with no change of their own (`protocol/browser.md`,
  section 16). Waits for an application whose people follow its data as
  it comes.
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

| Draft | About | Next |
|---|---|---|
