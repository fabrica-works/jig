# The client's API

Draft, 2026-10-15. This draft records the direction a design discussion
took on 2026-10-15 about where jig ends and an application begins on the
client side:
- jig's client becomes presentation-agnostic;
- jig's API exposes jig's primitives as plumbing, and shapes of work such
  as a chat are sugar over them;
- jig's server-side component speaks no HTTP;
- presentation, the server and the URL space are wholly the
  application's.

Nothing here is adopted: `client/domain/`, `protocol/` and `domain/` say
what is designed. Section 13 says what this changes in them. Section 15
gives the decisions taken with the user, all of them settled. Section 16
lists what is left to the designer's pass and what waits.

## 1. In one page

- **Presentation is the application's.** Its routes and URLs, its
  screens, words and look, its confirmations and drafts, home and
  navigation are all the application's. jig knows no page, place or
  arrangement. An application renders with skein's UI kit, with anything
  else, or with nothing at all.
- **So are the server and its URL space.** The application's service owns
  the clients' listener, its routes and every HTTP and browser guard.
  jig's server-side component is transport-agnostic. It takes jig's
  messages with an opaque credential, and the application mounts it
  wherever it likes. jig ships a default HTTP mapping, with example guard
  code, which an application uses if it suits it (section 10).
- **jig owns credentials, not sign-in flows.** jig mints, verifies,
  expires and revokes credentials. Its built-ins are the printed link on
  loopback and services' credentials. Any other way of signing in runs on
  the application's own routes, and then asks the engine to mint.
- **One client, with no view.** `jig-client` is a step machine the
  application holds as a component. It holds:
  - the link;
  - watches by handle, and the latest document of each;
  - keyed requests through skein-lib's request table;
  - the work kept across a restart;
  - the application's own families, carried on the same connection.

  It emits jig's messages, not HTTP. It is built for the host and for
  wasm32. The application calls it, and it never calls the application.
- **Plumbing and sugar, as in git.** jig's API is its primitives: create
  a batch of tasks, message, amend, cancel, release, propose, decide and
  answer. A shape of work is sugar: a template the client fills in, a
  helper composing requests, or a pure function over documents.
- **One rule:** sugar is built only from plumbing. Whatever must be
  atomic, what only the engine can compute, and what the engine's own
  behaviour turns on, is plumbing, in general form and never shaped
  (section 4).
- **A party does what an agent does.** The same operations pass the same
  authority check (`domain/authority.md`, 8.1 and 8.4). Proposing is
  explicit for both: a request beyond authority is refused, naming what
  it lacks, and an explicit proposal asks.
- **A goal stays plumbing, a chat does not.** A goal is a tracked task
  with a priority, and the engine acts on that mark (`domain/core.md`,
  6.2). A chat is a template: an agent's task with a label and two flags,
  closed with cancel-with-grace.
- **The engine never sees a template.** The application's configuration
  defines them, a project's policy may override their defaults, and
  `me` serves them. The client fills them into ordinary requests, and
  the engine checks those as it checks any request (section 7).
- **jig's promises stay below the API** (section 12):
  - a decision names the revision it saw, and is refused if it moved;
  - a request is made once, and answered once durable;
  - a document is delivered after the commit it reflects;
  - admission and authority are the engine's.

  Confirming a decision becomes the application's choice, and what a
  decision will do is given as data.
- **Lists are queries over tasks** by a fixed set of keys (section 8).
  Live tasks are scanned, as they are today. Ended tasks are found
  through two store indexes.
- **Optional helpers** (section 11):
  - jig's words for its enums;
  - facts and models: a chat's status, a conversation's thread;
  - helpers for the shapes;
  - jig's reference UI: today's view and pages, rebuilt on the public
    client alone.
- **Every consumer uses the same client:** a browser, a terminal, a chat
  bot, a one-shot command, or a service with no presentation at all.

## 2. Why

- **The toolkit rule** (`../README.md`, section 1): the application owns
  its loop, its URLs, its page and its tree, and jig gives parts. Today
  the client domain owns pages, places, navigation and the confirmation,
  and jig's component owns the listener and a reserved prefix. Both
  decide for the application.
- **jig's model already says so:** the client "knows the five primitives,
  not the shapes of work built from them" (`domain/core.md`, section 9).
  The requests and the wire break that rule. `StartChat`, `Close` with
  its disposals, `Prioritise`, the conversations subject and the goals
  pages each name a shape, so every new shape is a change to jig.
- **Several consumers are not screens.** factory.md's lab and run side
  (temper's `docs/design/draft/factory.md`, sections 8 and 9) need:
  - services acting as parties;
  - a chat bot paging the person on call;
  - one-shot commands.

  A client shaped around pages has no place for them, and a service's
  client root is not planned (`client/domain/root.md`, section 11).
- **The wire is already mostly data:** commands, documents and subjects
  (`jig-protocol-client-wire`'s `client-v7`). The coupling is in
  `jig-domain-client`. It holds about 10k lines of client work (contract,
  conversation assembly, objects, windows, intents, reads, store, link),
  beside about 7k lines of presentation (pages 5.0k; the confirmation,
  acts, drafts, notices, application slots and navigation).
- **A server is the application's.** skein's UI kit already gives signing
  in, cookies and routes to the server (skein's `ui-web.md`, section 2).
  A jig that fixes HTTP guards inside an application's server decides
  that server's policy.

## 3. The layers

```
presentation   the application's: routes, screens, words, look, confirmations, drafts
helpers        jig's, optional: shapes, facts and models, words for jig's enums, the reference UI
jig-client     jig's: the link, watches by handle, the request table, kept work, families
transport      the application's: HTTP, a socket, or in-process; jig's default mapping and example guards to use or replace
the API        jig's messages: plumbing operations, subjects and reads, documents with revisions and offers
the engine     the application's root around jig's core, and jig's transport-agnostic clients' component
```

- **Nothing below the presentation knows what a page is,** and nothing of
  jig's knows HTTP.
- **The helpers use only the public client,** as the application does.
  A helper the application ignores costs it nothing.
- **The application's own data,** its families' documents and requests,
  rides the same client and the same request table. It is decoded by the
  application's wire crate, as today (`client/domain/contract.md`,
  section 5).

## 4. Plumbing and sugar

### 4.1 The rule

- **Sugar is built only from plumbing.** If something cannot be
  expressed with the plumbing, it is not sugar: it is a missing
  primitive, added to the plumbing in general form.
- **So the plumbing holds:**
  - whatever must be atomic: several operations committed together;
  - whatever only the engine can compute: a join across a task's tree
    and the inbox, a pool's amount left;
  - whatever the engine's own behaviour turns on: tracked and priority.
- **Sugar never needs an engine operation, or engine knowledge, of its
  own.**

### 4.2 The forms of sugar

| Form | Example | Lives in |
|---|---|---|
| a template, as data the client fills in | the chat: charter, budget rule, flags, label | the application's configuration, overridden by a project's policy, served in `me` |
| a helper composing plumbing requests | `shapes::chat`, `close_chat` | jig's client crate, or the application |
| a pure function over documents | a chat's status, a conversation's `Thread` | jig's client crate, or the application |

### 4.3 What jig adopts

Sugar starts in an application. jig adopts it by three tests:

1. **Used by more than one consumer.** A second consumer makes it jig's,
   as with a drawing kind (`client/domain/view.md`, section 16).
2. **Hard to get right.** There are two kinds:
   - *correctness* (revisions, once, atomicity) is never sugar; it is
     plumbing;
   - *consistent interpretation* (what "waiting for you" means, how
     committed turns and provisional text interleave) goes into jig's
     helpers early, even with one consumer, since each application that
     reinvents it drifts.
3. **Settled.** Sugar in jig is a contract too, so jig adopts it once its
   shape has stopped moving among its users.

The plumbing changes rarely, and it is complete. Sugar changes faster,
and any application may refuse or replace it.

## 5. The plumbing: operations

### 5.1 The operations

| Operation | What it does | Today |
|---|---|---|
| `Create{batch}` | Tasks with their dependencies, checked whole and made at once (`domain/tasks.md`, section 4). Each task names: executor (an agent's charter, a procedure and its parameters, a person or a role), spec, result contract, authority, budget (an amount, or `AsAllowed`, 6.1), deadline, wake policy, dependencies and inputs, the resources it will write, tracked and priority, label and flags (section 6). | Agents: `delegate`. Parties: only `StartChat` and `SetGoal`, one task of a fixed shape each. The authority layer already takes a party's batch (`PersonRequest::Create`). |
| `Propose{action}` | A batch, an amendment, an effect or a widening, beyond the proposer's authority, to whoever may accept it (`domain/tasks.md`, section 9) | Agents: batches only. Parties: implicit only, through `SetGoal` and `Amend`. |
| `Withdraw{proposal}` | The proposer withdraws its pending proposal | Agents only; designed for parties (`domain/tasks.md`, section 9) |
| `Message{task, words}` | Words in the task's inbox | Parties: `Write`, which may also release. Agents: `message`. |
| `Amend{task, revision, change}` | Spec, instructions, authority, wake, address, tracked and priority | Parties name the revision. Agents do not yet. |
| `Cancel{task, reason, grace}` | Cancels a task and its tree. With a grace, the task is first asked to finish and cancelled once the grace ends. | Grace exists only for closing a chat |
| `Release{task, revision, again}` | Lifts a hold and judges the task again | Agents name no revision and give no `again` |
| `Stop{task}` | Holds a live run for the party's decision | Parties only |
| `Move{task}` | The party becomes the task's requester, funded from their pool | Parties only (`take_over`) |
| `Decide{entry, revision, choice}` | Accept, reject with a reason, or pass up a proposal or an escalation | Both |
| `Answer{entry, revision, answer}` | A question's answer, or a person task's result | Both, as their roles allow |
| `Take` / `HandBack{task}` | A person task addressed to a role | Parties only |
| `Seen{what, through}` | A party's read position over its results, or over a conversation's turns | Parties: results, and one chat's replies |
| notes, policy, roles, pools, services, adoption | as today (`domain/people.md`, 5.1 and 5.2) | unchanged |

### 5.2 Several operations in one request

- **Some decisions are made of several operations,** committed together
  or not at all:
  - closing a conversation whose delegates are live, where each delegate
    is cancelled or moved to the party, and the conversation is then
    cancelled with a grace;
  - reordering several goals at once;
  - amending a held task and releasing it.
- **One request holds a short list of operations.** They are admitted
  together and committed in one decision. The request has one key and
  one answer: all made, or refused, naming the first operation that
  failed and why.
- **The operations it may hold:**
  - `Cancel`, with or without a grace;
  - `Move`;
  - `Amend`, priority included;
  - `Release`;
  - `Message`;
  - `Stop`.

  Creating, proposing, deciding and answering keep requests of their own.
- **Bounds:**
  - one project;
  - a count bounded by a limit of the people child's, no smaller than a
    task's delegates plus one, nor than a project's goals.
- **Admission:**
  - operations are admitted in order;
  - one pool is counted across all of them (a move's reservation and the
    rest);
  - each operation names its revision where it names one alone. A
    priority change names none, as `Prioritise` does today, and the
    first commit decides (`domain/people.md`, 5.1).
- **It replaces** `Close{chat, disposals}`, `Prioritise` and `Write`'s
  release flag, which are shapes in today's plumbing.

### 5.3 A party and an agent

- **The same check.** A party's batch and an agent's batch reach the same
  check. The source differs: the party's role and pool, or the task's
  authority and what it has left (`domain/authority.md`, 8.1 and 8.4).
- **Proposing is explicit for both.** Today a party's goal or amendment
  beyond authority becomes a proposal on its own. In the plumbing it is
  refused, naming what it lacked, and the party proposes explicitly.
  - The reason is the one `domain/engine.md`, 7.3 gives for agents: a
    person is never asked by accident. It holds for services too, since a
    service is a program.
  - Proposing implicitly lives only in sugar, such as `set_goal`.
- **A party's proposal** goes to the roles the project's policy names for
  its kind. It is refused at admission when no role may accept it
  (`domain/people.md`, 5.1).
- **A party's root has someone to escalate to.** A party may create a
  root task only when its escalations have a holder: the party's role
  covers escalations, or the project's policy names an escalation role.
  This is the gate admitting a chat checks today, made general. It
  mirrors "a proposal needs someone who may accept it".
- **Where a party may reach.** A party's `Create` may depend on, or
  reference, a task in their tree, or any task of the project for a
  maintainer. An agent reaches its delegates and the tasks it was
  introduced to (`domain/tasks.md`, 8.5).
- **What stays asymmetric, by design:**
  - procedures decide nothing (`domain/tasks.md`, section 9);
  - only parties execute person tasks, stop runs, move tasks and run the
    project.
- **Gaps to close, so that 5.1 holds:**
  - parties' `Create`, `Propose`, `Withdraw`, and person tasks;
  - agents' revisions on `Amend` and `Release`, proposals beyond batches,
    and subscriptions to a connector's topics;
  - procedures' amend, cancel and message (`domain/connectors.md`, 6.3).

## 6. The plumbing: a task's fields and budgets

### 6.1 Fields

Beside what a task holds today (`domain/tasks.md`, section 3):

- **Tracked and priority,** which exist, are plumbing. The engine acts on
  them:
  - connectors' projections;
  - the core's news slot;
  - notes' scope;
  - the order of the queues for write holds;
  - the project's goals in priority order, and ended goals' index.

  "Goal" stays a word of jig's model (`domain/core.md`, 6.2). Its
  defaults are a template (section 7).
- **A label:** an opaque number the application gives, through a
  template or by hand. Queries filter on it (section 8). jig gives it no
  meaning, and no engine rule keys on it.
- **A conversation flag:** the task's turns are entries for its requester
  (replies in their inbox), with a read position per party, and its
  unread count.
  - Read positions are kept only for flagged tasks, so their bound stays
    people × tasks with a conversation, as today.
  - The positions over ended conversations are kept in the store, beside
    the ended task's row, not in the people child's memory.
- **A retention class:** what becomes of the task once it has ended.
  - *kept:* to the deployment's retention horizon, once retention exists
    (`protocol/store.md`, 11.1), and until then for good;
  - *forgotten once read:* erased once its requester has read its turns
    and its result, as an ended chat is today.

  jig's default chat template keeps conversations, so closed chats stay
  readable.
- **Last activity:** when the task last heard words, replied, or had a
  delegate end. It is kept on every task, and it orders lists.
- **Waiting on:** what the task waits for, computed by the engine, since
  only the engine holds the tree and the inbox together:
  - a party (who), for a question, a proposal or a person task in its
    tree;
  - its delegates;
  - news;
  - nothing.

### 6.2 `AsAllowed`

A budget may be given as `AsAllowed`: as much as the creator's role and
pool allow at admission, capped by the role's spend. The pool's amount
left is a live join no client can compute without racing, so the engine
resolves it, and the answer says what was given. This is the one piece
of plumbing the templates need.

## 7. Templates

- **A template is a recipe for creating a task of one shape:** the
  defaults that turn "a chat with these words" into a complete `Create`.
  It is the application's definition of a shape.
- **The engine never sees one.**
  - The client fills a template into an ordinary request, and the engine
    checks that request as it checks one written by hand.
  - A template grants nothing. Whatever it makes, a `Create` could make
    within the creator's authority.
  - A client that ignores a template only makes its own lists
    inconsistent; authority and limits still hold.
- **Where templates live:**
  - **The application's configuration** defines them, numbered and never
    renumbered: its own shapes (the lab's experiment, the run side's
    incident) and jig's defaults, chat and goal, which it may keep,
    change or drop.
  - **A project's policy** may override a template's defaults for that
    project, for example this project's chat charter or a budget cap.
  - **`me`** serves each project's merged templates to the client (8.1).
  - **Permissions stay primitive:** which charters and executors a role
    may create with, its spend, and its delegation (`domain/authority.md`,
    section 6). No "may use template X".
- **What a template carries:**
  - the executor and charter;
  - the result contract;
  - authority, as the creator's role narrowed;
  - the budget: an amount or `AsAllowed`;
  - a deadline;
  - wake and park, which are the charter's;
  - tracked, with a default priority;
  - flags and the label;
  - the grace for cancelling, which its helper uses.
- **A task keeps the values it was made with.** A template changed later
  touches no task already made.

| Field | chat | goal |
|---|---|---|
| executor | an agent; the project's chat charter | a charter or a procedure the role may create with |
| contract | a report | a report |
| authority | the role's, one level shallower in delegation | the role's, one level shallower, with the budget and deadline given |
| budget | `AsAllowed` | given, within the allotment |
| wake, park | the charter's | the charter's |
| tracked | no | yes, with a priority |
| conversation | yes | no |
| retention | kept | kept |
| label | chat | goal |
| cancel grace | the template's, used by `close_chat` | none |

## 8. Reads and watches

### 8.1 Subjects

- **`me`:** the signed-in party's own document, always about whoever the
  credential belongs to. It is today's `Frame`, renamed, since it
  describes the party, not a frame. It holds:
  - the party: number, kind, login and name;
  - their projects, with each project's name, the party's role in it
    and their pool;
  - what waits for them, counted (questions, proposals, escalations,
    person tasks), and their unread updates;
  - their read position over results;
  - each project's merged templates, which replace today's offered
    charters and chat budget;
  - what they may create: the bounds a draft is compared with, such as
    their allotment left;
  - their role in the deployment, if any.
- **`task{n}`, `tree{root}`, `run{n}`:** as today. `run` is provisional
  text, marked so.
- **`activity{n}`,** paged.
- **`proposal{key}`.**
- **`inbox`:** the people child's own index, with its filters (project,
  kind, root). It is not a task query: proposals, escalations and
  questions are entries, not tasks.
- **`tasks{query}`** (8.2).
- **`chips`, `deployment`,** and the application's own subjects (8.4).

### 8.2 Task queries

- **Keys:**
  - project;
  - requester (a party or a task);
  - root;
  - parent;
  - label;
  - phase class (working, waiting, held, closing, ended);
  - executor kind;
  - tracked.
- **Orders:** number, last change, last activity, priority, ended at.
- **Windows** by cursor. Live tasks come first, then ended ones, in two
  groups, as pages are today.
- **Live tasks are scanned,** as every list is today: the tasks child
  keeps no secondary index, and its live tasks are bounded by
  `Limits.tasks`. Any conjunction of the keys costs no more than today's
  lists. The fixed set bounds watch marking and the wire, not the
  filtering.
- **Ended tasks are found through store indexes:**
  - `(requester party, label, ended at, task)`;
  - `(project, label, ended at, task)`, only for labels a template marks
    as listed per project;
  - delegates, updates and results keep their families
    (`protocol/store.md`, section 7).

  The two replace today's conversation and goal families. A filter beyond
  an index's prefix scans its pages, which may come back cut.
- **What is not served:**
  - full-text search (`protocol/store.md`, section 16);
  - ended tasks filtered by outcome, resource or spend;
  - keys valued by a procedure's parameters, which belong in the
    application's own records naming the tasks;
  - ended goals across projects.

### 8.3 Offers and facts

- **A document offers what the party may do now:** each act, and whether
  it would be made, proposed or refused (`Offered` today).
- **A decision's effect is data:**
  - what accepting a proposal grants and costs, and the pool after;
  - what releasing does;
  - what cancelling ends.

  An application shows these wherever it confirms.
- **Bounds are in bytes and windows,** never in rows sized for one
  client's screen.

### 8.4 The application's own traffic

- **Scopes:** a project, the deployment, and a party's own: what is the
  signed-in party's across projects. The party's own is admitted on the
  credential alone. The connector judges what any scope shows
  (`client/domain/contract.md`, 7.2).
- **Windows:** the application's watches and reads carry a window, as
  jig's subjects do.
- **Streams:** the application's data followed from an offset, provisional
  and never committed state (`client/domain/contract.md`, 7.4). They are
  designed in the pattern of a run's text, which leaves through the
  journal's door for what decides nothing (`../README.md`, 6.3), and
  built when an application needs them.
- **Versions:** each owner's records are versioned apart, and a version
  not served is refused with a typed answer naming what is served
  (section 10.4).

## 9. The client

### 9.1 Its surface

```
jig_client::Client: step(now, Event, &mut Queue<Request>), fire(now), next_deadline()

Event, in
  from the application:
    Start { kept }                      the small store's records, as the host loaded them
    SignIn { secret } | SignOut         a printed link's secret, or a credential the application obtained
    Watch { handle, subject, window } | Unwatch { handle }
    Read { handle, read, cursor }
    Send { key, request }               plumbing operations, or a family's request as bytes
  from the transport:
    Delivered { message } | StreamGone | Stored

Request, out
  to the transport:   Message { message } | OpenStream | CloseStream | Save | Erase
  to the application: Link(state) | Changed { handle } | Ended { handle, why } | Page { handle, page }
                      Pending { key } | Answered { key, outcome } | Kept { key, request } | Text { handle }

read by the application's view:
  client.document(handle), client.pending(key), client.link()
```

The transport is the application's (section 10). jig's default mapping
turns `Message` into an HTTP exchange under the application's prefix,
and back.

### 9.2 Its parts

- **Handles replace generations.** The application names each watch and
  read. Anything that arrives for a handle it dropped is dropped.
  Several handles on one subject share one engine watch.
- **The request table is the client's.** It holds jig's operations and
  the application's family requests alike:
  - each is saved before it is sent;
  - each is sent again under its key until its answer is durable;
  - each is restored after a restart and reported as kept.

  No client root routes it any more (`client/domain/root.md`, section 4).
- **It keeps the latest document of each handle,** within the watch count
  and documents' byte bounds. Turns are already decoded below the
  client's surface, by `jig-protocol-transcript`, so neither the
  application nor the helpers link smith.
- **Families:** the application's records travel as bytes under its
  family number. Its wire crate encodes and decodes them.
- **The credential:** the client holds the credential only when its
  transport needs it in hand (a bearer token). In a browser, the
  application's server may keep it in a cookie the client never sees.
- **Modes:**
  - *long-lived:* a browser or a terminal; the host drives its loop;
  - *a service:* the same, with a service's credential and no
    presentation;
  - *one-shot:* sign in, read or send, wait for the durable answer, and
    exit, with the small store in a file.

  The client runs no loop of its own in any mode.
- **Builds:** for the host and for wasm32. It is step code under the
  strict subset (`programming-model.md`, section 10).

## 10. Transport, serving and sign-in

### 10.1 The application's server

- **The application's service owns the clients' listener, its routes and
  its URL space.** It serves its own pages and bundle (through skein's
  serving contract, skein's `ui-web.md`, section 4), its own API, and
  whatever else it wants.
- **It mounts jig's clients' component wherever it chooses,** under any
  prefix, and carries jig's messages to it and back. jig reserves no
  path.
- **jig's clients' component is transport-agnostic.** It takes jig's
  messages, each with the credential it came with as opaque bytes:
  - a keyed request;
  - a read;
  - opening or closing a watch;
  - the stream;
  - signing out.

  It answers with jig's messages and the stream's events. It knows
  nothing of HTTP, paths, cookies, hosts, CORS, SSE or WebSocket.
- **The transport is the application's:** HTTP, a local socket for a
  command line, or an in-process channel for a one-process deployment or
  a world.

### 10.2 Guards are the server's

The guards that protect a browser's credential belong to the transport,
not to jig's model, so they are the application's server's:

| Guard | What it stops |
|---|---|
| a Host allow-list | DNS rebinding: a hostile site's name pointed at a local engine |
| CSRF defence: a `SameSite=Strict` cookie, a required header that forces a preflight, no CORS | another site making the person's browser send a decision with their cookie |
| cookie attributes: `HttpOnly`, `Secure`, `Path` | scripts reading the credential; the credential reaching other routes |
| the page's policy: CSP, no frames, no inline styles | injected scripts acting as the person; a decision tricked by a frame |
| a referrer policy where a URL carries a secret | the printed link's secret leaking through `Referer` |

- **jig ships example guard code:** its default HTTP mapping applies these
  guards, and an application uses it if it suits it, wraps it, or writes
  its own. Replacing it means taking over what it held, as with skein's
  browser parts (skein's `ui-web.md`, 3.3).
- **jig's test kit carries attack stories** an application may run
  against its own server: a cross-site request, a rebound Host, a framed
  page. They are checked, never mandated, as jig's check is
  (`client/domain/testing.md`, section 6).
- **What jig promises, and where its promise stops:** a request presented
  with a valid credential is that party's, and everything after is checked
  below (section 12). Keeping a credential from being used across sites
  is the application's server's work, as keeping it out of its logs is.

### 10.3 Sign-in

- **jig owns credentials:**
  - it mints a credential for an identity;
  - it verifies a presented credential to its party;
  - it expires and revokes them, including signing out.

  Their digests stay in the store's secrets (`protocol/store.md`).
- **Its built-in ways of signing in:**
  - the printed link's one-time secret, on loopback. The URL the link is
    printed with is the application's;
  - services' credentials (`domain/people.md`, section 3).
- **Any other way is the application's.** It runs the flow on its own
  routes, redirects included (OAuth, a forge, a chat platform's
  identity), verifies the identity, and asks the engine, from the server
  side, to mint a credential for it. The application decides how the
  credential travels.
- **Today's provider route** (one POST exchange per numbered provider,
  `jig-protocol-engine-clients`) goes.
- **A bot relaying a person's decision:** jig adds no "acting for".
  - The person links their identity on the bot's platform through the
    application's sign-in, and the bot holds that person's own
    credential.
  - A decision is the person's request, naming the revision the bot
    showed them.
  - It is built when the run side needs it.

### 10.4 Versions

- **Each owner's records are versioned apart,** with the ranges the
  engine serves stated in the hello, as today
  (`client/domain/contract.md`, section 8).
- **A version not served is refused with a typed answer** naming the
  ranges served. Today it is a bare HTTP 400.
- **`jig-client` and the engine come from one jig pin** in an
  application. A change to the plumbing moves jig's wire version, and
  before a release every range is one version.
- **How jig itself is released and pinned,** beyond git revisions, waits
  for an application outside this workspace (`../README.md`, section 14).

## 11. Helpers and the reference UI

- **Shapes:**
  - `shapes::chat(me, project, words)` fills the chat template into a
    `Create`;
  - `close_chat(task, disposals)` gives one request holding several
    operations (5.2);
  - `set_goal(..)` fills the goal template into a `Create`, or a
    `Propose` when the creator's authority does not cover it.
- **Facts:** a chat's status, from its phase, its run and what it waits
  on; what a decision will do, read from its document.
- **Models:** a conversation's `Thread`. It merges committed turns, the
  party's words, work summaries, proposals and provisional text in
  order, across windows and gaps. This is what `jig-domain-client`'s
  conversation assembly does today.
- **Words:** jig's English for its enums (phases, holds, refusals), as a
  phrasebook an application overrides phrase by phrase.
- **The reference UI:** today's `jig-view` and the page half of
  `jig-domain-client`, rebuilt on the public client alone, for ops and
  jig's test application. It proves the API is enough. temper does not
  use it.
- **The transport's defaults:** the HTTP mapping on both ends, and its
  example guards (10.2).

## 12. The promises, below the API

| Promise | Where it is held |
|---|---|
| A decision names the revision it saw; a moved revision is refused | the engine (`client/domain/contract.md`, 5.2) |
| A request is made once, and answered once durable | the request table and the engine |
| A document reflects only committed state | the engine's views, held through the journal |
| Admission and authority | the engine |
| A credential is its party's | jig's credentials (10.3) |
| A deciding act is confirmed, with its facts shown | **the application.** jig gives the facts as data (8.3), and its test kit may check, in an application's worlds, that a decision named a revision the person was shown. |
| A credential is not used across sites | **the application's server** (10.2), with jig's example guards and attack stories |

## 13. What it changes

### 13.1 In jig's design

- **`client/domain/`:**
  - `client.md` loses pages, places, navigation, arrangement, the
    confirmation and drafts, and becomes the client (section 9);
  - `root.md` loses the client root's routing of jig's child and of the
    table;
  - `pages.md`, `objects.md` and `view.md` describe the reference UI;
  - `contract.md` takes sections 5 to 8;
  - `testing.md` tests the client through its surface, and leaves trees
    and the fake person to skein and the applications.
- **`domain/people.md`, 5.1, 5.2 and 7, and `domain/tasks.md`, sections
  3, 4, 7 and 9:** party create, propose and withdraw, cancel with a
  grace, requests holding several operations, the root's escalation
  holder, the task's new fields, `AsAllowed`, and credentials minted on
  the server's word.
- **`domain/core.md`, section 9:** that the client knows the primitives,
  restated for the API.
- **`protocol/client.md` and the wire:**
  - jig's messages, transport-agnostic;
  - plumbing operations, task queries and `me`;
  - typed version refusals and the application's party scope.

  `Address`, `Place` and `Draft` leave the wire and the small store.
- **`protocol/browser.md`:** mostly replaced. jig serves no page, no
  bundle and no reserved prefix. What remains is the default HTTP mapping
  with its example guards, and the attack stories.
- **`protocol/service.md` and `protocol/configuration.md`:**
  - the application's service owns the clients' listener and mounts
    jig's component;
  - templates join the application's configuration;
  - the clients' bundle directory leaves jig's startup.
- **`../README.md`, sections 10, 11 and 15.**
- **skein-ui's `later.md`:** its four rows waiting for "the analysis of
  jig's boundary with applications" are replaced by this direction:
  - jig's frame primitives;
  - `facts` and `check`;
  - the reduced `jig-shell-web`;
  - the reference shell and jig's browser tier.

### 13.2 In jig's code

- **`jig-domain-client` splits.** Its client half and
  `jig-protocol-client-link` become `jig-client`. Its page half joins
  `jig-view` as the reference UI.
- **`jig-protocol-engine-clients` stops owning a listener.** Its routes
  become the default HTTP mapping, and its sign-in providers become
  credentials minted on the server's word.
- **The engine gains:**
  - the operations of section 5;
  - the fields and `AsAllowed` of section 6;
  - task queries and the two ended indexes.

  It sheds most of its shape-specific code, about 3,500 lines of domain
  code today: about 1,300 for chats only, about 1,450 for goal machinery
  that stays in general form, and the rest for lists.
- **The wire** collapses its shape-specific records, about 45.

### 13.3 In temper

- **temper builds its own presentation and server on `jig-client`,**
  after its mockups (`docs/design/web/mockups/`).
- **What goes:**
  - its copied client root, view root and native client;
  - the repin onto jig's pages that its STATUS.md plans ("jig's client
    composition").
- **Its terminal** is a presentation of the same client, on skein's text
  shell or its own.

### 13.4 In skein

- **Nothing now.**
- **Later:** the generic half of the client may move to skein, as
  skein-ui did, once a second user such as hearth's wire wants it. That
  half is keyed requests until durable, level-triggered watches, the
  link, and generic guards.

## 14. Wrong today, whatever is decided

- `ConversationLine.changed`, `ConversationLine.runtime` and
  `GoalLine.changed` are always sent empty
  (`jig-protocol-engine-clients/src/translate/live.rs`).
- Conversations are ordered by task number, not by last activity. Ended
  goals are ordered by number, not by when they ended.
- `forget_chat` erases an ended chat once it is read, so a list of closed
  chats can only show unread ones (`jig-domain-core/src/inbox.rs`).
- A party's amendment proposal names the amended task as its proposer, so
  a rejection reaches the task, not the person.
- "The goal of a task" is computed two ways: the root only
  (`documents/project_tasks.rs`), or the nearest tracked ancestor
  (`documents/task_tree.rs`).
- Parties cannot create person tasks or withdraw their proposals, as
  `domain/people.md`, section 8 and `domain/tasks.md`, section 9 say they
  may.
- Agents' `release` and `amend` name no revision.
- `Navigation.home` is stored and never read (`jig-domain-client`).
- temper's `temper-service/src/family/refusals.rs` is a copy of
  `jig-protocol-engine-clients`' private `translate/refusals.rs`.

## 15. Decisions taken

All taken with the user on 2026-10-15.

**The direction:**

1. **Presentation-agnostic.** jig's client knows no page, and all
   presentation is the application's.
2. **The confirmation** is the application's UX. jig gives a decision's
   effect as data, and holds the revision, the "once" and admission
   below.
3. **Derived states:**
   - the engine computes what needs a join only it holds (what a task
     waits on);
   - helpers derive what follows from documents the client has (a chat's
     status, the thread).
4. **jig's view and pages** stay as an optional reference UI, on the
   public client alone.
5. **Service and one-shot clients** follow from the client's shape. They
   are built when the lab or the run side needs them.
6. **The generic half in skein:** later, by the rule skein-ui followed.

**Plumbing and sugar:**

7. **Sugar is built only from plumbing.** jig adopts sugar when it has
   two consumers, or is hard to interpret consistently, and once it has
   settled.
8. **Proposing is explicit in the plumbing,** for parties as for agents.
9. **Requests may hold several operations,** bounded as 5.2 says.
10. **A goal stays plumbing:** tracked and priority, with the engine's
    behaviour on them.
11. **Labels are opaque.** Flags carry semantics.
12. **Read positions** are kept only for tasks with a conversation, and
    for ended ones in the store.
13. **Retention** is a task's class, kept or forgotten once read. jig's
    default chat keeps.
14. **A party's root has someone to escalate to,** generalizing the gate
    on chats.
15. **Templates are expanded by the client,** never seen by the engine.
    The application's configuration defines them, a project's policy may
    override their defaults, and `me` serves them. Permissions stay
    primitive. `AsAllowed` is the plumbing they need.
16. **`me`** is today's `Frame`, renamed and extended (8.1).

**The server and the client's transport:**

17. **The server and the URL space are the application's.** jig's
    clients' component is transport-agnostic, and jig reserves no path.
18. **Guards are the server's.** jig ships example guard code in its
    default HTTP mapping, and attack stories in its test kit, to use if
    they suit.
19. **jig owns credentials, the application owns sign-in flows,** beyond
    jig's printed link and services' credentials.
20. **The application's traffic:**
    - a party scope;
    - windows on its watches;
    - typed version refusals;
    - streams designed in the run-text pattern and built when needed.
21. **Versions** stay per owner. The client and the engine come from one
    jig pin, and jig's own release and pinning wait.
22. **A bot relays a decision with the person's own credential.** jig
    adds no "acting for".

## 16. Left to the designer, and what waits

- **Left to the designer's pass:**
  - the limit for requests holding several operations, with its worst
    case;
  - the configuration's form for templates, and how a policy overrides
    them;
  - the exact messages of the clients' component and of `jig-client`'s
    surface;
  - the default HTTP mapping's routes and stream framing;
  - the store keys for the ended indexes and for conversations' read
    positions;
  - how today's shape-specific wire records collapse.
- **Waits:**
  - retention's horizon (`protocol/store.md`, 11.1);
  - the application's streams, until an application follows its data as
    it comes;
  - jig's release and pinning, until an application outside this
    workspace;
  - service and one-shot clients, and the bot, until the lab or the run
    side;
  - moving the generic half to skein, until a second user.
