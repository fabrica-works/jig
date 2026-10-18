# jig's design

Provisional, 2026-10-06, revised 2026-10-18. What jig is, how an
application is built around it, and what keeps the application's wiring
from breaking what jig promises. This document comes first. The documents
it lists in section 12 go deeper into each part. The mechanics are those
of skein's `programming-model.md`. What is still open is listed in
section 14.

## 1. In one page

- **A kit for agentic applications.** jig runs work done by agents,
  procedures and people, keeps that work in its own store, and drives
  external systems through connectors. It aims to be as flexible as a
  coding agent at a terminal, and adds what such an agent lacks:
  - work that outlives processes and machines;
  - many agents at once;
  - rules no agent can loosen;
  - effects on other systems made once.
- **Five primitives** (`domain/core.md`):
  - the **task**: what is asked;
  - the **executor**: who does it, which is an agent, a procedure or a
    person;
  - the **message**: everything a task hears once it has started;
  - **authority**: what a task may do, narrowed whenever it delegates;
  - the **connector**: an external system.

  Everything else is built from them: plans, coordinators, chats, goals,
  reviews.
- **The application owns the root.** skein's model nests step machines
  inside one another, and the top one is always the application's. The
  application's root domain composes jig's core with the application's
  connectors and its own domain logic, and translates between them
  (programming-model.md, 4.5). Extension is composition: no traits, no
  generic items, no callbacks. That is what makes jig a kit and not a
  framework.
- **The toolkit rule.** The application owns its roots and its loops,
  its server and its URL space, its presentation and its tree. jig gives
  parts, which the application composes, and decides nothing that is the
  application's. A part of jig's that would decide for it (a page, a
  path, a guard, a confirmation) is optional: a default or a reference,
  never mandated.
- **jig gives a mid-level core.** The core is the parent of tasks,
  authority, people, the fleet, accounts, briefs, notes and views. It
  routes among them and keeps the engine's numbers, its runs and their
  calls, and the order of restart. The root's code grows with the
  application's connectors, not with jig's internals.
- **Rails, not boilerplate** (section 6). The root routes and translates;
  it never decides or orders. The rails that keep this true:
  - the commit barrier is a generic container in skein-lib, the
    journal, and it is the root's only way out;
  - the core runs restart and effects as a script;
  - every part declares its worst cases, and the journal asserts them;
  - a conformance world crashes the application's domain at every
    commit;
  - a reference root is the template every application copies.
- **Connectors meet the core through one vocabulary** (section 7):
  resources as paths, effects keyed, topics, procedures' steps,
  requirements as verdicts. A connector keeps its own state per task,
  keyed by the task's number. The core never sees a connector's types.
- **The store, the hosts and the client follow the same pattern.**
  - The store holds jig's records and the application's, in one commit.
  - Agents are optional. An application may have none, run them inside
    its engine, or run them on workers, in workspaces of its own kinds.
  - The client is one, `jig-client`, with no view (section 10). It
    carries jig's primitives as plumbing, and the application's own
    families as bytes on the same connection; a shape of work, such as a
    chat, is sugar built from the plumbing. Presentation, the server and
    the URL space are the application's, and jig's reference UI is
    optional. It runs on any platform.

  In each case the application composes jig's part with its own: in its
  roots, in its protocol layer, and in its client.
- **The core never depends on smith.** Neither does the hub, which
  carries smith's payloads as bytes. Only the roots that compose smith's
  agent capabilities, the protocol layers' pieces that decode or encode
  smith's payloads, and the client's decoding of turns below its
  surface, link smith. An application without agents never does.
- **Flexibility and orthogonality over convenience.** An application
  writes some predictable code to wire jig in. Agents write such code
  well, and getting it wrong cannot break a promise silently.

## 2. What jig is for

- **What an application gets** is what is hard to get right, held to the
  promises of `domain/core.md`:
  - authority holds;
  - every effect made once, as far as its system allows: keyed and
    conditional effects at most once, others held for a person when
    their outcome is uncertain;
  - recorded spend never passes a budget;
  - nothing leaves the engine before the commit it follows from;
  - order: dependencies first, one writer per resource;
  - nothing is lost, nothing is written over;
  - what the engine holds is bounded.

  jig's referee checks these promises in the application's own worlds,
  with the application's wiring and its systems' fakes (6.6).
- **What an application brings:**
  - its connectors: the systems it drives, their procedures, the reads
    they give agents;
  - its charters: what its agents are told to be;
  - its policy;
  - its presentation and its server, and its own families on jig's
    client.

  For example, `ops` brings an observability and an infrastructure
  connector, whose procedures remediate and provision, and charters for
  triage and incident leads (`examples.md`).
- **What jig is not:**
  - a workflow catalog: structure is the work's, made by agents with
    tools;
  - an agent: that is smith;
  - io: that is skein.

## 3. The system

```
people    a client: the application's presentation, around jig's client
engine    where work is decided: the application's process, around jig's core
store     skein-kv inside the engine: jig's records and the application's, committed together
workers   if any: execution hosts, the application's process, around jig's host
agents    if any: smith's, inside the engine or one run per agent process on a worker
systems   what the application's connectors drive
```

- **One engine per deployment.** It is the store's only writer and the
  only client of every connector's system (`domain/core.md`).
- **The engine decides, hosts host, agents think.** A host is the engine
  itself or a worker that dials it (section 9). Agents reach the engine
  only through tools their host relays.
- **Every process is the application's.** The application writes the
  root, the protocol layer, `iterate` and `main`, as with any service on
  skein. jig supplies the parts beneath its roots, and the pieces of its
  protocol layers that speak jig's vocabularies: both sides of the
  workers' channel, the LLM accounts', the host tools' translation, the
  clients' component and the client's wire, and the store, which holds
  jig's records and the application's (`protocol/README.md`, sections 5
  and 9; `protocol/store.md`). In a client, the application holds jig's
  client inside its own presentation (section 10).

## 4. The application owns the root

```
application root    routing between its children, translation; its store, channel and web vocabularies
├── core            jig: tasks, authority, people, fleet, accounts, brief, notes, views, and their routing
├── <connector>     the application's: one subtree per system it drives
├── …
└── …               the application's own child domains, if it has any
```

- **Control.** The application chooses its connectors, what it adds
  around the core, its limits, and its entrances. The core is the top of
  a subtree of its own, as a connector is.
- **Composition only.** Step code defines no traits or generic items, and
  uses no `dyn` or closures (programming-model.md, 10.3). The root
  therefore matches on what the core and each connector ask for, and
  calls the one each request goes to.
- **Change arrives on purpose.** Matches are exhaustive, with no `_`
  arm, so a new variant in jig breaks every application's build in one
  place. A change in jig reaches an application through a new pin, and
  the compiler lists what the application must decide.
- **What the core never knows:** any connector's types, the application's
  records, and any wire format.
- **An escape hatch.** The core's children are public crates. An
  application may compose them without the core. If it does, it takes on
  the invariants the core keeps, and the conformance world (6.6) still
  applies to it.

## 5. The core

- **What it holds:**
  - the routing among its children, for what arrives: a run's turn,
    call or answer; a person's request; a task due; a timer;
  - the deployment's numbers;
  - the runs in flight, and the calls committed for each since its last
    turn;
  - the gathering of a run's brief;
  - the restart script (6.4).
- **Its vocabulary toward the root:**
  - the store: its own records, saved, erased, loaded and restored, as
    every child's are (`domain/engine.md`);
  - agents' hosts: the workers' channel, the engine's side, or the host
    inside the engine (section 9);
  - people's requests, and the replies to them;
  - LLM accounts: wanted, granted, failed;
  - the connectors' vocabulary (section 7).
- **Connectors are numbered.** The core knows from its configuration that
  connector `k` exists and that its resources are paths under its
  prefix, and nothing more.
- **Its children** are each a child domain of their own, with their own
  vocabulary, limits and worlds (`domain/engine.md`, section 3):
  - **tasks**, the center of the core, which keeps every live task:
    batches, lifecycle, holds, inboxes, wakes, proposals;
  - **authority**: policy as data, checks and budgets;
  - **people**: identities, sign-ins, roles, requests, inboxes;
  - **fleet**: agents' hosts, slots, placement, attempts;
  - **accounts**: the LLM accounts and the application's credentials
    runs are lent;
  - **brief**: a run's context, in typed sections within a byte budget;
  - **notes**: what agents learn, in scopes;
  - **views**: parties' watches, each delivered its subject's current
    document as it changes.
- **Agents are optional.** An application without them configures no
  charters. The batch check then refuses tasks with agent executors, and
  the fleet and accounts stay idle.

## 6. The rails

### 6.1 The barrier

A child domain decides immediately. Its state changes as it decides, and
it emits its writes and its outward requests at once. It never waits for
durability; durability is its parent's job. For each call to its entry
point, the root does these things:

1. **Admission:** before any child changes, it checks that there is room
   for one more commit. If there is not, it answers busy.
2. **Gathering:** every write from every child the event touched goes
   into one numbered commit.
3. **Holding:** every output that someone outside could act on is tagged
   with that commit. That covers an effect, an assignment, an
   acknowledgement, and an answer to a person or to a tool call. The
   output is held until the store says the commit is durable.
4. **Releasing:** held outputs leave in their order. After a failed
   commit, nothing leaves, and the engine stops.

The order of a restart, and the authority check on every effect, belong
to the same set of duties.

Together, these make "once" and "nothing ahead of its commit" true.
Connectors are the application's children, so their writes and outputs
pass through the application's root, and that root is on this path
whatever jig does. Its mistakes pass every test of expected behaviour and
show only when the engine crashes:

- **An output that leaves early:** an effect made before the commit of
  its outbox entry, made but never recorded;
- **A decision split in two:** a call's record in one commit and its
  outbox entry in the next;
- **Admission after a change:** a connector changed, then no room found
  to commit what it changed;
- **Restart out of order:** the core deciding before a connector has
  restored its outbox, and asking again for an effect already in flight;
- **An effect that skips authority:** a procedure's effect reaching its
  connector's outbox without the core's check.

The rails below make each of these hard to write, loud when written, and
caught by a test.

### 6.2 The journal, in skein-lib

The barrier cannot live in the core: the core may not hold the
application's records and outputs, since its code may not be generic.
The barrier is not policy either. It is a data structure, as a queue is.
So it goes into skein-lib, generic over the writes and outputs it holds,
as lib's containers are generic over what they hold. programming-model.md,
10.2 allows exactly this: lib may use generic types, and application code
does not hand-roll data structures.

- **What it does,** in short (`domain/root.md`, section 4, has it whole):
  it admits a decision by counts (commits in flight, writes per commit,
  outputs held); it gathers one decision's writes and outputs into a
  value that makes at most one numbered commit; it tags each output with
  the commit it follows, or the last commit made if the decision made
  none, and releases outputs in order as the store answers, nothing
  after a failed commit; and its worst case is priced like every lib
  container's.
- **What it does not do:** inspect what it holds. Whoever owns the bytes
  checks byte bounds before handing them over. The deployment's record
  and numbers are the core's.
- **It keeps to the subset:** generic types and lib's own containers, no
  traits defined, no closures. Its design is a section of skein's
  `lib.md`.

### 6.3 One shape for every root

The root's entry points have no outbound queue of their own. What leaves
the domain is taken from the journal by the service's `iterate`, so the
journal is the only way out. In outline (`domain/root.md`, section 3,
gives the shape whole):

```
step(root, event):
    journal.takes(room for this event)?  else: busy     ← before any child changes
    d = journal.decision()
    route the event through the children: each write and each outward request goes into d
    journal.accept(d)                                   → at most one commit out
store answers n → journal.committed(n) → the outputs held for n leave, in order
```

Outputs that decide nothing and show nothing durable leave through one
explicit door of the journal, untagged, so review finds every use. These
are, among others, a run's text streamed to watchers as it is written,
and answers to a run's read tools (`domain/engine.md`, 4.2). What parties read and watch shows state, so it is
held like any other output (`domain/engine.md`, 5.2;
`client/domain/contract.md`, sections 6 and 7).

### 6.4 Ordering belongs to the core

- **Restart is a script** (`domain/engine.md`, section 6; `domain/root.md`,
  section 8). The core asks the root for one step at a time:
  1. the core's own live ranges, loaded;
  2. connector `k` restored: the root asks the connector for its live
     ranges, loads them and feeds them back;
  3. runs adopted, as workers dial in;
  4. connectors reading afresh what their systems own of the live work;
  5. the outbox settled: every entry that may have been made is looked
     for by its key before it is made again;
  6. decisions open.

  The root does the step it is asked for, and the core asks for the next
  one only after hearing that this one is done.
- **Effects follow one loop** (`domain/engine.md`, 4.4;
  `domain/connectors.md`, section 4):
  1. a connector proposes an effect, from a procedure's step or from an
     agent's call;
  2. the core admits it against authority;
  3. the connector records the outbox entry, in the same decision;
  4. the connector makes the effect only when the journal releases it.
- **Write holds:** the core keeps holds and writer slots, in tasks
  (`domain/tasks.md`, section 6). A connector tells the core when it has
  read a resource afresh after a lost attempt.

### 6.5 Mistakes are loud

- **Worst cases are declared.** Each child declares the most it writes
  and outputs per event, and the root adds them up along the route
  (programming-model.md, 4.5).
- **The journal asserts.** A decision that exceeds what it reserved stops
  the engine (fail-stop), rather than leaving state ahead of the store.

### 6.6 Mistakes are caught: the conformance world

- **What jig ships:**
  - the fakes for its own peers: the store on skein-kv's in-memory mode,
    scripted workers and scripted people;
  - the referee for `domain/core.md`'s promises;
  - the faults: crashes at every commit, commits held, commits failed.
- **What the application adds:** its whole domain and the fakes of its
  systems.
- **Generic in the world, concrete in the domain.** A world is ordinary
  Rust (programming-model.md, 10.2), so the conformance world may be
  generic over the application. The application's domain stays concrete.
- **The referee looks from outside.** Before a fake system saw any
  effect, that effect's record was durable. No run was assigned without
  a durable claim. No person was answered ahead of their commit. Every
  effect was made as its recovery class allows.

### 6.7 A reference root

jig's example application, `ops` (`examples.md`), is a small system for
running services in production. It is chosen because it shares none of
the shapes of work of an application that writes code:
- two connectors, observability and infrastructure;
- a stream of events, mostly noise;
- effects on shared and scarce resources that cost money;
- its agents inside its engine.

It is complete: a root, its connectors, their fake, a client and worlds.
Its root is the template every application copies. With
exhaustive matches and one shape (6.3), roots look alike, and agents fill
in their arms.

### 6.8 What is left to the application

Which child an event goes to, and the small total functions between
vocabularies. A mistake there is an ordinary bug in the application, and
its own stories find it. It cannot break jig's promises silently,
because every write and every output still passes through the journal,
and the conformance world crashes the application at every commit.

## 7. Connectors

- **The vocabulary between the root and a connector is jig's API**
  (`domain/connectors.md`):
  - resources named as paths of bytes, with roles: owned, fork,
    context; the objects behind them owned or participating;
  - write holds, and one writer at a time;
  - effects keyed per connector, made from the outbox, with uncertainty
    after a failure or a restart;
  - effect and read kinds as numbers, whose order authority holds as
    data;
  - topics, and news for subscribers, classified as waking, kept or
    dropped;
  - procedures' steps and their decisions;
  - projections: what a connector writes for people to read;
  - brief sections and reads for agents;
  - workspaces: what a worker prepares for a run (section 9);
  - drift, and adoption.
- **Requirements are verdicts.** Facts such as CI at a head or a
  reviewer's approval are a connector's. The connector that makes an
  effect judges its own facts and gives a verdict: met, wait, or refuse
  with a reason. The core holds one rule: no effect without its verdict.
- **What the core does not act on, it does not hold.** A connector keeps
  its own state per task, keyed by the task's number, and the root joins
  the two. What the core orders but does not read (a brief section, a
  read's answer, a connector's result) it names by token and size, and
  the root puts outputs together from their owners' parts
  (`domain/core.md`, section 5).
- **The set is closed in each application.** The application's root
  knows every connector and matches on them exhaustively. Nothing below
  the root knows any connector but itself.
- **A missing place in the vocabulary is jig's to add,** since no root
  can add one. A second application tests the vocabulary for that reason.

## 8. The store

- **jig's part:**
  - the core's records and keys;
  - their encoding, ordered as the keys are and versioned;
  - transcripts as files, one per task;
  - the secret records: minted credentials' and services' credentials'
    digests (`protocol/store.md`, section 9); LLM accounts' records are
    skein's accounts driver's own files.
- **The application's part:**
  - its connectors' records: outboxes, procedures' states, what was made
    by key, and each connector's own state;
  - the root's keys, records and writes, which wrap both jig's records
    and its own;
  - its families' keys and encodings, handed to jig's store as bytes
    under the application's tag (`protocol/store.md`, section 12).
- **One commit spans both,** through the journal. The live records of
  both are loaded by the core's restart script (6.4). skein-kv is
  underneath, inside the engine (`domain/engine.md`, section 5).

## 9. Agents and their hosts

- **smith is the agent.** Nothing in smith assumes code, repositories or
  a workspace, so the same agent serves every application, told apart by
  its charters.
- **The core never depends on smith.** It keeps turns opaque, and reads
  only a turn's size, its spend, the last message it read and whether it
  opens a window. Charters are put into smith's form by one translation
  of jig's, outside the core (`domain/hosts.md`, section 8).
- **What the core needs is a host for each run, not a worker.** The
  core's contract with a run is:
  - a start: the charter, the brief, the current window of the
    transcript, the messages waiting, credential grants, an attempt
    number, the run's identity;
  - turns sent up, committed and acknowledged;
  - calls relayed and answered once, and messages relayed;
  - an answer at the end;
  - cancellation, and graces for a host that is lost.

  smith's host contract carries the same, as frames over a channel or as
  calls inside one process (smith's `domain/host.md`, sections 3 and 9).
- **Three shapes of application,** each with its costs in
  `domain/hosts.md`, section 3:
  1. **No agents,** procedures and people only, such as an approval
     pipeline: no host, no smith, no LLM accounts (section 5).
  2. **Agents in the engine,** such as a triager or a chat assistant,
     whose tools are the engine's and the connectors' reads: the
     engine's root composes the hub with `smith-inline-agent`, and the
     fleet places runs on the engine's own slots, with no workspace and
     no process containment.
  3. **Agents on workers,** for runs that execute commands, hold
     workspaces or need many machines: each run is its own agent
     process, whose tree a stop ends. On one person's machine the engine
     and its workers may share one process, the agents still processes
     of their own (`protocol/service.md`, section 10).

  An application may mix the second and third shapes: a charter says
  which hosts may run it.
- **One host for every shape that runs agents** (`domain/hosts.md`):
  - `jig-domain-host`, the hub: slots, admission, fenced attempts,
    relays, turns and the answer kept until acknowledged, cancel;
    composed by the engine's root for its own slots, and by a worker's
    root;
  - its agent, a capability of two kinds, both smith's, speaking smith's
    parent face: agent processes, supervised through
    `smith-host-domain`, on a worker; `smith-inline-agent`, smith's
    domains in the host's process, in the engine, or on a worker whose
    root chooses it;
  - smith's payloads, bytes in the hub and typed in the agent,
    translated by hand-out to the root's protocol layer
    (`domain/hosts.md`, 6.8);
  - its link to the core, its root's: a channel the worker dials, or the
    core's fleet directly, in the engine.
- **Workspaces are the application's.** A connector says which of its
  resources a run needs prepared. A connector whose resources are not
  files gives a run tools instead. The worker's root is the
  application's: it composes jig's host, smith's host domain and the
  application's kinds of workspace (a checkout of a repository, say). The
  engine has no workspace, by policy.
- **Host tools:** the engine's tools are jig's, and each connector's
  reads are declared by the application.

## 10. The client

```
presentation   the application's: routes and URLs, screens, words and look, confirmations, drafts, home and navigation
helpers        jig's, optional: shapes, facts and models, jig's words, the reference UI
the client     jig-client: the link, watches by handle, the request table, kept work, families as bytes
transport      the application's: HTTP, a socket or in-process; jig's default HTTP mapping and its example guards, to use or replace
the API        jig's messages: plumbing operations, subjects and reads, documents with revisions and offers
the engine     the application's root around jig's core, and jig's clients' component
```

- **Nothing below the presentation knows what a page is,** and nothing
  of jig's knows HTTP but its default mapping onto it.
- **Presentation is the application's:** its routes and URLs, its
  screens, words and look, its confirmations and drafts, its home and
  navigation. jig's client knows no page, place or arrangement. An
  application presents with skein's UI kit (skein's `ui.md`), with
  anything else, or with nothing at all.
- **So are the server and its URL space.** The application's service
  owns the clients' listener, its routes and every HTTP and browser
  guard. It serves its own pages and bundle, through skein's serving
  contract (skein's `ui-web.md`, section 4), its own API, and whatever
  else it wants. It mounts jig's clients' component,
  `jig-protocol-engine-clients`, under a prefix of its choosing, and
  carries jig's messages to it and back. jig reserves no path
  (`protocol/service.md`, section 2).
- **The clients' component is transport-agnostic.** It takes jig's
  messages from the transport, each with the credential it came with as
  opaque bytes, or none, and the stream's opening and end; it answers
  with jig's messages and the stream's events. It takes a `Mint` from the
  application's server code, never from a client. It knows no HTTP,
  path, cookie, host or CORS (`protocol/client.md`, section 2).
- **jig's default HTTP mapping** puts the component's messages onto
  HTTP, at both ends: `jig-client-protocol-http` at the client's,
  `jig-protocol-engine-clients-http` at the engine's. Its routes sit
  under the prefix the application mounts it at; its stream is
  server-sent events; the credential travels as a cookie in a browser,
  and as a bearer token elsewhere. Its engine end carries example guards:
  a Host allow-list, a required header that forces a preflight with no
  CORS, the cookie's attributes, and a referrer policy on the printed
  link's route. The page's policy (its content security policy, frames)
  is the application's page's, of which jig's test application's server
  sends an example (`protocol/browser.md`, 12.1). An application uses
  the mapping as it is, wraps it, or writes its own; replacing it means
  taking over what it held. jig's test kit carries attack stories (a
  cross-site request, a rebound Host, a framed page) an application runs
  against its own server, checked and never mandated
  (`protocol/browser.md`).
- **jig owns credentials; the application owns sign-in flows and
  guards.** jig mints a credential for an identity, verifies a presented
  one to its party, expires and revokes them, signing out included, and
  keeps their digests in the store's secrets (`protocol/store.md`,
  section 9). Its own ways of signing in are two: the printed link, a
  one-time secret on loopback whose URL the application prints, and a
  service's credential (`domain/people.md`, section 3). Any other way is
  the application's sign-in: its own routes, redirects included, then a
  `Mint` of the clients' component for the identity it verified. The
  application decides how the credential travels. jig adds no "acting
  for": a bot relays a person's decision with that person's own
  credential, naming the revision it showed them.
- **One client, with no view.** `jig-client` is a step machine the
  application holds as a component, `jig_client::Client`, with `step`,
  `fire` and `next_deadline` (`client/domain/client.md`). It holds:
  - the link;
  - watches and reads by handle, and the latest document of each;
  - keyed requests, through skein-lib's request table;
  - the work kept across a restart, in a small store its host keeps;
  - the application's own families, as bytes under their owner's
    number, on the same connection.

  It emits jig's messages, never HTTP, and is built for the host and for
  wasm32. The application calls it and reads its documents; it never
  calls the application, and runs no loop of its own. Turns are decoded
  below its surface, by `jig-protocol-transcript`, so neither the
  application nor jig's helpers link smith. It serves three modes:
  long-lived, in a browser or a terminal whose host drives its loop; a
  service, the same with a service's credential and no presentation;
  and one-shot, which signs in, reads or sends, waits for the durable
  answer and exits, with its small store in a file
  (`client/domain/client.md`, section 20).
- **Plumbing and sugar.** jig's API is its primitives, as plumbing
  (`client/domain/contract.md`):
  - operations: create a batch of tasks, propose, withdraw, message a
    task, amend, cancel with a grace, release, stop, move a task to
    oneself, decide, answer, take, hand back, mark what it has read, and
    the project's own (notes, policy, roles, pools, services, adoption);
    several of them in one compound request, committed in one decision,
    all or none;
  - subjects and reads: `me`, a task, a tree, a run's text, a task's
    activity, a proposal, the inbox, task queries, chips, the deployment,
    and the application's own;
  - documents with their revisions, the acts the party may make (made,
    proposable or refused), and a decision's effect, as data.

  Sugar is built only from plumbing, in three forms: templates, data the
  client fills in; helpers, functions composing plumbing requests; facts
  and models, pure functions over documents. Templates are the
  application's configuration's, jig's chat and goal among its defaults,
  which a project's policy may override; the engine keeps them and
  serves them merged in `me`, and never expands or checks one
  (`protocol/configuration.md`, 3.1). Whatever must be atomic, whatever
  only the engine can compute, and whatever the engine's own behaviour
  turns on, is plumbing, in general form and never shaped: so a goal, a
  tracked task with a priority, is plumbing, and a chat, a template, is
  sugar (`domain/core.md`, section 9).
- **jig's helpers are optional,** modules of `jig-client` on its public
  surface alone, as the application's code is: `shapes`, which fills
  jig's templates into requests and composes a chat's closing; `facts`,
  such as a chat's status; `thread`, a conversation's turns, words and
  provisional text in order; and `words`, jig's English for its enums, a
  phrasebook an application overrides phrase by phrase. A helper the
  application ignores costs it nothing. jig adopts sugar an application
  wrote when a second consumer uses it, or early when its interpretation
  is hard to keep consistent, and once it has settled
  (`client/domain/client.md`, section 19). Correctness (revisions, once,
  atomicity) is never sugar.
- **The reference UI is optional:** jig's pages, cards, frame,
  confirmation, drafts and view, in `jig-client-view`, on the public
  client alone and skein's UI kit, presented over the native shell,
  `jig-client-shell-native`, or jig's browser parts,
  `jig-client-shell-web` (`client/domain/pages.md`;
  `client/domain/view.md`). `ops` and jig's test application use it
  (examples.md, section 8). It proves the API is enough: it reads
  nothing an application cannot. An application takes it whole, takes a
  part of it, or leaves it.
- **Every consumer is the same client:** a browser, through wasm32; a
  terminal, on skein's text shell (skein's `ui-text.md`); a phone or a
  desktop application on Linux, macOS or Windows; a chat bot; a one-shot
  command; a service with no presentation at all.
  - **Only the shell and the transport are the platform's.** The client,
    its wire and link, the default mapping's client end and the
    reference UI are step crates, built for the host and for wasm32.
  - **A shell can sit on any HTTP stack.** The default mapping's client
    end gives the shell a method, a path, headers and a body, and cuts
    the stream's server-sent events into events itself. The browser's
    fetch, the operating system's own HTTP stack, or skein's HTTP client,
    whose io is Linux's, each make a small shell.
- **Clients that are not people.** A service acting through a client is
  a party of its own kind, with an identity and a role
  (`domain/people.md`, section 3).
- **The application's own traffic** rides the same client, the same
  request table and the same connection: its families' requests, reads
  and watches, each under its owner's number (`client/domain/root.md`,
  section 15), encoded and decoded by its own wire crate. Its scopes are
  a project, the deployment, and the party's own, which is admitted on
  the credential alone; its watches and reads carry windows; each
  owner's records are versioned apart, and a version not served is
  refused, naming the ranges served (`client/domain/contract.md`,
  sections 7 and 8). `jig-client` and the engine come from one pin of
  jig. Data its people follow as it comes is a stream of the
  application's (`client/domain/contract.md`, 7.4).
- **The same rails as the engine's,** made for a client:
  - **the request table is the barrier:** every keyed request, jig's and
    the application's alike, leaves through skein-lib's request table,
    which has its host keep it in the small store before it is sent, and
    sends it again under its key until its answer is durable, as the
    journal keeps the engine's outputs;
  - **views are level-triggered:** a watch delivers its subject's current
    document whenever it changed, one delivery in flight, so nothing is
    queued and nothing missed;
  - **the owner keeps the value:** jig's documents carry jig's values,
    and name an application's by reference, as parts, which their owner
    shows (`domain/core.md`, section 5);
  - **the promises stay below the API,** whatever presents them: a
    decision names the revision it saw, and a moved revision is refused;
    a request is made once, and answered once durable; a document
    reflects only committed state, and leaves after the commit it
    reflects; admission and authority are the engine's; a credential is
    its party's.
- **What is the application's:** confirming a deciding act with its
  facts shown, from what jig gives as data in its documents (the
  reference UI keeps jig's confirmation); and keeping a credential from
  use across sites, its server's work, with jig's example guards and
  attack stories. jig's check, in its test kit and never mandated, finds
  in an application's worlds a decision naming a revision no document
  delivered to it carried, or a deciding act made on a tree that did not
  show its facts (`client/domain/testing.md`, 6.2).
- **The client's generic half** (keyed requests until durable,
  level-triggered watches, the link) is jig's while it has one user
  (`protocol/README.md`, section 10).

## 11. Building an application on jig

| Part | Written by |
|---|---|
| the engine's root domain: routing, translations, its vocabularies | the application, copied from the reference root |
| the core and its children | jig |
| connectors: their domains, protocol layers and fakes | the application |
| the journal | skein (skein-lib) |
| the store's encoding: of jig's records / of the application's, and their composition | jig / the application (`protocol/README.md`, section 9) |
| the protocol layers' components for jig's vocabularies: the workers' channel, accounts, host tools, configuration / its connectors' protocols, its parts' encodings | jig / the application (`protocol/README.md`, sections 5 and 6) |
| the engine's protocol layer's routing, `iterate` and `main` | the application, copied from the reference (`protocol/service.md`) |
| agents' hosts: the hub, the charter's translation / the worker's root, kinds of workspace, `main` (none without agents) | jig / the application |
| the agent and its capabilities, `smith-host-domain` and `smith-inline-agent` / its charters | smith / the application |
| the UI kit: the tree and its vocabulary, the builder, the diff, the phrasebook's machinery, Markdown, scenes, the outline's rules; the text shell; the browser parts and their default mapping; the bundler; the fake person, its browser face and the harness | skein (`skein-ui`, `skein-ui-text`, `skein-ui-web`, `skein-ui-bundle`, `skein-fake-person`, `skein-fake-person-browser`: skein's `ui.md`, section 2) |
| its presentation: its routes and URLs, its screens, words and look, its confirmations and drafts, its home and navigation; on skein's UI kit, on anything else, or on nothing | the application |
| the client: `jig-client`, its domain, jig's wire and its link; jig's helpers, `shapes`, `facts`, `thread` and `words` / its families' wire, carried as bytes by the same client, and which helpers it uses | jig / the application (`client/domain/client.md`; `protocol/client.md`) |
| the server: the clients' component, transport-agnostic, and credentials, minted, verified, expired and revoked; the printed link and services' credentials / the clients' listener, its routes and URL space, its pages and bundle, the component mounted under a prefix of its choosing, its sign-in flows and every guard | jig / the application, copied from jig's test application's (`protocol/service.md`; `protocol/client.md`) |
| the transport: jig's default HTTP mapping at both ends, `jig-client-protocol-http` and `jig-protocol-engine-clients-http`, with its example guards, and attack stories in jig's test kit / the default mapping as it is, wrapped, or its own: HTTP, a local socket, or in-process | jig / the application (`protocol/client.md`, section 17; `protocol/browser.md`) |
| the reference UI: pages, cards, the frame, the confirmation, drafts and the view, on skein's UI kit, over the native shell and jig's browser parts, and jig's test application's browser shell to copy / whether it takes it, whole or in part; its families' views beside it; its browser shell, `<application>-client-shell-web`, copied from jig's test application's, `jig-client-test-shell-web` | jig / the application (`client/domain/pages.md`; `client/domain/view.md`; `protocol/browser.md`) |
| the request table | skein (skein-lib) |
| the conformance world, the referee, jig's check, the attack stories, scripted workers and people, the client's test kits / its worlds and its systems' fakes | jig / the application |

Before code, an application writes its connectors' vocabularies and
limits, and the worst cases of its root. Then, in order:

1. its connectors' domains;
2. its root;
3. its worlds, on jig's conformance world;
4. its protocol layers, its `iterate` and `main`.

What it finds missing in jig along the way is added to jig, with the
application as its first user.

## 12. Documents

- **`domain/`:** jig's domain layer: the model, tasks, authority,
  connectors, the core, the application's root, parties, hosts, and
  testing. Its `README.md` gives the reading order and the names, and
  the decisions it took on what this document left open.
- **`client/domain/`:** jig's client: `jig-client`, what it holds and
  its surface; the API between a client and the engine: plumbing
  operations, subjects and reads, documents and their revisions,
  versions; owners; the reference UI, its pages and cards and its view
  on skein's UI kit; and testing. Its `README.md` gives the reading
  order, the names, and the decisions it took.
- **examples.md:** the example application, `ops`, and the reference
  root.
- **`protocol/`:** jig's protocol layer: the components an application's
  protocol layers own, the engine's and the worker's services, the
  workers' channel, LLM accounts and LLM calls in a host's own process,
  configuration, templates among it, the client's protocol: jig's
  messages, their records and the clients' component; and jig's default
  HTTP mapping with its example guards, and jig's browser parts on
  skein's. Its `README.md` gives the reading order, the names,
  what jig needs from skein and smith, and what the domain owes, and
  `store.md` the store.
- **`draft/`:** drafts, not contract: ideas and inputs gathered for a
  design to come.

## 13. Conventions

- **A bare file name** names a document in this directory;
  `domain/core.md` names one of the domain's.
- **skein's foundation documents** are named by their own file names:
  `programming-model.md`, `testing-strategy.md`, `notes.md`. skein's
  design documents are named with skein's name: skein's `lib.md`.
- **smith's documents** are named with smith's name and their path in
  smith's `docs/design/`: smith's `domain/run.md`.
- **An application** appears only as an example, and says so.
- **Each document** says in one page what it is. Then it gives its
  structure, its parts, what it owes and is owed below the domain, its
  world, and what is open.

## 14. Open questions

The domain's design took decisions on most of what this document left
open (`domain/README.md`, section 5), and the client's on its own
(`client/domain/README.md`, section 5). What remains here:

- **Versioning:** how jig is released and pinned beyond a revision of
  its repository, and how a change to its vocabulary reaches every
  application. What is settled: an application takes `jig-client` and
  the engine's parts from one pin of jig, and each owner's client
  records are versioned apart, with a typed refusal
  (`client/domain/contract.md`, section 8).

## 15. Crates' names

- **Named by layer.** A crate's name says the layer it is in
  (programming-model.md, section 2):
  - `jig-domain-<name>`: a domain's step code: the core and its children,
    the hub;
  - `jig-protocol-<name>`: the protocol layer's: codecs, components,
    translations, parsing;
  - test kits keep their kind as their prefix (`jig-test-`, `jig-fake-`,
    `jig-scripted-`, `jig-conformance`), and worlds end in `-world`.
- **Client crates live under one prefix.** Every client crate, wherever
  it lives, is named `<project>-client-{domain,protocol,<kind>}-<...>`,
  so that everything client-related lives under `<project>-client`. The
  base crate of that prefix is `<project>-client`, as skein's `skein-ui`
  is the base of `skein-ui-*`. After the prefix comes the layer, as for
  the engine's crates (`-client-domain`, `-client-protocol-<name>`), or
  the crate's kind: `-client-view`, `-client-shell-<platform>`, a test
  kit's `-client-test-<name>`, `-client-fake-<name>` or
  `-client-scripted-<name>`, and a world's name ending in `-world`. The
  project is jig, or an application under its own name: `jig-ops` for
  `ops`. An application's name, `ops` in `jig-ops-client`, is one word,
  and never one of jig's layer or kind words (`domain`, `protocol`,
  `engine`, `test`, `fake`, `scripted`, `conformance`, `shell`, `view`),
  so that a crate named `jig-<layer>-client`, such as
  `jig-domain-client`, is no application's client.
- **What a client crate is:** a crate that exists for the client: its
  domain, its protocol (the records it speaks, even where the engine
  reads them too), its presentation, its shells, its test kits and its
  worlds. A crate that runs in the engine's process keeps the engine's
  name, even when it serves clients; a crate shared by both ends for
  another purpose keeps its own.
- **An application follows the same rules** for its own crates, under
  its own name: its engine's `jig-ops-domain-*` and
  `jig-ops-protocol-*`; its client's `jig-ops-client-domain`,
  `jig-ops-client-protocol-wire` and `jig-ops-client-view`, and its
  browser shell, `jig-ops-client-shell-web`, as every application's is
  `<application>-client-shell-web`;
  `jig-ops-domain-connectors`, its connectors' numbers, which its
  engine's root and its client share, keeps the engine's name. Its
  service (its `iterate`, its protocol layer's routing and its server)
  and its binary carry its name alone: `jig-ops-service`, `jig-ops`.
- **jig's engine crates,** by layer, and where each is designed:

| Layer | Crates |
|---|---|
| a domain's step code | the core, `jig-domain-core`, and its children `jig-domain-accounts`, `-authority`, `-brief`, `-fleet`, `-notes`, `-people`, `-tasks`, `-views` (`domain/engine.md`, section 3); the hub, `jig-domain-host` (`domain/hosts.md`, section 6) |
| the protocol layer's | `jig-protocol-charter`, `jig-protocol-transcript`, and the codecs, components and parsing `protocol/README.md`, section 5 lists; among them the clients' component, `jig-protocol-engine-clients` (`protocol/client.md`, section 2), and the default HTTP mapping's engine end, with its example guards, `jig-protocol-engine-clients-http` (`protocol/client.md`, 17.4; `protocol/browser.md`, 12.2) |

- **jig's client crates,** with `ops`'s and an application's browser
  shell, and where each is designed:

| Crate | What it is | Designed in |
|---|---|---|
| `jig-client` | the client the application holds, `jig_client::Client`, composing the client's domain and its link; jig's helpers as modules: `shapes`, `facts`, `thread`, `words` | `client/domain/client.md` |
| `jig-client-domain` | the client's domain: the link's state, watches by handle and their latest documents, the request table's use, kept work, families as bytes | `client/domain/client.md` |
| `jig-client-protocol-wire` | jig's wire records, for both ends, from schemas | `protocol/client.md`, section 3 |
| `jig-client-protocol-link` | jig's messages encoded and decoded, the stream's events, the small store's records; no HTTP | `protocol/client.md`, section 2 |
| `jig-client-protocol-http` | the default HTTP mapping's client end: a message as a method, a path, headers and a body; the stream's server-sent events cut into events; for the host and wasm32 | `protocol/client.md`, 17.4 |
| `jig-client-view` | the reference UI: pages, places, navigation, the confirmation, drafts, notices, cards and the view, on `jig-client` and skein's `skein-ui` | `client/domain/pages.md`; `client/domain/view.md` |
| `jig-client-shell-native` | the native shell, over skein's HTTP client, with `jig-client-protocol-http` | `protocol/client.md`, section 12 |
| `jig-client-shell-web` | jig's browser parts, over skein's `skein-ui-web`, each usable alone: the client's adapter over skein's fetch, the printed link's sign-in, jig's look | `protocol/browser.md`, section 2 and 5.1 |
| `jig-client-test-shell-web` | jig's test application's browser shell, written to be copied | `protocol/browser.md`, 3.1 |
| `jig-client-test-domain`, `jig-client-test-view`, `jig-client-test-kit`, `jig-client-test-records`, `jig-client-test-protocol-wire` | the client's test kits | `client/domain/testing.md` |
| `jig-client-scripted-engine` | the engine as the client's worlds play it | `client/domain/testing.md`, section 4 |
| `jig-client-fake-person-browser` | jig's stories for a browser shell and its attack stories, on skein's face and harness | `protocol/browser.md`, section 15 |
| `jig-client-world`, `jig-client-protocol-world`, `jig-client-wire-world` | the client's worlds: through its surface; through bytes; its wire's records | `client/domain/testing.md`, section 4; `protocol/client.md`, section 14 |
| `jig-ops-client-domain`, `jig-ops-client-protocol-wire`, `jig-ops-client-view`, `jig-ops-client-shell-web` | `ops`'s client crates, written to be copied: its client's domain, its families' wire, its presentation on the reference UI, its browser shell | `examples.md`, section 8; `protocol/client.md`, section 11 |
| `<application>-client-shell-web` | an application's browser shell: its start and its loop, copied from `jig-client-test-shell-web` | `protocol/browser.md`, 3.1 |

- **Not client crates,** since they run in the engine's process: the
  clients' component, `jig-protocol-engine-clients`, and its world,
  `jig-engine-clients-world`; the default mapping's engine end,
  `jig-protocol-engine-clients-http`; and `jig-fake-parties`, the parties
  of the engine's worlds. `jig-protocol-transcript`, turns decoded for
  the engine and the client alike, keeps its name too.
- **Outside the layers,** a crate is named by its kind: the engine's test
  kits (`jig-test-domain`, and the rest), its worlds, the reference
  root's test application, `jig-test` and `jig-test-service`
  (`protocol/service.md`, section 7), and `jig-boundary`, the check of
  the crate graph.
- **The UI kit's crates are skein's** (skein's `ui.md`, section 2): the
  tree, `skein-ui`, which `jig-client-view` builds on; the text shell,
  `skein-ui-text`; the browser parts, `skein-ui-web`; the bundler,
  `skein-ui-bundle`, a tool that makes the bundle a browser loads
  (skein's `ui-web.md`, 4.2); the person, `skein-fake-person`, and its
  browser face and harness, `skein-fake-person-browser`. jig has no
  crate of its own for them: a terminal presents the client with
  `skein-ui-text`, and the reference UI with jig's settings
  (`client/domain/view.md`, section 15).
