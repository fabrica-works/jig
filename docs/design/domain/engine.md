# The core

Provisional, 2026-10-07, revised 2026-10-16. What jig's core is, as a
domain layer inside an application's engine: its parts and how they fit,
what crosses between it and the application's root, how a decision
reaches the store before anything leaves the engine, how it restarts,
how it runs agents, and the capabilities it keeps (the fleet, briefs,
notes, views, accounts). The model is core.md; tasks are tasks.md,
authority authority.md, connectors connectors.md, parties people.md,
hosts hosts.md; what the application's root does with all of it is
root.md. The mechanics are those of skein's `programming-model.md`. What
is still open is listed in section 16.

## 1. In one page

- **The core is where work is decided.** It keeps the tasks, checks
  every action against authority, starts agents' runs on hosts, asks
  parties, and tells the application's connectors what to make, judge,
  step, project and release. It is a child of the application's root,
  and the parent of its own children.
- **A parent and its children.** The tasks child domain is the hub;
  authority is policy; the fleet, people, accounts, briefs, notes and
  views are capabilities. The core routes between them and faces the
  root (section 3).
- **It speaks one vocabulary to the root** (section 4): the store's
  records, the hosts', the parties', the accounts', and the connectors'
  in jig's terms. It knows connectors by number.
- **A decision commits once,** and nothing leaves before it does
  (section 5). The core says, of every output, whether it waits for the
  decision's commit; the root's journal holds it until it is durable.
- **A restart is the core's script** (section 6): its own records, then
  each connector's, then the runs adopted, the connectors reading afresh
  and settling their outboxes, and only then decisions.
- **Agents run from their task** (section 7). A run is given its task's
  charter, a brief built for why it runs, the messages waiting in its
  task's inbox, its transcript's current window if it resumes, and the
  engine's tools its authority allows. Each turn it takes is
  committed as it ends; every call that changed something is committed
  and answered once, however often it is asked.
- **It computes what only it can** (section 11): the documents parties
  watch and read, from committed state: `me`, the signed-in party's own;
  task queries, over the live tasks it holds and the ended tasks' indexes
  in the store; what each task waits on, from its tree and the entries
  waiting in it; what a decision would do, as data. A conversation is
  known by its flag, never by its charter, and a template is served,
  never used.
- **Bounded.** The core holds live work only: live tasks, their inboxes
  and proposals, the runs in flight. Ended tasks, transcripts, history
  and notes are in the store, loaded when wanted.
- **The domain is complete** (programming-model.md, section 4): a world
  of domains and fakes runs everything the core does, with no protocol
  and no io (section 15).

## 2. The core in the engine

```
application root        the journal, routing, translation; faces the protocol layer
├── core                tasks, authority, people, fleet, accounts, brief, notes, views
├── <connectors>        the application's
├── jig-domain-host     jig's hub, for runs on the engine's own slots, if the application runs agents there (hosts.md)
├── smith-inline-agent  the hub's agent capability in the engine: smith's domains in this process, with no workspace
└── …                   the application's own children
```

- **The root faces the protocol layer,** and the core faces the root.
  Everything the core hears arrives through the root, already in the
  core's vocabulary; everything it asks for leaves through the root,
  which translates it and puts it into the journal (root.md).
- **Hosts are reached through the root:** workers over the channel, which
  the protocol layer carries; the engine's own hub as a sibling the root
  routes to (hosts.md, 6.1), its agent smith's `smith-inline-agent`
  (hosts.md, section 5). Both speak the same vocabulary. The core's
  fleet does not tell them apart beyond what placement needs.
- **The engine never reads a system** but through its connectors; what a
  workspace holds is for the run to find.
- **The store is the engine's alone.** Nothing else reads or writes it,
  so the engine needs no compare-and-swap against anyone: its own
  ordering of commits is the truth.

## 3. Structure

```
jig-domain-core             the core: routing among its children, runs, calls, the restart script, numbers
├── jig-domain-tasks        the hub: tasks, batches, lifecycle, holds, inboxes, wakes, proposals
├── jig-domain-authority    authority's order and checks; the rules and the projects' policies
├── jig-domain-people       parties: identities, sign-ins, roles, requests, inboxes
├── jig-domain-fleet        hosts: slots, placement, attempts, relaying
├── jig-domain-accounts     LLM accounts and an application's credentials: usable, granted
├── jig-domain-brief        a run's brief, from typed sections and connectors' sized ones, within a budget
├── jig-domain-notes        notes: scopes, entries, indexes
└── jig-domain-views        parties' watches, level-triggered
```

The tree follows programming-model.md, 4.5: each child domain has its
own vocabulary, limits and world; a parent owns its children's state and
routes between them; siblings share no domain types, and the core
translates between them with small total functions.

- **`tasks` is the hub:** it knows a task's lifecycle and nothing of
  runs, connectors or clients.
- **`authority` is policy:** decisions over data, keeping only the rules
  and policies in force.
- **The others are capabilities,** each knowing its own concern: the
  fleet the hosts, people the parties, accounts the credentials runs are
  lent, the brief a run's context, notes what agents learn, views who
  watches.
- **The core keeps little of its own:**
  - the deployment's numbers: the next task number and every other number
    the core gives, committed as they are used;
  - the runs in flight: for each, its task's number, its attempt, and
    what its assignment was made from, so an answer can be routed and a
    brief rebuilt (section 7);
  - for each live agent task, its conversation's first attempt and where
    its current window starts (7.1 and 7.2);
  - the calls committed for each live task since its last committed turn,
    by attempt and name, with the core's part of their answers (7.3);
  - the decisions in progress, between the hand-offs the root completes
    within one step (4.3);
  - the restart script's state (section 6).

## 4. Its vocabulary toward the root

### 4.1 What it hears

- **from the store,** through the root: the answers to its loads, in
  pages; the records restored at a restart; that the loading it asked for
  is done. It does not hear commits' answers: the journal does (root.md,
  section 4), and feeds back what the core asked to resume after one;
- **from hosts:** a host's hello with its slots, what it hosts and its
  agent's bounds; turns, with the messages a run refused, calls, answers
  and facts of runs; a host lost or back;
- **from parties:** sign-ins and their ends, a credential to mint for an
  identity the application's server verified (people.md, section 3);
  keyed requests, each one operation or a compound request of several
  (people.md, 5.1); reads, and watches opened and closed (section 11);
- **from accounts' protocol:** a generation granted, a failure, the
  component closed;
- **from the application,** through its root: each generation of an
  application's credential granted (12.1); a project's period begun,
  saying when the next begins if its source knows. The core keeps no
  calendar, infers no cadence and schedules nothing of its own
  (authority.md, section 7);
- **from connectors,** each by number, in jig's terms (connectors.md,
  section 2): resources' roles and kinds of hold; pools' slots; effects'
  descriptions; verdicts; procedures' step decisions; outbox outcomes;
  classified news for subscribers; sections and workspace items ready,
  with their sizes; adoptions; drift; a restart step done;
- **from itself:** an output it asked to resume once released (4.2), and
  its children's timers, through `fire`.

### 4.2 What it asks for

Each request the core makes says which of three it is:

- **part of the decision:** a record saved or erased, which goes into
  the decision's commit; and the hand-offs the root completes before the
  step returns: describe this effect, judge this requirement, keep or
  drop this effect, take this procedure's parameters, gather or cut this
  section;
- **held:** what must wait for the decision's commit to be durable: an
  assignment, a message relayed to a run, a cancel, an acknowledgement,
  an answer to a call or to a party, a load (so that it sees every commit
  before it), an outbox entry to make, a resumption of the core's own;
- **now:** what decides nothing: facts to watchers, the answer to a read,
  a host's refusal before anything was admitted.

The root keeps each as the core said (root.md, section 3); the journal
releases held ones in order, and the conformance world checks that
nothing held left early (testing.md, section 5).

### 4.3 Decisions and hand-offs

A decision is a route through several children and connectors, completed
within one step of the root's (programming-model.md, 4.5):

- **One event in,** from the root.
- **Hand-offs out and back:** the core asks the root for what only a
  connector can say (a description, a verdict, a section's size), and
  the root asks the connector and gives the answer back to the core,
  before its step returns. Hand-offs are bounded per decision and
  acyclic: the core never asks a connector something whose answer needs
  another of the core's answers.
- **One end:** the core says the decision is complete, and the root
  accepts it into the journal (root.md, section 3).

What must first wait for a load, a read or a host is a state of its own
(a task preparing, a brief gathering), and its decision is made in the
step its last answer arrives.

### 4.4 The usual routes

| What arrives | Route | Then |
|---|---|---|
| a run's turn | fleet (fenced) → tasks (the inbox taken) → authority (spend) → views | commit; the turn acknowledged |
| a run's call to an engine tool | fleet → authority (the check) → tasks or notes | commit; the call answered |
| a run's call for an effect | the connector describes it → authority, its judges asked → the connector keeps it | commit; answered when made, or past its deadline |
| a run's call for a read | fleet → authority (the grant) → the connector serves it | answered now; nothing committed |
| a run's answer | fleet → tasks (result checked, task closing or idle; the messages it refused or left unread stay, and wake it by its policy) → authority (spend); what it left → its connectors | commit; acknowledged; effects made |
| a procedure's step | its connector → authority (each effect, its judges; each batch) → tasks | commit; effects made; tasks started |
| a connector's news | classified by the connector → tasks (inboxes, wakes) | commit; due tasks run, procedures step |
| an outbox outcome | its connector → tasks (settled; closing advances) | commit |
| a party's request | people → authority (their role and pool; a budget given `AsAllowed` resolved) → tasks | commit; the party answered, with what an `AsAllowed` budget was given |
| a party's compound request | people (one project, within operations per request) → for each operation in order: authority, one pool counted across them → tasks | one commit, or none; one answer: all made, or the first operation refused and why |
| a task due | tasks → authority (the run may start) → brief, with its connectors gathering, and its transcript's current window loading → tasks (claim, with its inbox's messages for the start) → fleet (place) | commit the claim; assign |
| a task closing | tasks → its connectors (release) | commit; releases made |
| a goal changing | tasks → its projecting connectors | commit; projections made |
| a project's period | authority (its spend within the policy) → tasks (the period's funding opened; recurring tasks' batches; standing allotments renewed) | commit; tasks started |
| a timer | its child | as its child says |

- **A budget given `AsAllowed`** is resolved by authority as the batch is
  admitted, from what the creator may give then (authority.md,
  section 7). One task of a batch at most gives it, resolved after the
  batch's named amounts are counted, and refused as lacking budget if it
  would give nothing (tasks.md, section 4). A pool's amount left is a
  live number only the core holds, which no client could read and send
  back without racing. The task keeps the amount, and the answer, a
  party's or a run's, says what was given.
- **A compound request is one decision** (people.md, 5.1): its operations
  are admitted in order, each against what those before it left, and
  committed together, so the route above runs once per operation within
  one step of the root's, and its hand-offs are bounded by its operations
  (section 13).

## 5. Decisions and the store

### 5.1 Commits

- **One decision, one commit.** What a decision changes in every child
  and connector it touched is one commit: records written or removed, as
  one.
- **Numbered and in order.** The journal numbers commits and the store
  applies them in that order. Its answer is that a commit is durable,
  which makes every earlier one durable too.
- **The state runs ahead.** A child's state changes as the decision is
  made, not when it commits, so the next decision sees it.
- **Commits in flight are bounded,** and the store's answers are always
  taken, so the bound never stops the answer that frees it. Past the
  bound, a decision that would need a commit is not taken, and what asked
  for it is told so: a party's request is answered busy; a run's tool
  call is answered busy, and the run asks it again after a backoff
  (smith's `domain/run.md`, 5.2); a turn or an answer is answered busy,
  and its host sends it again after a backoff; a connector keeps its
  events and its outbox's outcomes until there is room. The journal says
  whether there is room before any child changes (root.md, section 4).
- **A failed commit stops the engine** (core.md, section 4). Nothing
  waiting for that commit, or any later one, goes out. A restart starts
  from the last durable commit.

### 5.2 What waits for a commit

Every output that someone outside the engine could act on waits for the
commit it follows from:

- **effects:** an outbox entry is handed to its connector to make
  (connectors.md, section 4);
- **assignments:** a run is assigned only once its claim is durable, so
  an attempt never runs that a restart would not know of;
- **acknowledgements:** a host's answer, and a run's turn, are
  acknowledged once committed; until then the host keeps them;
- **answers to tool calls** that changed state, and to parties'
  requests;
- **messages relayed to a live run,** so a run never reads what a
  restart would not have;
- **loads,** so a load sees every commit before it;
- **what parties read and watch:** a read's answer and a watch's
  delivery decide nothing, but are made from state that may run ahead of
  the store, so each waits for the last commit made before it, as an
  output of a decision with no commit of its own (section 11);
- **grants:** a generation sent to an attempt (section 12) decides
  nothing, but goes to the attempts the fleet keeps, which run ahead of
  the store: one whose assignment waits for its commit is an attempt a
  restart would not know of, and its host does not know it yet either.
  So a grant waits for the last commit made before it, and leaves after
  its attempt's assignment, since held outputs leave in order
  (`../README.md`, 6.2).

What decides nothing and shows nothing durable waits for nothing: a
run's text streamed to watchers as it is written (section 11), and the
answers to a run's reads.

### 5.3 Loads

- **What is loaded, not held:** ended tasks and their results;
  transcripts; history; notes' entries; anything a client pages through.
- **A load is a request** with a token and one terminal event, rows in
  bounded pages (programming-model.md, 4.4). What waits on it is in a
  state that says so: a brief gathering, a run being prepared to resume,
  a party's page.
- **Bounded:** the loads in flight, and the bytes a load brings, are
  limits; a load past its bytes is cut, and says how much.
- **Parties' reads** are documents and pages the client's contract lists
  (`../client/domain/contract.md`, section 6): from live state where the
  engine holds it, loaded otherwise, and answered once the last commit
  made is durable (5.2).
- **Ended tasks are read through the store's indexes.** An ended task is
  loaded by its key; a list of ended tasks, a task query's ended part,
  through one of the ended indexes (11.2): by requester, every ended task
  a party requested; a task's ended delegates; by project, the ended
  tasks flagged *listed*. A subtree's is read by the delegates' index,
  task by task. A row joins its task's record, and an ended
  conversation's read position, by point reads in the same step
  (`../protocol/store.md`, 7.3). A filter beyond an index's prefix
  scans its pages, which may come back cut, and the next page goes on
  from the last row examined.

### 5.4 What the store keeps of the core

| Record | Written by | Loaded |
|---|---|---|
| the deployment: its id, made at its first start, and its numbers | the core | at start |
| projects: resources and roles, homes, policy, parties' roles | people | at start |
| parties: identities, sign-ins | people | those with a live sign-in or a role, at start; others as they sign in |
| parties' requests' keys, with their answers, for a retention | people | at start, those within it |
| tasks: every field of tasks.md, section 3 (its label, flags and last activity among them), holds and their queues, writer slots, the last turn committed for a live attempt | tasks | live ones at start; ended ones on demand |
| the indexes of ended tasks: by requester, every ended task a party requested; by project, the ended tasks flagged listed; each saved in the decision that ends its task (11.2; `../protocol/store.md`, 7.4) | tasks | on demand |
| parties' read positions over conversations: a live one's; an ended one's, moved by the decision that ends the task and erased with it (11.4) | people | live ones at start; ended ones on demand |
| calls committed since each live task's last turn, by attempt and name, with the core's part of their answers | the core | with their task |
| each live agent task's conversation: its first attempt, and where its current window starts (7.2) | the core | with their task |
| funders' numbers: each party's pool and each project's spend, per period; a period's beginning, and when the next begins if its source said | tasks | at start |
| a task's history: made, amended, held, released, cancelled, by whom, why; and its activity: words written to it, runs begun and ended, what it heard (7.2) | tasks | on demand |
| inboxes: messages not yet taken | tasks | with their task |
| proposals: pending, and decided ones' outcome | tasks | pending ones at start |
| transcripts: turns, with their spend and whether each opens a window | the core | to resume, from the current window's start; on demand |
| notes | notes | indexes per scope in use; entries on demand |
| secrets: sign-ins' digests; LLM accounts' records are kept apart, by skein's accounts driver | the protocol layer, never the domain, which names them by account and generation, or by a sign-in's number | at start, by the protocol layer |

Each connector keeps its own records beside these (connectors.md,
section 2): its outbox, procedures' states and parameters, what it made
by key, its projections' states, and its own state. The application's
root wraps both in its store vocabulary (root.md, section 6).

**Times** the store keeps are wall times (`env.wall`), so a record means
the same after a restart; a wait the core arms is a deadline on its
monotonic clock (`env.now`), computed from them (programming-model.md,
section 9).

### 5.5 What the core asks of the store

The store is assumed (core.md, section 2); this is what the core relies
on, and all it relies on:

- a commit of several records is applied whole or not at all;
- commits are applied in the order sent, and durable once answered;
- a load sees every commit answered before it was sent;
- records are keyed by the domain's own keys, and listed by key ranges,
  in bounded pages; what is live (live tasks, pending proposals, the
  outboxes, live resources) is kept in key ranges of its own, so a
  restart reads it without reading history;
- what one engine wrote, the next reads: records carry the version of
  their shape;
- nothing else writes it.

### 5.6 How a child's records reach a commit

Each child that keeps durable state, and each connector, defines its own
records and ordered keys in its own vocabulary. Neither a sibling nor
the store chooses their shapes. The parent translates their keys and
carries their records in its own store vocabulary; the store's protocol
layer encodes each shape with a version (5.5).

- **`Save { record }`** keeps the record under its own key, and
  **`Erase { key }`** removes that key. These requests are part of the
  decision the parent is making. They have no terminal event and do not
  ask the child to wait: its state changes as it decides (5.1).
- **`Restore { record }`** supplies one record at restart, and
  **`Restored`** says every record in the child's live ranges has been
  supplied. The child then decides from that restored state.
- **`Load { owner, range }`** asks for one bounded page of the child's
  own key range. Its one terminal event is **`Loaded { owner, rows,
  more }`**. The token is echoed, and `more` says another page remains,
  which the child asks for with a load of its own.

A child never waits for a commit and never holds an output back for one.
Its outputs leave it as it decides, each saying which of the three of
4.2 it is, and its parent passes them up. A child's world plays its
parent and the store: it keeps saved records, applies erases, commits at
the end of each step, and restarts by feeding the durable records as
`Restore` followed by `Restored`. Restart is therefore tested before the
root is built.

## 6. Restart

A restart is a cold start that rebuilds everything a decision reads from
the store and the connectors, and makes nothing twice. The core runs it
as a script: it asks the root for one step at a time, and asks for the
next only once the root says this one is done (root.md, section 8).

1. **The core's records:** the deployment's numbers; projects, their
   policies and parties' roles; parties' read positions over live
   conversations, never ended ones' (11.4); live tasks with their inboxes,
   subscriptions, holds and queues; pending proposals; the calls
   committed since each live task's last turn; each live agent task's
   conversation; the funders' numbers; LLM accounts. The core's limits
   bound what is live (tasks.md, section 11), so what is loaded fits; a
   live set past limits lowered since refuses the start, saying which.
2. **Each connector's records,** connector by connector: the root loads
   that connector's live ranges and restores them into it
   (connectors.md, section 14).
3. **The runs adopted.** Every task claimed or running names its attempt
   and the kind of host it was placed on. Claims placed in the engine
   died with it, and are settled at once as lost, or as not started if
   they committed nothing (tasks.md, 5.2); so are claims on workers in
   the engine's own process, which no worker outlives
   (`../protocol/service.md`, section 10), when its configuration says
   its workers are in process. Claims on other workers reach the
   fleet, which adopts each attempt a worker reports as it dials in,
   keeps waiting for one within its grace, and cancels any it does not
   name (section 8). Nothing new is assigned before every claim loaded has
   reached the fleet.
4. **Read afresh.** Each connector reads, for the live tasks that name
   them, the resources its system owns.
5. **Settle the outboxes.** Each connector resolves every entry not
   committed as made, as its recovery class says (connectors.md, 4.3):
   looked for by its key or the state it leads to, written again if it
   is a set; one not found waits for the retry deadline committed with
   its last attempt, which the restart keeps; one of a kind that cannot
   be recovered holds its task for a person.
6. **Decide afresh.** Every procedure steps once on the facts read;
   every agent task due gets a run; every inbox's messages are relayed or
   wake as their policy says.

Workers keep the answers and turns they were not acknowledged for, and
send them again after their hello; the core commits any it does not
have, and acknowledges each.

### 6.1 Stopping

The engine stops when its service is told to (a signal, an operator), and
the core hears it from its root before anything closes
(`../protocol/service.md`, section 5). From then on it takes nothing that
would make a commit:

- **a party's request** is answered not ready, and an application's
  request likewise, and so are a sign-in, a credential to mint and a
  sign-out, which commit too (`../protocol/client.md`, section 8);
- **a run's turn or call** is answered busy, and its answer left
  unacknowledged, so its host keeps them and sends them again, to this
  engine or the next;
- **a connector's events and outcomes** stay with the connector, as when
  the journal is full;
- **no run is placed, and no procedure stepped.**

It ends every watch saying the engine is stopping
(`../client/domain/contract.md`, 7.2), and says once when nothing it holds
waits for a commit, so the service can close its components. A restart
then starts from the store as section 6 says.

## 7. Agent executors

### 7.1 From a task to a run

When an agent task is due (tasks.md, 5.2), the core:

1. **Checks** that the run may start, and with what budget
   (authority.md, 8.3).
2. **Prepares it,** the task preparing over as many steps as it takes:
   gathers its brief (section 9), for why it runs: its first activation,
   or what woke it; has each connector that owns a resource the task
   names gather its workspace items, and name the application's
   credentials they need (connectors.md, section 10); and loads its
   transcript's current window, if its charter resumes and it has one
   within the resume limit (7.2). Either way it is given the calls
   committed after its task's last committed turn: a resumed run as a
   note after the transcript, a fresh one as a section of its brief.
3. **Commits the claim:** the task claimed, with its new attempt number,
   the host kind it is placed on, the writer slots of the holds its
   workspace writes (tasks.md, 6.3), and the messages its start carries
   (tasks.md, 8.2).
4. **Places it** through the fleet, once the claim is durable, on a host
   its charter allows, with an assignment the root puts together
   (root.md, section 7):
   - its run and attempt, the core's names for them, and its
     conversation's first attempt, from which the engine's protocol layer
     derives the run's identity (smith's `domain/host.md`, section 2): 16
     bytes from the deployment's id, the task's number and that attempt,
     the same for every attempt of a conversation, for the provider's
     prompt cache, and never shared by two. No domain holds it;
   - its workspace: the connectors' items, and a workstream key, the
     number of the task that holds the resources written, so that the
     runs of one piece of work find their workspace cached;
   - its start's messages, named, oldest first; the rest are relayed as
     the run reads (7.4);
   - its charter, in the core's terms: the task's instructions, the
     brief, the tools its authority gives, the outcome its result
     contract asks for; its budget for the run: turns, spend and time,
     with a reserve of turns and time for finishing, its spend reserved
     from what its task has left in the claim's commit (authority.md,
     section 7); and its models, the main session's and those its
     sub-agents may use, each with its window (the input tokens a
     conversation's window may hold), its output (the most output tokens
     a completion may have) and its prices (authority.md, section 7).
     The engine's protocol layer translates it into smith's (hosts.md,
     section 8);
   - its transcript's current window, when it resumes, and the calls
     committed since its last turn;
   - its credential grants (section 12): those its models' accounts
     need, one account per charter, which its models' endpoints share,
     until a run needs several; and the application's credentials its
     connectors named (12.1). Each names whose table the protocol layer
     fills its value from.

### 7.2 Transcripts

- **A transcript belongs to its task,** whichever runs added to it. Its
  conversation is the turns from the last run that started fresh, and
  its current window the conversation's turns from the last that opens a
  window: a window turn, which a compaction made, or the conversation's
  first (smith's `protocol/transcript.md`, section 3). That conversation
  is the transcript's, which every agent task has; a task flagged
  *conversation* is another matter, whose turns are entries for its
  requester (11.4).
- **Beside it, the task's activity:** the words parties wrote to it as
  they wrote them, its runs' beginnings and ends, the results and news it
  heard and the decisions made about it, kept in the order they
  committed, so a client reads a conversation whole after its words have
  been taken from the inbox (`../client/domain/contract.md`, section 6).
- **Committed turn by turn.** A run sends each turn up as it ends: what
  the LLM said and the calls it made, their results, what it spent, the
  last inbox message it read, and whether it opens a window. The core
  commits the turn with its spend, with the messages up to that one
  taken but those it says the run refused (hosts.md, 6.4), and, when it
  opens a window, with its task's current window starting there; then
  it acknowledges it. A host keeps each turn until it is acknowledged,
  and a run with too many unacknowledged turns waits (hosts.md, 6.4).
  Admission is whole: the hub validates the attempt, turn and offered
  read-through before changing its spend; a refused turn changes
  nothing, and a duplicate accepted turn changes nothing again.
- **The core never looks inside.** A turn is bytes of the agent's own
  conversation vocabulary, bounded, which a client's protocol layer
  renders. The core reads only its size, its spend, what it took and
  whether it opens a window, which the turn carries beside its bytes.
- **Resuming.** A run that resumes is given its current window only,
  within the resume limit (the deployment's bound on the current
  window's bytes, across the attempts it spans, each turn counted with
  the four bytes of its length, as the window crosses to a host:
  `../protocol/channel.md`, 6.1), and, as a note after
  it, the calls committed after its last turn, with their answers. One
  past the limit starts fresh from a brief that carries the transcript's
  tail. Earlier windows' turns stay readable by parties
  (`../client/domain/contract.md`, section 6), and are never sent to a
  run. A transcript the agent cannot use fails its run as transient,
  saying so, and the next run is prepared fresh.
- **Turns are numbered within their attempt,** from one. The task keeps
  the last turn committed for its live attempt, restored with the fleet's
  adoption of the claim, so a copy already committed is acknowledged
  again and dropped, after a restart too.

### 7.3 The engine's tools

A run reaches the engine through smith's host tools, declared in its
charter, which its host relays; their schemas, decoding and rendering
are the protocol layer's (hosts.md, section 8). Each is checked against
its task's authority (authority.md, section 8), decided, committed, and
answered after the commit:

| Tool | What it does | Answers |
|---|---|---|
| `delegate` | makes a batch of tasks as its delegates, each with what a party's `Create` names, its label and flags among them (tasks.md, section 4) | their numbers and their specs' revisions, and what a budget given `AsAllowed` was given; refused, and why; beyond authority, naming what it lacked and whether a proposal could be accepted |
| `message` | words, a question or an answer to a task it references | sent; no reference; the inbox full; too large for a run to read |
| `amend`, `release` | a task below it: an amendment naming its spec's revision as the run last read it, but for a change of priority alone; a release naming its hold's, and, for an effect held uncertain, whether to make it again (tasks.md, 5.5 and section 7) | done, with the task's spec revision after the commit, the one its next amendment names (a release moves none); refused, and why; moved, with what it is now |
| `cancel` | a task below it and its tree, with a reason, and a grace if the run gives one, within the tasks child's bound: the task is asked to finish first (tasks.md, section 7) | done; refused, and why |
| `decide` | accepts, rejects or passes up a proposal or an escalation waiting for it, an escalation naming its hold's revision (tasks.md, section 9) | done; beyond its authority, and passed up; moved, with what it is now |
| `propose` | an action beyond its authority, with a reason: a batch, an amendment, an effect, or a widening of its own authority (authority.md, section 9) | proposed, and to whom; refused, when nobody may accept it (tasks.md, section 9) |
| `withdraw` | its own pending proposal (tasks.md, section 9) | withdrawn; already decided, and how |
| `introduce` | two tasks it references, each given a reference to the other, within their limits (tasks.md, 8.5) | done; no reference; refused, and why |
| `subscribe`, `unsubscribe` | a topic a connector offers to agents (connectors.md, 5.3), under a read grant on the resources the connector says it concerns (authority.md, 8.5); a referenced task's state; a timer (tasks.md, 8.4) | done; refused, and why |
| `effect` | one of a connector's kinds of effect offered to agents (connectors.md, 4.5) | made, with what it made; committed and pending, past the call's deadline; waiting for a verdict; failed |
| `note`, `recall` | writes a note in a scope; reads entries by name or by a search (section 10) | written; the entries |
| a connector's reads | reads its system (connectors.md, section 9) | the answer, within the read's budget |

**Calls are named, and decided once.** Every call names its task's
attempt and the run's own name for it. The core commits a call that
changes state together with its part of the answer, and answers a call
it has seen before from that record, never deciding it twice; what a
connector adds to an answer (what an effect made) it keeps under the
call's key, and the root puts the answer together again (root.md,
section 7). The core keeps a task's records, by attempt and name, until a
turn of the task that carries their answers commits, or the task ends;
so calls an attempt made in a turn it lost are still there for the next
attempt to be told of (7.1). A run whose call was withdrawn, lost with
its channel, or answered busy asks it again with the same name while it
lives, so the LLM is told the outcome rather than trying anew; an
effect's key is derived from the call that asked for it (connectors.md,
4.4).

Three tools a run has are not the engine's: `wait`, which the run and its
host serve, holding the slot until a message arrives (7.4); `finish`,
which the run serves, checking its result against the contract (smith's
`domain/run.md`, section 7), and whose result reaches the core as the
run's answer (tasks.md, 5.6); and `deliver`, which its host serves
through the application's workspace (hosts.md, section 9).

Beyond authority, a call is refused naming what it lacked; the run may
`propose` the same action with a reason. Calls never become proposals on
their own, so a person is never asked by accident.

**The tools are a party's operations.** Each engine tool that acts on
tasks, `introduce` aside, is one of the operations a party's requests
make (people.md, 5.1), checked by the same rules (authority.md, 8.1 and
8.4), so a run and a party do the same things to the same tasks:
`delegate` is `Create`;
`message` is `Message`, and `Answer` for a question; `propose`,
`withdraw`, `amend`, `cancel`, `release` and `decide` are theirs by
name. What only parties do stays theirs (people.md, 5.1): carrying out
person tasks, stopping runs, moving tasks, marking what they have seen,
and running the project. `introduce` is a task's alone: only a task
holds references (tasks.md, 8.5).

**A call names what it saw.** An amendment, a release and a decision
about an escalation name the revision the run was shown, as a party's
do: its brief shows its delegates' revisions (section 9), the messages
relayed to it show a hold's (hosts.md, section 9), and `delegate`'s,
`amend`'s and `release`'s answers show the spec's revisions they leave.
One whose object has moved since is answered moved, with what it is
now, and the run decides again on that; a change of priority alone
names none, and the first commit decides (tasks.md, section 7). So no
call writes over a change its run was not shown.

### 7.4 Answers, waiting and parking

- **A run answers once:** finished, with its result; parked; or failed,
  with a typed failure. The core commits it, once it has every turn the
  answer counts, and acknowledges it; a finished task closes, a parked
  one is idle, a failed one retries or is held (tasks.md, 5.5). What the
  answer says the run left on resources goes to their connectors
  (connectors.md, section 10).
- **Waiting holds the slot; parking frees it.** A run that calls `wait`
  keeps its host's slot while it waits, so a run that is talked with
  answers at once. Idle
  past its charter's threshold, it parks: every turn it took is committed
  first, and it ends. A coordinator's threshold is zero: it parks as soon
  as it waits.
- **Woken again,** a parked task is due. Its charter says whether the
  next run resumes the transcript or starts fresh from a brief; by
  default a charter for conversations resumes and a coordinator's starts
  fresh, its brief
  carrying its goal, its plan's state and its notes, and its start the
  messages that woke it (7.1).
- **Messages to a live run** are relayed as the run reads, named, in
  order, never more relayed and unread than the run's inbox takes; a
  turn that reads one takes it (7.2; tasks.md, 8.2).
- **Refused or unread, a message stays** in the task's inbox, untaken,
  and wakes the task again when the run ends (hosts.md, section 9;
  tasks.md, 8.2 and 8.3). A turn's commit never takes a message the run
  refused, whatever its fence covers (7.2).
- **A cancel** ends the run: the host cancels it and answers; what it
  committed before stays committed.
- **Saved state.** A run that parks, fails or is cancelled with unfinished
  work has it saved by its host, where its connectors say, and the next
  run of its task starts from it (connectors.md, section 10).

## 8. The fleet

The fleet child domain knows the hosts and the runs they host:

- **Hosts of two kinds** (hosts.md, section 3): workers, which dial in
  with their slots and the workstreams their workspaces hold; and the
  engine's own slots, a configured number, hosted by the engine's own hub
  (hosts.md, section 6), its agent `smith-inline-agent` with no
  workspace, present from construction. A charter says which kinds may
  run it. A run goes to a host with a free slot of a kind its charter
  allows, preferring a worker that holds its workstream. A refusal as
  busy is no failure.
- **Attempts are fenced.** Everything a host sends is dropped unless its
  attempt is its task's live one, but a cancelled attempt's answer. No
  attempt of a task is placed while the fleet knows a host may still run
  another.
- **Answers and turns are acknowledged once committed** (5.2), or once
  fenced off; a copy before then is dropped unacknowledged, one for an
  attempt the core has finished with is acknowledged again.
- **Losing contact.** A lost worker's runs are kept for a grace; one that
  reconnects says what it hosts, and the core keeps or cancels each. Past
  the grace they are lost, and their tasks retry. The engine's own slots
  are lost only with the engine (section 6).
- **Relaying:** messages and cancels down, tool calls up and their
  answers back, each call answered once; a call the fleet cannot pass up
  is answered at once, unserved, or busy while the engine stops
  (section 6.1), never left without an answer.
- **The graces are ordered.** The engine's grace is longer than the
  longest a worker may take to stop a run it lost contact for, which the
  worker says at its hello (hosts.md, 6.6); a worker whose bound is equal
  to or longer than the engine's grace is refused. So the core places no
  next attempt while a worker may still host the last, and one run at
  most writes a resource at a time.
- **A host's agent must run the engine's charters.** The fleet refuses
  a hello, as not admitted and naming why, whose agent's bounds
  (hosts.md, section 2) do not hold every run the engine may place there:
  a model a charter it may run names missing, or held with a smaller
  window or output than the charter's; turns, spend, time or waiting per
  run smaller than the deployment's most for a run, or a charter's; a checks' timeout longer
  than the reserve's time of a charter that allows a change; a largest
  message smaller than the deployment's message bound (tasks.md, 8.2), or
  an inbox that holds fewer than a start carries; fewer turns kept
  unacknowledged than the hub's bound. So no run is placed where smith
  would refuse its start, and no run refuses a message as too large or
  its start's messages as full.
- **Grants follow their attempts:** each new generation of an LLM
  account or an application's credential goes to every live attempt
  that holds one (section 12), and each attempt the fleet keeps at a
  worker's hello, or adopts, is sent its current grants, so no run
  outlives its token for want of a resend. A grant never overtakes its
  attempt's assignment (5.2), so a generation renewed while the
  assignment waits for its commit reaches the attempt after it.
- **Every start and adoption ends once:** answered, lost, withdrawn (the
  core cancelled it), or refused (busy, a duplicate, or invalid).
- **Restarting** (section 6): the fleet adopts the claims the store
  names, then hears that the loading is done; a run a worker lists that
  no claim adopts is cancelled once a grace has passed.
- **Its history is diagnostic:** the deployment's recent lost workers,
  refused hellos and capacity refusals are kept in memory, within their
  limits, for the deployment's document, and start empty after a
  restart.

## 9. Briefs

The brief child domain builds every run's brief from sections, each
within a budget, the brief within its own: sections follow why the run
runs, those it runs for are required, the rest optional; what does not
fit is cut in its kind's order and says how much; sections are gathered
within one deadline, while the task is preparing, and a required one
missing sends the task back to due after a backoff, no attempt spent
(tasks.md, 5.2).

Its sections come from two places:

- **The core's,** which it holds typed: the task (its spec and contract,
  and its lineage: its requesters' specs, cut); its dependencies' and
  inputs' results in jig's terms; its delegates, their states, and the
  revisions a decision about them names, each one's spec's and a held
  one's hold's (7.3), and the results that came since (the plan, for a
  coordinator); its earlier
  attempts and why they failed; the calls its task committed since its
  last turn, when it does not resume; the proposals and escalations
  waiting for its decision; the index of its notes; a transcript's tail,
  when it does not resume.
- **Its connectors',** which it plans by kind, priority and budget but
  never holds (core.md, section 5): for each resource the task names, and
  for each dependency or input whose result is a connector's, the
  connector gathers the section, keeps it, and says its size; the brief
  child decides what fits, asks for cuts by size, and, when the brief is
  complete, the root takes each section from its connector to put the
  assignment together (connectors.md, section 9).

A section is loaded or read as it is cut, asking for no more than it
keeps. The host's protocol layer renders the whole as text, in order.

A brief has no inbox section. The messages waiting in the task's inbox
go with the assignment as its start's messages, and the run reads them
as messages, whole (7.1; tasks.md, 8.2).

## 10. Notes

Notes are what agents learn and pass on: a service that is slow to warm
up, why an approach was dropped, which check is flaky, what a person
prefers. The notes child domain knows scopes, entries and indexes:

- **An entry** has a one-line description, a body, a scope, references to
  tasks, and who wrote it: a party, or a task.
- **Scopes:** the deployment; a project; a goal; a resource pattern of a
  connector's (a service, a repository), within a project.
- **In the store,** where parties read, correct and delete them through a
  client. Notes are hints: where a note and the facts disagree, the facts
  are right.
- **Written by a tool,** checked against the task's note scopes
  (authority.md, section 3), and committed like any decision. A revision
  names the revision its run recalled, and is refused as moved if the
  entry changed since.
- **Read in two steps.** A brief carries the index of the notes in its
  scope, a line per entry, saying how many more there are when they do
  not all fit; a run asks for the entries it wants with `recall`, by name
  or by a search of their descriptions, answered a page at a time.
  Indexes are kept per scope in use, and evicted least recently used;
  entries per scope are bounded.
- **Knowledge that belongs to a system** belongs there, such as a
  repository's guide: a note that matures may be proposed as an effect
  on it.

## 11. Views

The views child domain keeps parties' watches, and paces what each is
delivered; the core makes the documents they deliver, and answers
parties' reads, from what its children hold. What a watch shows, and
what a client reads, is the client's contract
(`../client/domain/contract.md`, sections 5 to 7); what the core
computes for them is said below (11.1 to 11.5).

- **Watches:** jig's subjects (`../client/domain/contract.md`, 7.1):
  `me`, the signed-in party's own document (11.1); a task; a task's
  tree; chips, a bounded set of tasks; the party's inbox, across
  projects; a task query's live lines (11.2); a task's live run; the
  deployment; and an application's own subjects, through the root to the
  connector that owns them. The other objects a client shows (a
  proposal, an escalation, a question, a person task, a result, a task's
  activity, a task query's ended part, a project, a note) are read, and
  read again when a watch that names them moves.
- **An application's subjects name their scope,** as its reads do, since
  the core cannot read a connector's subject: a project, the deployment,
  or the party's own, what is the signed-in party's across projects. The
  core admits a watch or a read of one on the party's sign-in and their
  role in that scope, the party's own on the sign-in alone, and on the
  views' limits, as it admits jig's; the connector, asked for the
  document, judges that the subject is in that scope and what the party
  may see of it, and answers unknown, or not allowed, otherwise
  (connectors.md, 5.5). Its watches and reads carry a window, as jig's
  do (`../client/domain/contract.md`, 7.5). Only the document, or why
  there is none, crosses back; the root only routes.
- **Level-triggered,** as facts are (core.md, section 4). A delivery is
  the subject's whole current document, within the watch's window, made
  by the subject's owner when the delivery is made: the core, or a
  connector through the root. A watch has one delivery in flight; a
  change meanwhile marks it, and the next delivery, once the watcher has
  taken the last, is made from the state as it is then. Changes in
  between are coalesced, never queued. A watch ended with a delivery in
  flight ends once that delivery is taken or dropped
  (`../protocol/client.md`, section 7).
- **Watches are never stored.** A restart (section 6) starts with none,
  and is told nothing of the deliveries made before it; each client
  opens its watches again (`../protocol/client.md`, section 4).
- **A slow watcher never holds the engine up,** and is never told it
  missed anything: it sees fewer deliveries. What the views child keeps
  per watch is its subject, its window, and two marks: a delivery in
  flight, and a change since. A task query's subject is its keys and its
  order, of a fixed size. Nothing grows.
- **Marked by commits:** each decision's commit names the tasks, projects
  and parties it touched, and the views mark the watches they concern:
  - *a task touched:* its own, the trees along its lineage, chips
    naming it, and every task query it matches before the decision or
    after it (11.2);
  - *what a task waits on moving:* the same watches of the tasks along
    its lineage (11.3);
  - *a party touched,* their entries, a turn of a conversation they
    requested, their read positions, their roles or pools, or a policy's
    templates in a project of theirs: their `me` and their inbox.

  A connector marks its own subjects (connectors.md, 5.5).
- **Durable before shown:** a delivery waits for the last commit made
  before it, as every read's answer does (5.2).
- **A run's text as it is written** is the one stream: kept per live run
  being watched, within a bound, named by the turn and the retry the host
  reports it for (hosts.md, 6.5), and delivered from the offset each
  watcher has; never stored, and marked provisional. It decides nothing
  and leaves at once (5.2). A committed turn reaches a client through the
  task's activity, read (7.2).
- **History is the store's.** What a client shows of the past (a task's
  results, its activity, a plan's revisions and why, the ended tasks a
  query finds) is read (5.3), paged, never held.

Nothing the core decides depends on a view arriving.

### 11.1 `me`

- **The signed-in party's own document,** about whoever the credential
  belongs to; no party watches or reads another's. The core makes it
  from:
  - the people child: the party, their kind, login and name; their
    projects and their role in each; what waits for them, counted
    exactly, and their updates unread, counted up to a limit; their read
    position over results; their role in the deployment, if any;
  - authority: their pool in each project, and what they may create,
    the bounds a draft is compared with, their allotment left among
    them;
  - the templates: each of their projects' merged templates.

  What it carries, field by field, is the client's contract's
  (`../client/domain/contract.md`, 5.1).
- **Templates are served, never used.** The core keeps the
  configuration's templates (`../protocol/configuration.md`, 3.1) and
  each project's policy's overrides of them (authority.md, section 6)
  only to merge them, per project, into `me`. No decision reads one: the
  client fills a template into an ordinary request, which the core
  checks as one written by hand, so a template grants nothing, and a task
  keeps the values it was made with, never which template gave them.
- **Marked** as any of it moves. A pool's amount left moves with spend;
  its marks are coalesced, as every watch's are.

### 11.2 Task queries

- **One query for every list of tasks.** A task query names keys, an
  order and a window: its keys among project, requester (a party or a
  task), root, parent, label, phase class, executor kind and tracked;
  its order one of number, last change, last activity, priority and
  ended at; its window by cursor (`../client/domain/contract.md`, 6.1).
  Its rows are task lines, the one record every list of tasks is made of
  (`../client/domain/contract.md`, 5.1): its live tasks first, in its
  order, then its ended ones, each group with its own cursor. Its live
  lines are watched; its ended ones are read, a page at a time. The core
  admits it on the party's role in the project it names, or, naming
  none, for tasks whose requester the party is
  (`../client/domain/contract.md`, 7.1).
- **Live tasks are scanned.** The tasks child keeps no secondary index,
  and its live tasks are bounded by its limits (tasks.md, section 11), so
  a query's live part is one pass over them, whatever its keys, keeping
  the window's rows in order. The fixed set of keys bounds what marking
  compares and what a query carries, never the filtering.
- **Ended tasks are found through indexes** in the store, which the tasks
  child saves in the decision that ends a task (`../protocol/store.md`,
  7.3 and 7.4):
  - *by requester:* every ended task whose requester is a party, by that
    party and the label;
  - *the ended delegates',* for a query naming a requester task or a
    parent: that task's ended delegates;
  - *by project:* only the ended tasks flagged *listed*, by their project
    and the label. The flag is the task's, set when it was made, so the
    core reads no template to index.

  A query's keys choose one index, in that order: a requester party; a
  requester task or a parent; a project (`../protocol/store.md`, 7.3).
  Its other keys, the root among them, filter that index's pages, which
  may come back cut, and the next page goes on from the last row
  examined. A query whose keys choose none has no ended part, and its
  answer says so. A task forgotten once read leaves
  its index rows in the decision that forgets it (11.4); retention erases
  the rest with their tasks (`../protocol/store.md`, 11.1).
- **What a query never finds:** words in a text (`../protocol/store.md`,
  section 16); ended tasks by outcome, resource or spend; tasks by a
  procedure's parameters, which belong in the application's own records
  naming the tasks; ended tracked tasks across projects.
- **Each line** comes from its live task, or from an ended task's index
  row joined to its record, with what only the core can give: what the
  task waits on (11.3), and, for a conversation, its unread count
  (11.4).
- **Marked by a task it matches,** before the decision or after it, so a
  task entering a query, leaving it, moving within its order or ending
  marks it. Matching compares the task's fields, and its lineage's root
  and parent, with the query's keys: a bounded comparison per watch. The
  core gives the views each touched task's keys as they were and as they
  are.

### 11.3 What a task waits on, and its last activity

Two values each task line and each task's document carry (tasks.md,
section 3):

- **What it waits on is computed,** since only the core holds a task's
  tree and the entries waiting in it together. It is never stored, so no
  restart finds it stale. It is the first of:
  - *a party,* when an entry in the task's live subtree, itself
    included, waits for a party: a question put to one, a proposal or an
    escalation waiting at parties, a person task addressed to a party or
    a role. It names whom the oldest such entry waits for, a party or a
    role, and how many wait;
  - *nothing,* when it is working: its run live and not waiting;
  - *its delegates,* when it has live delegates;
  - *news,* when it is idle, its run parked or waiting, or its procedure
    waiting on its facts: what moves it next is a message its wake
    policy lets through, or a fact its connector reads (tasks.md, 8.3;
    connectors.md, 6.2);
  - *nothing,* otherwise: its phase says what it waits for (its
    dependencies, its holds, its closing), or it has ended.

  The core computes it as it makes a line or a document, by a walk of
  the task's live subtree, within tasks per tree. An entry for a party
  made or ended, and a delegate made or ended, touch every task along
  its lineage, within the depth limit, so their watches are marked
  (section 11).
- **Its last activity is kept** on every task: a wall time, set by the
  decision that commits its cause, words the task heard, a turn its run
  committed, or a delegate's end (tasks.md, section 3). It orders lists,
  moves no revision, and stays on the ended task, so a query's ended
  rows show it.

### 11.4 Conversations

- **Known by its flag,** never by its charter, its label, or the
  template that made it (tasks.md, section 3). Only an agent's task
  carries it, so its turns are its transcript's (7.2).
- **Its turns are entries for its requester,** while that requester is a
  party: a turn committed is an update for them, and marks their `me`,
  their inbox, and the queries the task matches.
- **Read positions are the people child's** (people.md, section 6),
  moved only by the party's `Seen`. A line's unread count is the task's
  turns past its requester's position. There is one per task
  flagged conversation, its requester's, so they number at most the
  tasks flagged conversation:
  - *a live task's,* in the people child, restored at start (section 6);
  - *an ended task's,* moved to the store by the decision that ends it,
    never held, and joined by a point read where a party reads the
    conversation or a query's ended row (5.3), so an ended conversation
    reads with what its party had read.
- **A task's retention class** says what becomes of it once ended
  (tasks.md, section 3): *kept,* to the deployment's retention horizon
  (`../protocol/store.md`, 11.1), and for good while retention waits for
  a deployment whose store nears its budget; *forgotten once read,*
  erased once its requester, a party, has read its result and, for a
  conversation, its turns, and no live task names it, in one decision
  with its read position and its index rows, so no restart sees one
  without the other, and nothing read comes back unread. A task whose
  requester is not a party is kept, whatever its class.

### 11.5 What a decision would do

- **Offers are the core's judgement.** Each object's document says which
  acts the party may make on it now, and for each whether it would be
  made, proposed and to whom, or refused and why, with its bounds. The
  core judges them from the authority, policies and pools only it holds,
  and judges again when the act comes (`../client/domain/contract.md`,
  5.3).
- **A decision's effect is data,** made from the core's state as the
  document is made (`../client/domain/contract.md`, 5.4): what accepting
  a proposal grants and costs, and the pool after; what a release does;
  what a cancel ends, by phase, runs, proposals and effects; the
  reservation a move takes. Confirming a decision is the application's;
  the core gives the facts it shows, and takes the decision naming the
  revision its party was shown (`../client/domain/contract.md`, 5.2).

## 12. Accounts

The accounts child domain keeps the deployment's LLM accounts, and an
application's credentials beside them (12.1): which are usable, and
which runs hold which generation of each. The protocol layer's accounts'
component signs LLM accounts in, refreshes them, keeps their records
before lending, and holds their values (`../protocol/llm.md`,
section 5). The domain names an account by its number and a token's
generation, with how long it is valid, and never sees a value or a
record.

- **Every configured account is wanted,** from the restart's first step
  (section 6) to the close, so the component refreshes each before its
  token expires, and answers each new generation, granted.
- **Usable** while the child holds a granted generation still valid and
  the account is not spent. A run whose account is unusable is not
  started.
- **Every live attempt hears each new generation** of its account, on
  whatever host, the engine's own included; and the fleet sends each
  attempt it keeps its current grant when its worker says hello, and
  when it adopts the attempt (section 8).
- **Rejected:** a run's provider refused a generation. The child passes
  it to the component, which refreshes once; a second rejection of the
  same generation fails.
- **Exhausted:** a run's provider says the account is spent until a time.
  The child marks it spent until then, and asks nothing of the component.
- **Failed,** in one of five classes:
  - **later,** with how long: the child asks again after it, while the
    account is still wanted;
  - **sign in again:** its grant was revoked, or it has no record. The
    account is unusable, the deployment's document says a person must
    sign it in, and the child asks again at its retry interval, which
    finds a record a new sign-in put in place;
  - **misconfigured:** the provider rejected the deployment's client. The
    account is unusable until a restart with its configuration fixed,
    and the document says so;
  - **unloaded:** its record could not be read. The account is unusable,
    the document says where and why, and the child asks again at its
    retry interval;
  - **cancelled,** at the close.
- **Close** releases every account, and ends once the component has
  closed.

### 12.1 An application's credentials

A connector's workspace may need credentials of the application's own: a
token to fetch from a system and push to it. The accounts child keeps
them beside LLM accounts, and lends them the same way:

- **Named, never seen:** each by the application's number for it and a
  generation, with how long it is valid. Their values are the
  application's protocol layer's, in a table of its own, from a source
  of its own: a file, or an exchange it refreshes.
- **Granted through the root:** the application's root tells the child
  each generation granted, as the accounts' component does for an
  account (4.1). One is usable while a granted generation is still
  valid (authority.md, 8.3).
- **Lent to runs:** a connector names those a run's workspace needs as
  it gathers its items (connectors.md, section 10); the assignment
  carries them as grants, each naming the application's table (7.1;
  `../protocol/channel.md`, section 8); and they follow their attempts
  as an account's do (section 8).
- **Rejected and exhausted are LLM accounts' only.** A system that
  refuses a credential is the workspace's failure (hosts.md, 6.3), and
  its source's to refresh.

## 13. Limits and working sets

What the core holds, each bounded by its limits and summed into its
worst case (programming-model.md, 6.3):

- live tasks, with their inboxes, subscriptions, references, holds and
  queues; pending proposals (tasks.md, section 11);
- the rules and the live projects' policies, within limits on projects,
  resources per project and parties holding roles;
- parties signed in, and their pending requests; their read positions
  over live conversations, one per live task flagged conversation
  (11.4);
- the configuration's templates and each live project's overrides of
  them, which the core only serves (11.1);
- the calls committed since each live task's last turn, and the funders'
  numbers;
- hosts, their runs, and the calls and messages relayed;
- LLM accounts, and an application's credentials;
- decisions in progress and their hand-offs; loads in flight and their
  rows;
- briefs gathering, with the sizes of the connectors' sections, and the
  notes' indexes in use;
- watches, each its subject, a task query's keys and order among them,
  its window and two marks (section 11).

None of it grows with the store's history. A new task, a party, a run or
a watch past its limit is refused at its entrance, and the asker told
why. The root adds the core's worst case to its connectors' and its own
(root.md, section 10).

- **A compound request's decision** is priced as operations per request,
  the people child's limit (people.md, 5.1), times the costliest
  operation it may hold: a cancel of a whole tree, within tasks per tree,
  with the hand-offs to the connectors its tasks name. It is summed into
  the worst case with the rest, as every decision is (`../README.md`,
  6.2), so one request holding a whole conversation's close, or a whole
  project's goals reordered, fits.
- **What reads and deliveries compute** is priced with them: a task
  query's live part is one pass over the live tasks, and what each of
  its lines waits on a walk of a tree, within tasks per tree (11.2 and
  11.3); its ended part is a load, within the loads' limits (5.3). None
  of it is kept between deliveries.

## 14. Below the domain

What the protocol and io layers owe the core, through the application's
protocol layer, which composes jig's pieces with its own:

- **The store:** commits of typed records, numbered, answered once
  durable, and loads by key range in pages, over skein-kv; jig encodes
  the core's records, the application its connectors', each with a
  version per shape.
- **Workers:** jig's channel, the engine's side: assignments, messages,
  grants, cancels, answers to relayed calls, acknowledgements down;
  hellos, turns and answers with the messages refused, calls and facts
  up (hosts.md, section 10).
- **Connectors:** each its own protocol, the application's.
- **Parties:** jig's messages, carried by the application's transport to
  the clients' component, which knows no transport
  (`../protocol/client.md`): jig's documents and the application's, its
  requests and reads, and credentials. jig mints, verifies, expires and
  revokes credentials; the printed link and a service's credential are
  its own ways of signing in, and any other is the application's, which
  asks the component to mint a credential for the identity it verified
  (people.md, sections 3 and 10; `../client/domain/contract.md`,
  sections 2 and 10).
- **Accounts:** sign-in, refresh, records kept and tokens lent, by
  skein's OAuth accounts driver (`../protocol/llm.md`, section 5); an
  application's credentials by its own protocol layer, each generation
  granted told to the core through the root (12.1).
- **Configuration:** the deployment's rules, connectors and their
  systems' addresses and credentials, LLM endpoints, accounts, and models
  with their windows, outputs and prices, workers' secrets, the engine's
  own slots, the charters the deployment offers with their runs' budgets
  and reserves, the application's templates, which the core only serves
  (11.1), procedures' settings, limits. Projects and their policies are
  the store's, seeded from configuration on a first start.

## 15. The world

Each child domain has a world of its own, its parent and neighbours
scripted in it, its expectations in a referee; authority, which keeps no
state, is tested by its step tests.

The core's world runs the core with every child beneath it, under a root
of jig's own for testing, against fakes that share none of its types
(testing.md, section 4): jig's test connector, with resources, holds,
pools, priced effects, requirements, procedures and faults drawn from
the seed; one to three scripted workers and the engine's own slots,
whose runs play scripts; scripted parties; the store on skein-kv's
in-memory mode, slow or failing a commit; and the engine restarting
cold, at drawn moments or at one a scenario chooses.

Its stories: a conversation that answers, parks and resumes; a plan of
reports, a choice and procedure tasks, revised mid-way, and done; a
procedure's effect waiting for another connector's verdict, then made; a
delegate held past its tries, escalated to a person and released; a goal
cancelled with runs live and effects in flight, closing deepest first; a
tool call whose answer is lost with the channel and the engine, asked
again after a restart and answered from its record, made once; a run
handing a held resource to a procedure while it still holds the writer
slot; two tasks queued for a pool's last slot, and the pool shrinking
under them; an uncertain effect across restarts made before its retry
deadline; an effect of a kind that cannot be recovered, uncertain and
held for a person; a judge's facts changing between its verdict and the
effect; a policy narrowed while a proposal waits, an entry is committed
and a run is live; a note written, corrected by a person and recalled; a
recurring task across a period's reset; a person's words to a running
run; a conversation resumed from its current window after a compaction;
a message a run left unread, waking its task again as the run ends;
drift holding a task, and the client told why.

Its stories of what the core computes and serves: a task query watched
while tasks enter it, leave it, change its order and end, each delivery
after its commit, the ended ones then read through the ended indexes,
by requester and, for a listed task, by project, a filter past the
prefix coming back cut; what a task waits on moving from its delegates
to a party as a delegate asks its person a question, and back once it
is answered; a conversation's turns counted unread until its person
marks them seen, the conversation ended and read after a restart with
the position its person left, and one forgotten once read leaving
nothing that shows; `me` marked as a pool is spent from, deliveries
coalesced; a budget given `AsAllowed` answered with what was given.

Its stories of agents and procedures doing what parties do: a run
amending a delegate a party amended since it was shown, answered moved
with what it is now, then amending again; a run releasing a held
delegate, naming its hold's revision, and making an uncertain effect
again; a run proposing an amendment and an effect beyond its authority,
each accepted by a person; a run subscribing to a connector's topic and
woken by its news; a run cancelling a delegate with a grace it finishes
within; a procedure amending, cancelling and messaging its delegates,
one amendment refused as moved and the procedure stepping again on what
it is told.

Its referee holds the core to core.md, section 11, seen from outside;
applications' worlds reuse it on jig's conformance world (testing.md,
section 5).

## 16. Open questions

- **Commits in flight:** how many, and whether the store should take
  several decisions as one commit when they wait together.
- **Briefs:** whether a brief's store loads and connectors' gathering
  share one budget; cancelling a brief while it gathers; how a missing
  section shows in what the agent reads.
- **Transcripts' limits:** how the resume limit is sized against a
  charter's models' windows, so that a conversation smith compacts
  (smith's `domain/session.md`, section 8) resumes rather than starts
  fresh; and how long the store keeps earlier windows' turns, which no
  run is sent.
- **Views:** a limit on watchers per subject; how a watch of a task's
  tree is bounded when the tree is large; whether, should a pass over the
  live tasks per delivery, or a walk of a tree per line, prove slow, the
  tasks child keeps what a task waits on along its lineage as entries
  come and go (11.2 and 11.3).
- **Scaling out:** one engine per deployment until it binds; the store
  being the engine's alone is what would change first.
