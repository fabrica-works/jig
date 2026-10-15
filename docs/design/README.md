# jig's design

Provisional, 2026-10-06, revised 2026-10-15. What jig is, how an
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
  - The client domain covers the primitives, and the application adds
    its own kinds of object. It runs on any platform.

  In each case the application's root composes jig's part with its own,
  in its domain and in its protocol layer.
- **The core never depends on smith.** Neither does the hub, which
  carries smith's payloads as bytes. Only the roots that compose smith's
  agent capabilities, the protocol layers' pieces that decode or encode
  smith's payloads, and the client's decoding of turns below its domain,
  link smith. An application without agents never does.
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
  - its client's own objects.

  For example, temper brings a forge connector whose procedure lands
  changes, and charters for coders and reviewers.
- **What jig is not:**
  - a workflow catalog: structure is the work's, made by agents with
    tools;
  - an agent: that is smith;
  - io: that is skein.

## 3. The system

```
people    a client: the application's, around jig's client domain
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
  client's wire and connection, and the store, which holds jig's records
  and the application's (`protocol/README.md`, sections 5 and 9;
  `protocol/store.md`).

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
  vocabulary, limits and worlds:
  - **tasks**, the hub: batches, lifecycle, inboxes, wakes, proposals;
  - **authority**: policy as data, checks and budgets;
  - **people**: identities, roles, requests, inboxes, chats;
  - **fleet**: agents' hosts, slots, placement, attempts;
  - **accounts**: the LLM credentials;
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

- **What it does:**
  - **admission by counts:** commits in flight, writes per commit,
    outputs held;
  - **a decision:** a value that gathers one decision's writes and
    outputs; it is `#[must_use]` and not `Clone`;
  - **at most one numbered commit per decision;**
  - **tags:** each output carries the commit it follows, or the last
    commit made if the decision made none;
  - **release:** outputs leave in order as the store answers, within a
    bound per step;
  - **a failed commit:** nothing after it is released, and the journal
    says stop;
  - **its worst case,** priced like every lib container.
- **What it does not do:** inspect what it holds. Whoever owns the bytes
  checks byte bounds before handing them over. The deployment's record
  and numbers are the core's.
- **It keeps to the subset.** It uses generic types and lib's own
  containers, and nothing else of what lib alone may use. It defines no
  traits and takes no closures. Its design is a section of skein's
  `lib.md`.

### 6.3 One shape for every root

The root's entry points have no outbound queue of their own. What leaves
the domain is taken from the journal by the service's `iterate`, so the
journal is the only way out:

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
are a run's text streamed to watchers as it is written, and answers to a
run's read tools. What parties read and watch shows state, so it is
held like any other output (`client/domain/contract.md`, sections 6 and
7).

### 6.4 Ordering belongs to the core

- **Restart is a script.** The core asks the root for one step at a time:
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
- **Effects follow one loop:**
  1. a connector proposes an effect, from a procedure's step or from an
     agent's call;
  2. the core admits it against authority;
  3. the connector records the outbox entry, in the same decision;
  4. the connector makes the effect only when the journal releases it.
- **Write holds:** the core keeps holds and writer slots, in tasks. A
  connector tells the core when it has read a resource afresh after a
  lost attempt.

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
temper's shapes of work:
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
  - resources named as paths of bytes, with roles: owned, participating,
    context;
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
  - the secret records: sign-ins' digests (LLM accounts' records are
    skein's accounts driver's own files).
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
- **Three shapes of application:**

  ```
  no agents           procedures and people only             no host, no smith, no LLM accounts
  agents in-engine    the hub and smith-inline-agent         no workspace; tools are host tools only
  agents on workers   the hub and agent processes            workspaces, commands, many machines
  ```

  1. **No agents,** such as an approval or operations pipeline. Nothing
     links smith (section 5).
  2. **Agents in the engine,** such as a researcher, a triager or a chat
     assistant, whose tools are the engine's tools and the connectors'
     reads. The engine's root composes jig's host with smith's
     `smith-inline-agent`, smith's domains in this process, and the fleet
     places runs on the engine's own slots. The costs:
     - no process containment, which is fine when nothing runs
       commands;
     - the engine's worst case includes its runs;
     - LLM traffic goes through the engine's protocol layer, on the
       engine's loop.
  3. **Agents on workers,** when runs execute commands, hold workspaces,
     or need many machines, as temper's do. Each run is its own agent
     process, so stopping a run ends its process tree. On one person's
     machine the engine and its workers may share one process, the
     agents still processes of their own (`protocol/service.md`,
     section 10).

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
  application's kinds of workspace (for temper, git checkouts). The
  engine has no workspace, by policy.
- **Host tools:** the engine's tools are jig's, and each connector's
  reads are declared by the application.

## 10. The client

- **jig's part** (`client/domain/`):
  - the client domain over the primitives: tasks as a tree,
    conversations, a party's inbox, proposals, escalations, questions,
    person tasks, results and live runs, with pages, the link, and what
    a page reads and watches;
  - the wire documents for these;
  - their views, conversations among them, whose turns are decoded with
    smith's transcript crate below the domain, the client's only use of
    smith;
  - its view's primitives, on skein's UI kit (skein's `ui.md`): the
    frame's parts, the sections, the cards, the confirmation, the owed
    cards and the notices, each a function building part of a tree,
    which an application places anywhere or nowhere; jig's bindings and
    their decoders; jig's region names, style tokens and words; and the
    facts a decision rests on, as data (`client/domain/view.md`,
    section 4; `client/domain/pages.md`, section 12);
  - jig's default arrangement of them, its banner, its pages and
    `after_page`: a default and a reference, never mandated;
  - shells and their parts: skein's browser parts (skein's
    `ui-web.md`), with jig's sign-in part, its link's adapter and its
    look, and a reference shell over them to copy
    (`protocol/browser.md`); a native shell on skein's HTTP client for
    Linux and the worlds; and skein's text shell for a terminal, with
    jig's settings;
  - in its worlds, skein's fake person, who reads the tree by role,
    name and text, and jig's check, which the referee runs on every tree
    the person acted on (`client/domain/testing.md`, sections 3 and 6).
- **The application's part:**
  - its own kinds of object, families, as children of its client root,
    each owned by one of its owners, numbered alike in both its roots
    (`client/domain/root.md`, section 15);
  - their wire documents and their views;
  - its tree, built with skein's UI kit: jig's default arrangement as
    it is, its own arrangement of jig's primitives, or its own
    presentation of jig's objects from jig's documents, bound through
    jig's bindings; and its own pages, which may show jig's cards and
    sections in slots;
  - its browser shell: its start and its loop over skein's browser
    parts, jig's and its client host, copied from jig's reference shell,
    and its paths, parsed and printed by its client host
    (`protocol/browser.md`, sections 3, 5 and 9).

  The client root composes these with jig's client domain, under the
  same rule as the engine's: the root routes, and never decides or
  orders.
- **The same rails as the engine's,** made for a client:
  - **a barrier in skein-lib:** every keyed request leaves through
    skein-lib's request table, which keeps it in the client's small store
    before it is sent, and sends it again under its key until its answer
    is durable, as the journal keeps the engine's outputs;
  - **the owner keeps the value:** jig's documents carry jig's values,
    and name an application's as parts, which jig's pages show in slots
    the application's children fill;
  - **views are level-triggered:** a watch delivers its subject's current
    document whenever it changed, one delivery in flight, so nothing is
    queued and nothing missed;
  - **the promise below the tree:** a deciding act goes through the
    client domain's confirmation and names the revision of the data its
    binding was issued from, and a moved revision is refused, whatever
    tree showed it; nothing in the tree is mandated, and jig's check
    finds, in the application's worlds, a deciding act made on a tree
    that did not show its decision's facts (`client/domain/view.md`,
    section 4; `client/domain/testing.md`, section 6).
- **Any client, on any platform:** a browser through wasm32, a phone, a
  desktop application on Linux, macOS or Windows, a command-line tool, or
  another service.
  - **Only the shell is the platform's.** The wire, the protocol layer,
    the domain and the view are step crates, built for the host and for
    wasm32.
  - **A shell can sit on any HTTP stack.** The client's protocol layer
    sits above HTTP: it gives the shell a method, a path and a body, and
    gets back whole bodies, and the stream's body as it comes, which the
    protocol layer cuts into events itself. A shell may use the
    browser's fetch, the operating system's own HTTP stack, or skein's
    HTTP client, whose io is Linux's. Each shell is small.
  - **The view is optional.** The view tree is skein's, semantic,
    shaped as an accessibility tree, and each shell maps it onto its
    platform (skein's `ui.md`, section 3). A terminal has skein's text
    shell, with jig's settings (skein's `ui-text.md`;
    `client/domain/view.md`, section 15). A service may use the domain
    and the protocol only: keyed requests sent again until durable,
    watches and paging.
- **Clients that are not people.** Every party today is a person who
  signed in, with a role in a project. A service acting through a client
  needs an identity and a role too: a service is a party of its own kind
  (`domain/people.md`, section 3).
- **The web protocol** carries jig's documents and the application's
  over one connection, composed in each end's protocol layer. Sign-in
  providers are the application's.

## 11. Building an application on jig

| Part | Written by |
|---|---|
| the engine's root domain: routing, translations, its vocabularies | the application, copied from the reference root |
| the core and its children | jig |
| connectors: their domains, protocol layers and fakes | the application |
| the journal | skein (skein-lib) |
| the store's encoding: of jig's records / of the application's, and their composition | jig / the application (`protocol/README.md`, section 9) |
| the protocol layers' components for jig's vocabularies: the workers' channel, accounts, host tools, configuration, the client's / its connectors' protocols, its parts' encodings | jig / the application (`protocol/README.md`, sections 5 and 6) |
| the engine's protocol layer's routing, `iterate` and `main` | the application, copied from the reference (`protocol/service.md`) |
| agents' hosts: the hub, the charter's translation / the worker's root, kinds of workspace, `main` (none without agents) | jig / the application |
| the agent and its capabilities, `smith-host-domain` and `smith-inline-agent` / its charters | smith / the application |
| the UI kit: the tree and its vocabulary, the builder, the diff, the phrasebook's machinery, Markdown, scenes, the outline's rules; the text shell; the browser parts and their default mapping; the bundler; the fake person, its browser face and the harness | skein (`skein-ui`, `skein-ui-text`, `skein-ui-web`, `skein-ui-bundle`, `skein-fake-person`, `skein-fake-person-browser`: skein's `ui.md`, section 2) |
| the client domain, wire and native shell, the view's primitives and their default arrangement, jig's bindings, words, region names and tokens, the facts a decision rests on / its families, its client and view roots, and its tree, from jig's primitives and its own nodes | jig / the application |
| the browser's client: jig's sign-in part, its link's adapter and its look, jig's default codec for jig's places, the engine's front door and the policy's floor, jig's stories for a browser shell / its browser shell, `<application>-shell-web`: its start and its loop over skein's browser parts, jig's and its client host; its look, as data the parts take; its stylesheet and its pictures | jig / the application, its shell copied from jig's reference shell, `jig-test-shell-web` (`protocol/browser.md`, sections 3 and 5; skein's `ui-web.md`, section 3) |
| the client host: its client root, its view root and the link, and the routing between them, one machine shared by its shells; its paths, with jig's default codec for jig's places, as is, under a prefix, renamed, or its own | the application, copied from jig's test application's (`protocol/browser.md`, 3.2 and section 9) |
| what its browser changes beyond the reference shell, a step at a time, each needing the ones before only where it says so: its page, its files, its elements and components, its own JavaScript from the origin, additions to the policy, a part replaced or left out, but the engine's front door | the application (skein's `ui-web.md`, 3.3; `protocol/browser.md`, 3.5) |
| the request table | skein (skein-lib) |
| the conformance world, the referee, jig's check, scripted workers and people / its worlds and its systems' fakes | jig / the application |

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
- **`client/domain/`:** the client's domain: what a client holds, the
  application's client root, jig's pages and cards, the contract with
  the engine, the view on skein's UI kit, and testing. Its `README.md`
  gives the reading order, the names, and the decisions it took.
- **examples.md:** the example application, `ops`, and the reference
  root.
- **`protocol/`:** jig's protocol layer: the components an application's
  protocol layers own, the engine's and the worker's services, the
  workers' channel, LLM accounts and LLM calls in a host's own process,
  configuration, the client's protocol, and jig's browser layer, on
  skein's browser parts. Its `README.md` gives the reading
  order, the names, what jig needs from skein and smith, and what the
  domain owes, and `store.md` the store.
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
open (`domain/README.md`, section 5), and the client's on how jig's
pages show an application's objects (`client/domain/README.md`,
section 5). What remains here:

- **Versioning:** how jig is released and pinned, and how a change to its
  vocabulary reaches every application.

## 15. Crates' names

- **Named by layer.** A crate's name says the layer it is in
  (programming-model.md, section 2):
  - `jig-domain-<name>`: a domain's step code: the core and its children,
    the hub, the client's domain;
  - `jig-protocol-<name>`: the protocol layer's: codecs, components,
    translations, parsing;
  - `jig-view`: the client's view, on skein's `skein-ui`;
  - `jig-shell-<platform>`: the client's native shell, and jig's
    browser parts, on skein's;
  - test kits keep their kind as their prefix (`jig-test-`, `jig-fake-`,
    `jig-scripted-`, `jig-conformance`), and worlds end in `-world`.
- **An application follows the same rule** for its own crates, under its
  own name: `jig-ops-domain-*`, `jig-ops-protocol-*`, `jig-ops-view`; its
  browser shell is `<application>-shell-web`, `jig-ops-shell-web`. Its
  service (its `iterate` and its protocol layer's routing) and its binary
  carry its name alone: `jig-ops-service`, `jig-ops`.
- **jig's crates,** by layer, and where each is designed:

| Layer | Crates |
|---|---|
| a domain's step code | the core, `jig-domain-core`, and its children `jig-domain-accounts`, `-authority`, `-brief`, `-fleet`, `-notes`, `-people`, `-tasks`, `-views` (`domain/engine.md`, section 3); the hub, `jig-domain-host` (`domain/hosts.md`, section 6); the client's domain, `jig-domain-client` (`client/domain/client.md`) |
| the protocol layer's | `jig-protocol-charter`, `jig-protocol-transcript`, and the codecs, components and parsing `protocol/README.md`, section 5 lists |
| the client's view | `jig-view`, on `skein-ui` and `jig-domain-client` (`client/domain/view.md`) |
| the client's shells | `jig-shell-native` (`protocol/client.md`, section 12); `jig-shell-web`, jig's browser parts over skein's `skein-ui-web`: the sign-in part, the link's adapter and jig's look (`protocol/browser.md`, 5.1) |

- **Outside the layers,** a crate is named by its kind: the test kits
  (`jig-test-domain`, `jig-test-domain-client`, `jig-test-view`,
  `jig-fake-person-browser`, jig's stories for a browser shell on
  skein's face and harness, and the rest), the worlds, the reference
  root's test application, with its reference browser shell,
  `jig-test-shell-web` (`protocol/browser.md`, 3.1), and `jig-boundary`,
  the check of the crate graph.
- **The UI kit's crates are skein's** (skein's `ui.md`, section 2): the
  tree, `skein-ui`, which `jig-view` builds on; the text shell,
  `skein-ui-text`; the browser parts, `skein-ui-web`; the bundler,
  `skein-ui-bundle`, a tool that makes the bundle a browser loads
  (skein's `ui-web.md`, 4.2); the person, `skein-fake-person`, and its
  browser face and harness, `skein-fake-person-browser`. jig has no
  crate of its own for them: its terminal client composes
  `skein-ui-text` with jig's settings (`client/domain/view.md`, section
  15).
