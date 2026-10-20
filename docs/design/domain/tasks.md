# Tasks

Provisional, 2026-10-07, revised 2026-10-19. The core's tasks: what a
task is, how tasks are made, how each moves from asked to ended, the
resources it holds, what it hears and when it wakes, and how proposals
and escalations travel the tree. It is the first primitive of core.md
(3.1) in depth, kept by the tasks child, the center of the core
(engine.md, section 3).
Authority, which every action here is checked against, is authority.md;
what an agent's run is, engine.md, section 7; a connector's procedures,
connectors.md, section 6; what a party asks, people.md, section 5. What
is still open is listed in section 13.

## 1. In one page

- **The tasks child domain is the center of the core.** The other
  children serve it. It keeps every live task: its place in the tree,
  its dependencies, its lifecycle, its holds, its inbox, its
  subscriptions to other tasks and to timers, its budget's numbers, and
  the proposals and escalations waiting for a decision. It knows nothing
  of runs, connectors or clients: an executor is a kind and an opaque
  reference, a connector's news a message it is handed.
- **Batches, whole or not at all.** Tasks are made in batches, checked
  whole (no cycles, within the limits, within the creator's authority as
  authority.md answers) and committed at once.
- **Parties and agents ask alike.** A party's request and an agent's tool
  make the same batches, amendments, cancels and proposals, against the
  same checks; only the source of their authority differs. Nothing
  beyond authority becomes a proposal unasked (section 9).
- **Several operations, one decision.** A party's compound request
  cancels, moves, amends, releases, messages and stops in one decision,
  all or none (section 7).
- **One lifecycle for every executor** (programming-model.md, 5.2): a
  task waits on its dependencies and its holds, is active while its
  executor works, closes (its run ended, its delegates closed, its
  effects settled, its holds released), and ends done, failed or
  cancelled. It may be held for a decision at any point before it ends.
- **Holds are taken whole.** A task takes every resource it writes at
  once, or waits with none, so no two tasks wait on each other.
- **Decisions go up the tree.** What a task may not do alone, and what
  holds it, waits for the nearest task or party above it with the
  authority to decide, and wakes them.
- **News is kept; only some wakes.** Each task has a durable, bounded
  inbox and a wake policy over what arrives.
- **The tree is the plan's history.** Every task made, amended,
  cancelled, released or held, and why, is kept in the store as it
  commits.

## 2. Structure

The tasks child domain is a child of the core (engine.md, section 3),
and its center: it keeps every live task, and the other children serve
it. Its state is a slab of live tasks, a slab of pending proposals,
their inboxes, subscriptions and holds, and the timers it arms. It is a
step machine of its own (programming-model.md, 4.5): its own
vocabulary, limits and world.

What it does not know, and how it is told:

- **Executors** are a kind (agent, procedure, person) and a reference
  the core gives it: for an agent, a charter's number; for a procedure,
  the connector that owns it (or the core) and an opaque code; for a
  person, a party or a role, the task's project's role (people.md,
  section 1). It asks for a task to be activated, and hears when the
  activation ends.
- **Authority** is a value of its own type, which it carries and keeps
  the numbers of (budgets spent and reserved, tasks made), as it keeps
  every other funder's (authority.md, section 7), and never judges: every
  check goes to the authority child domain, whose answers say what the
  numbers become.
- **Resources** are names (a connector's number and a path) and the kind
  of hold each takes; what they are is their connector's.
- **Connectors' news** reaches it as messages already classified (8.3).
- **The store** is the root's: this domain's saves and erases are part of
  the decision the core's step is making (engine.md, 5.6).

## 3. A task

What the store keeps of a task, and the tasks child domain holds while
it is live:

- **Its number,** given by the core, never reused, and its project.
- **Its spec:** words, bounded; its typed parameters, numbers and texts,
  each text UTF-8 as the words are, never other bytes; the resources it
  reads and writes, by name; the ended tasks whose results it takes as
  **inputs**; and what its executor needs: a charter's number for an
  agent, the question for a person. A procedure's parameters are its
  connector's, kept by the connector under the task's number (core.md,
  section 5). A value that is not text, such as a commit's id, goes in
  as text its application spells (`../client/domain/contract.md`, 5.3).
- **Its result contract:**
  - *a report:* words, bounded;
  - *a verdict:* one of a closed list the spec gives, each with a
    contract of its own: which fields it requires, how many follow-up
    tasks it may propose and of which kinds;
  - *a connector's result:* what a connector's procedure produces, such
    as a change landed or an environment ready; the connector keeps it
    under the task's number, and the task keeps words that summarise it;
  - *a choice:* one of the options the spec lists, or an answer in
    words, for a person;
  - *an approval,* for a person, of a state a procedure names: the
    state as bytes its connector gave, which the core carries and
    compares but never reads, and the part its connector shows it by.
    The result names the state it approves, which must equal the
    contract's, byte for byte (people.md, section 8).

  Every result may also be a failure, with a reason: the executor's
  verdict that the task cannot be done as asked (5.6). Declining an
  approval is that failure.
- **Its requester:** a task, a party, or the deployment, for what its
  configuration or a connector starts on a project's behalf (a recurring
  task, a standing watch). Its **proposer,** when an accepted proposal
  made it requested by the accepter (section 9).
- **Its dependencies:** live tasks it starts after, each done before it
  starts.
- **Its references:** tasks it may message beyond its requester and its
  delegates (8.5).
- **Its executor** (2) and, for an agent, its **charter**: the role's
  instructions, the models it allows, in order, or every configured
  one, the tools, the outcome it must produce, its default wake policy
  and waiting time, whether a run resumes its transcript or starts
  fresh, and which hosts may run it (engine.md, 7.1). It names no
  effort: each model declares its efforts, in order, in the deployment's
  configuration (`../protocol/configuration.md`, section 3).
- **Its LLM choice,** for an agent task, what its runs' LLMs are:
  - *main:* a configured model, and an effort among those the model
    declares (`../protocol/configuration.md`, section 3); its runs use
    it, and their check holds its most costly completion (authority.md,
    8.3);
  - *sub-agents:* the models its runs' sub-agents may use, each with an
    effort, among which smith chooses within a run (smith's
    `domain/run.md`, 5.3);
  - *account:* optionally, one LLM account its creator wants in
    particular, within the account's uses (accounts.md, 4.3).

  Its creator gives it, or a template does
  (`../protocol/configuration.md`, 3.1), each model within the task's
  authority (authority.md, section 3). A task given none takes its
  charter's default as its batch is checked: the first of the charter's
  models, in its order, whose LLM provider has an account usable now and
  serving its payer, or the charter's first when none has, its run then
  waiting for an account (accounts.md, 4.3); each model the default
  names takes the first effort it declares, or none where it declares
  none, which leaves its LLM provider's default.
  The task keeps the choice it was given, and only an amendment changes
  it (section 7).
- **Its account:** the LLM account its runs use for its main model's LLM
  provider, chosen at its first run's check and kept from run to run,
  so that a conversation stays on one account's cache and replay, until
  a change of its payer or of that LLM provider chooses again
  (accounts.md, 4.3). The accounts its sub-agents use are each run's,
  recorded in its claim (engine.md, 7.1, step 3). This kept number is separate
  from the creator's optional preference. Only a successful claim changes it,
  in the same decision as its reservation and phase; a refused claim leaves
  it unchanged, and clearing the optional preference alone retains it.
- **Its authority** (authority.md, section 3), and the numbers kept
  against it: spent by itself, spent by the ended tasks it funded,
  reserved for the live tasks it funds, and who funded its own budget.
  Its budget is an amount, whether its creator gave one or `AsAllowed`
  (section 4).
- **Its wake policy** (8.3) and **its subscriptions** (8.4).
- **Whether it is tracked,** and its priority if so: a goal (core.md,
  6.2). The engine acts on the mark: it orders the queues of write holds
  by it (6.2), subscribes the goal to its connectors' goals' topics
  (8.4), has it projected (connectors.md, section 8), and gives it a
  scope for notes (engine.md, section 10).
- **Its label:** an opaque 32-bit number the application gives, through
  a template or by hand (`../protocol/configuration.md`, 3.1). jig gives
  it no meaning, and no rule of the engine's keys on it: task queries
  filter on it, and the indexes of ended tasks are keyed by it
  (`../protocol/store.md`, 4.4).
- **Its flags,** each with the engine's semantics:
  - *conversation:* its turns are entries for its requester, while that
    requester is a party, with their read position and an unread count
    (people.md, section 6), and its run is asked for its text as it is
    written (engine.md, 7.1). Only an agent's run has turns, so the flag is
    refused on any other executor (section 4);
  - *listed:* once ended, it is indexed by its project and its label, so
    its project's lists find it. Every ended task a party requested is
    indexed by that party and its label, listed or not
    (`../protocol/store.md`, 4.4);
  - *its retention class,* what becomes of it once it has ended: *kept*,
    to the deployment's retention horizon, and whole while none is set
    (`../protocol/store.md`, 11.1); or *forgotten once read*, erased once
    its requester, a party, has read it. When and how is people.md,
    section 6.

  The label and the flags are what a task is made as: no amendment
  changes them (section 7).
- **Its times:** when it is made, when it last changed, when it ended,
  and its **last activity**: when it last heard words, its run last
  committed a turn, or one of its delegates last ended, and until then
  when it is made. Task queries order by them (engine.md, section 11).
- **Its holds:** the resources it writes and holds alone, or the pool
  slots it holds; or, while it waits for them, its place in their queues
  (section 6).
- **Its saved resources:** those a run of it left saved state on, which
  its next run starts from (hosts.md, section 7).
- **Its phase** (section 5) and why, if held; its **attempt number,**
  which only grows, fencing stale runs; and its **tries** per class of
  failure, which a release resets.
- **Its result,** once it has one.

What the store keeps beside it, and the engine loads only when wanted:
its transcript (engine.md, 7.2), its history (every change made to it,
by whom, and why), and the results of its ended delegates. While a task
that ended is named by a live one (a dependency's result not yet read,
an input, a reference), the core keeps a stub of it: its number, its
phase, and where its result is.

What a task **waits on** is not kept: the engine computes it when a
document shows the task, since only the engine holds the tree and the
inbox together (engine.md, section 11). It is a party, and who, for a
question, a proposal, an escalation or a person task in its tree; its
delegates; news; or nothing.

## 4. Batches

A batch is what one decision makes: one task or many, with the
dependencies among them.

- **Made by** an agent's `delegate` tool, a procedure's step, a party's
  `Create` (people.md, 5.1), an accepted proposal, or a recurring task at
  a period. Its tasks' requester is the task or the party that made it,
  or the accepter of the proposal that asked for them to be its
  (section 9).
- **A party's batch is an agent's.** Each task names the same things,
  whoever creates it: its executor, spec, result contract, authority,
  budget, deadline, wake policy, dependencies and inputs, the resources
  it will write, whether it is tracked and its priority, its label and
  its flags, and an agent task's LLM choice (section 3). Both pass the
  same checks; only the source of the creator's authority differs: a
  task's authority and what it has left, or a party's role and pool
  (authority.md, 8.1 and 8.4).
- **A party's root has someone to escalate to.** Every task a party
  requests is the root of a tree, whose escalations wait for that party
  or past them (section 9). A party becomes a root's requester only
  while its escalations have a holder (people.md, 5.1); otherwise the
  request is refused, whatever authority it fits.
- **Dependencies** name tasks in the same batch, or live tasks the
  creator reaches: for a task, its own delegates, or a task it is
  introduced to (8.5); for a party, a task in their tree, or any task of
  the project for a maintainer (people.md, 5.1). A task's dependencies
  are never added to once it is made (section 7). Admission checks the
  new edges with the existing dependencies and delegation waits, so that
  no reference makes a new task depend on an ancestor, or joins two
  subtrees into a wait cycle. A result of a task that has ended reaches
  a new task as one of its inputs, not as a dependency, within the same
  reach.
- **A budget as `AsAllowed`.** A creator gives each task's budget as an
  amount, or, for one task of a batch, as `AsAllowed`, which the
  authority child resolves as it checks the batch (authority.md,
  section 7); the task keeps the amount, and the batch's answer names
  it.
- **A deadline as a span.** A creator gives each task's deadline as a
  wall, as a span from the decision that makes the task, or none;
  authority makes a span a wall as it checks the batch (authority.md,
  section 7), and the task keeps the wall.
- **Checked whole,** before anything of it exists:
  - every task well formed: a known executor kind, a charter or
    procedure the deployment has, a result contract its executor can
    produce, its spec within its limits, its words and text parameters
    UTF-8 (section 3), the conversation flag on an agent's task only,
    and an LLM choice on an agent's task only, its models configured
    and its efforts among those its models declare;
  - at most one task giving `AsAllowed`;
  - no cycle;
  - within the limits: live tasks in the deployment, tasks in the tree,
    depth below its root, delegates of the creator, goals in its project
    (section 11);
  - within authority: each task's authority fits what the creator has
    left, its budgets carved from what it has left (authority.md, 8.1),
    or, for a party, fits their role and their pool (authority.md, 8.4);
    its LLM choice names models its authority allows, and an account
    only one whose uses serve its payer (authority.md, 8.1).

  A batch that fails a check other than authority is refused, saying
  which task and why, and nothing of it is made. One that fails only on
  authority is refused naming what it lacked, and whether a proposal
  could be accepted; its creator may propose it instead (section 9). It
  never becomes a proposal on its own, a party's no more than a run's.
- **Made at once.** Its tasks, their dependencies, the creator's carved
  budgets and the holds handed down commit together; its tasks with
  nothing left to wait for become active in the same commit.
- **Resources named for it.** A creator names the resources a task will
  write: resources it holds and hands down, the hold moving with the
  commit (6.4); resources named after the new task, which the batch
  resolves once the task has its number (authority.md, section 4); or
  any other its authority covers, which the task takes when it is ready
  (section 6). Which of those a task will hold, besides what is handed
  down, each connector says from its executor and spec before the batch
  commits, whoever creates it. So a party's batch names its tasks'
  resources by their specs and the grants named after them, and hands
  down none, since a party holds nothing (6.1).
- **Procedure tasks' parameters** go to their connector in the same
  decision, under the numbers the batch gives (connectors.md, 6.1).

## 5. Lifecycle

### 5.1 Phases

```
waiting ──► active ──► closing ──► done, failed or cancelled
   │          │           │
   └──────────┴───────────┴──► held ── released ──► back where it was
```

- **Waiting:** some dependency is not done, or some hold it needs is
  not yet free (section 6).
- **Active:** its executor is engaged (5.2 to 5.4). It enters active in
  the commit that makes it ready, which takes its holds.
- **Closing:** its executor has finished, failed, or the task was
  cancelled, and it is settling before it ends. In order:
  1. its run, if one is live, has ended: cancelled, and its answer
     heard;
  2. its delegates have closed and ended, deepest first, the follow-ups
     its result made among them, and the proposal its result made, if
     any, has been decided (5.6);
  3. every effect it asked for that went out has settled, made or
     failed, and those not sent are withdrawn if it was cancelled
     (connectors.md, 4.2);
  4. its connectors release the resources it held, as effects of their
     own, after all its others (6.5).

  Then it ends, and its result, or why it failed or was cancelled, goes
  to its requester. So a task that depends on another never starts
  before what that task did is visible and settled.

  What the core knows of a closing task is what it waits for: which of
  its delegates are still closing, and which connectors it waits on to
  settle its effects, by number. It never knows their outbox entries,
  which are each connector's, kept by the connector and settled by it
  (connectors.md, 4.2; core.md, section 5).
- **Done, failed, cancelled:** ended. Its result is a message to its
  requester, and kept. An ended task leaves the engine's working set once
  that message is committed, leaving a stub while live tasks name it;
  the store keeps it all.
- **Held:** stopped for a decision, from waiting, active or closing. It
  says why (5.5). A release returns it where it was, the reason it was
  held judged afresh.

### 5.2 Agent tasks

An active agent task is one of:

- **idle:** nothing to do until a wake (8.3);
- **due:** a run is wanted: the first activation, or a wake;
- **preparing:** its brief is gathering and, if it resumes, its
  transcript's current window loading (engine.md, 7.1). An amendment or
  a cancel that comes meanwhile drops what was gathered and decides
  again; a required section that cannot be gathered sends it back to due
  after a backoff, as no attempt was made;
- **claimed:** an attempt is committed, naming the run and its attempt
  number, the writer slots of the holds its workspace writes (6.3), and
  the messages its start carries (8.2), before the run is assigned;
- **running:** a run is live on a host. Its start carried the messages
  waiting; it hears those its policy lets through as it reads (8.2);
- **backing off:** an attempt failed and the next is due after a backoff
  (5.5).

A run ends by **finishing**, with a result the core checks against the
contract (5.6), or by **parking**, its transcript kept, the task idle
until its next wake. A task has one run at most at a time. Whether a
waiting run holds its host's slot or parks is engine.md, 7.4. A claim
that no host reports after a restart, its attempt having committed no
turn and no call, counts as not started: no try is spent on it.

### 5.3 Procedure tasks

An active procedure task is stepped by the part of the engine that owns
its procedure: a connector (connectors.md, section 6), or this child
domain for the core's (section 10). The tasks child hears a step's
decisions (effects asked, tasks made, amended or cancelled, proposals,
messages up, the result) as one decision, committed with the
procedure's state.
It steps when it is first active, when a message its policy lets through
arrives, and whenever what it reads changes, which its owner knows
(level-triggered).

### 5.4 Person tasks

An active person task is in the inbox of the party, or of the parties
holding the role, its executor names (people.md, section 8). Its result
is the answer one of them gives, checked against its contract. A person
task waiting past its stall (section 11) is news for its requester,
which may amend, cancel or re-address it.

### 5.5 Failures and holds

- **Tries, within a task.** A run or a step that fails is tried again
  after a jittered backoff, within a number of tries per class of
  failure:
  - *transient:* a peer unavailable, the engine's own reads missing, a
    transcript the agent could not use (engine.md, 7.2);
  - *permanent until something on a system changes:* a resource missing,
    an identity refused, an assignment its host found invalid;
  - *the run's own:* it reported failing;
  - *the agent's:* it crashed, broke its channel's rules, stalled;
  - *lost:* its host went away;
  - *invalid:* a result that broke its contract.

  A refusal before anything ran (a host busy, a writer slot taken) is no
  failure: the task claims again after a pause. Nor is a run ended
  because its LLM account's source turned limited, refused or in need of
  a sign-in: its task waits for its account, saying why (accounts.md,
  3.5 and 4.3). Every attempt, a failed one included, takes a new
  attempt number.
- **Past its tries, a task is held.** So it is when its executor stalls
  past its bound, a dependency fails or is cancelled, drift breaks a
  resource it relies on (connectors.md, section 11), an effect it asked
  for fails for good (a procedure's task hears it and steps instead,
  connectors.md, 4.2), an effect it asked for is uncertain and of a kind
  that cannot be recovered (connectors.md, 4.3), its budget or its
  deadline runs out (authority.md, section 7; for an agent task, too
  little left for its next run's reserve and one completion more of its
  main model, and on a program backend one more again, with nothing to
  free more), a party stops it, it waits for a hold past its
  bound (6.2), its pool lost its allocation (6.6), or its procedure says
  it needs a decision. A hold keeps whatever the task had: its result
  while closing, its inbox, its attempt number.
- **A hold is escalated** (section 9): it waits for the nearest task or
  party above it that may decide, which it wakes. Whoever it reaches
  decides it: releases the task from it, leaves the task held or passes
  it up. Releasing, amending or cancelling the held task is for those
  section 7 names, wherever its escalation waits.
- **A release lifts what it releases:** tries count again from it, and
  what held the task is judged afresh; a deadline still past holds it
  again, with a fresh escalation.

### 5.6 Results

- **Checked twice.** An agent's run checks its result against the
  contract before it finishes (smith's `domain/run.md`, section 7); the
  core checks it again as it commits, against the contract and the state
  as it is now. A result that breaks its contract is a failed attempt,
  its reason given to the next run.
- **What a result may carry** beyond its words: effects, through the
  tools that ask for them; follow-up tasks its verdict's contract allows,
  as one batch of the task's own delegates, made in the commit that takes
  the result. Follow-ups beyond the task's authority become one
  proposal, made as the result commits; the result stands either way.
- **Follow-ups are waited for.** The task, closing, waits for its
  follow-ups as for any delegate (5.1), and for the proposal its result
  made to be decided: accepted, its tasks are delegates like the others;
  rejected, there is nothing more to wait for. Closing does not withdraw
  that proposal (section 9); cancelling the task does. So its requester
  hears its result, and its dependents start, once what the result asked
  for has been done or refused.
- **A task ends after its delegates.** An executor that finishes while
  delegates of its task are live is refused, naming them, unless its
  finish asks for them to be cancelled, so a task that depends on another
  waits for that task's whole subtree. Its result goes to its requester
  once it has closed (5.1).
- **Delivered once,** as a message to the requester, committed with the
  task's end. A party requester sees it in their inbox.
- **Kept.** A result is read by every task that depends on its task, or
  names it among its inputs, in the brief of its next run (engine.md,
  section 9), and by its requester as a message. A connector's result is
  read through its connector's section of that brief.
- **Failure is a result.** An executor that finds the task cannot be done
  as asked ends it failed, saying why. A failed task's dependents are
  held (5.5); its requester decides what follows.

## 6. Holds

### 6.1 What a task holds

- **A write hold** is a resource a task writes alone: a branch a change
  is pushed to, a service being remediated, an environment it provisioned.
  Its connector says which kinds of resource take holds (connectors.md,
  3.3).
- **A pool slot** is one of a pool's counted slots: an environment from a
  pool of five, a share of a quota. Its connector names the pool, a
  resource of its own, and says how many slots it has, which may change
  as its facts do (6.6).
- **Delegates write under their holder's hold.** A procedure's producing
  and repair runs write the resource their procedure's task holds.
- **Shared resources are not held:** writing into them is ordered by
  their connector's mechanics (a landing queue), not by a hold.

### 6.2 Taking holds

- **All at once.** A task takes every hold it needs in the commit that
  makes it active, or takes none and waits. A task never holds one while
  waiting for another, so no set of tasks waits in a circle.
- **Refuse or wait.** For each kind of resource, its connector says
  whether a hold that is taken refuses or waits. A batch whose task needs
  a hold that refuses is refused, naming the holder. A task whose hold
  waits is made, and waits in the resource's queue.
- **Queues are ordered** by the priority of the goal each task serves
  (core.md, 6.2), then by when it began to wait, and bounded per
  resource. A task waiting past its bound is held (5.5), and its
  requester hears why.
- **Freed holds go to the queue** in the commit that frees them: the
  first waiter whose every hold is now free takes them all and becomes
  active. A pool slot is free only while the pool has fewer holders than
  the slots it is known to have.

### 6.3 Writer slots

Each write hold has one writer slot: the one run that may write it at a
time.

- **A run takes the slot** when it is claimed, whether its task holds the
  resource or writes under a task above it that does, and frees it when
  its answer is committed, or, when its attempt is settled as lost past
  the engine's grace (engine.md, section 8), once the connector has read
  the resource afresh, so that what a lost attempt wrote is known before
  anyone writes again.
- **A claim that finds the slot taken waits,** which is no failure.
- **A connector's own effects** on a held resource take the slot too,
  while they are in flight, and wait for it like a claim.

### 6.4 Handing down

A creator may hand a resource it holds to a task it makes: the hold
moves in the batch's commit, and the writer slot stays with a run that
has it until that run's answer, so the new holder's first writer waits
for it. A chat whose run wrote to a resource hands it to the procedure
task that finishes the job.

### 6.5 Release

Holds are released as the task closes (5.1), after every other effect of
the task has settled. What becomes of the resource is its connector's to
say, per kind and per way the task ended, as effects of its own: an
environment provisioned for a task that ended is torn down; a branch a
change landed from is deleted; one a failed change left is kept for a
person to read until its tree's root closes.

### 6.6 Pools that shrink

A pool's slots are its system's to say, and may fall below the number of
tasks holding it: a quota cut, a machine gone.

- **Holders keep their slots.** Nothing the core admitted is taken back
  by a fact: the tasks holding the pool keep their holds.
- **The pool is overcommitted,** and admits no holder, from its queue or
  from a batch, until its holders are fewer than its slots.
- **A holder whose allocation is gone** is told by the connector, as
  drift (connectors.md, section 11): its task is held, saying so, and its
  release frees its slot.
- **The promise is about admission:** no holder is admitted beyond the
  slots the pool was known to have when it was admitted.

## 7. Amending, cancelling, moving

- **Who may:** a task's requester, and any task above it in the tree; a
  party in whose tree it is, or whose role in the project allows it
  (people.md, 5.1). Nobody else. An escalation's holder decides the
  escalation (5.5; section 9). A party's request, an agent's tool and a
  procedure's step reach the same rules (people.md, 5.1; engine.md, 7.3;
  connectors.md, 6.3).
- **What an amendment may change:** the spec and its parameters; the wake
  policy; the dependencies of a task still waiting, by removing some,
  never adding; the authority, narrowed freely and widened only within
  the amender's own (beyond it, refused, and proposed if the amender asks:
  section 9), the amender, or the proposal's accepter, funding the
  widened task from then on (authority.md, section 7); an agent task's
  instructions; a procedure's parameters, which go to its connector; a
  person task's address, to another party or role, naming the person
  task's revision as an amendment names the spec's, a party who took it
  holding it no more; whether it is tracked, within goals per project
  (section 11), and its priority; an agent task's LLM choice, within the
  amender's authority as a batch's is (section 4). An amendment is
  refused once a task is closing. Its label and its flags are never
  amended (section 3).
- **A new LLM choice is for the next run.** A run in flight keeps its
  own models and account; the next run's check uses the new choice and
  chooses its account again when its main model's LLM provider changed
  (accounts.md, 4.3). A conversation that moves to another LLM provider
  or another account is carried over, its text, calls and results kept
  and the provider's opaque replay dropped (smith's
  `protocol/transcript.md`, section 4). An amendment naming another
  account of an LLM provider whose account the task keeps is refused
  while that account is limited (accounts.md, 4.3; providers.md, 2.5 and
  3.4).
- **Tracking and untracking.** A task an amendment tracks is a goal from
  that commit, with all a goal has (core.md, 6.2). One it untracks is no
  goal from that commit: the core ends its goals' topics, and its
  projections write their last, as when a goal closes (connectors.md,
  section 8). Neither touches its flags, so a conversation stays one:
  tracked, its turns are still entries for its requester, with their
  read position and unread count (section 3; people.md, section 6). No
  tracking is refused for a task's being a conversation.
- **A decision names what it saw.** An amendment names its task's spec's
  revision, and a release its hold's (5.5), whoever makes it: a party,
  as their client showed it (`../client/domain/contract.md`, 5.2); a
  run, as it last read it (engine.md, 7.3); alone, or in a compound
  beside words or an amendment. One whose revision has moved is refused
  as moved, saying what it is. No message releases a task: words to a
  task its party stopped come with a release, which names the hold's
  revision as any release does (people.md, 5.1). A priority moves no
  revision, and an amendment of the priority alone names none: the
  first commit decides (people.md, 5.1). A cancel, a move and a stop
  name none.
- **Instructions are replaced whole.** An agent task runs with its
  charter's instructions until an amendment gives it its own. An
  amendment's instructions replace the task's whole instructions, and
  empty ones leave it with none, not with its charter's; an amendment
  that leaves them out changes nothing of them. Its history records the
  instructions as the task held them, its own or none of its own, never
  which charter's text was in effect.
- **An amendment is a message** to the task's executor, committed with
  the change: it wakes an agent, steps a procedure, and shows a person
  what changed. A run already live hears it as it arrives; a narrowing it
  cannot honour (a grant its live run is using) cancels that run, and the
  next starts with the narrower authority.
- **Cancelling closes down the tree.** A cancelled task, and every task
  below it, is closing (5.1), each with the same reason: live runs are
  cancelled, pending proposals withdrawn, outbox entries not yet sent
  withdrawn, and those in flight settle, since what may have been made
  counts; a task waiting for holds leaves their queues. Each task ends
  once its own closing is done, deepest first, and its requester hears
  that it ended cancelled, with what it had done.
- **Cancelling with a grace.** A cancel may give a grace, within the
  longest the tasks child allows (section 11), so that a task finishes
  what it can before it is cut off: a conversation closed by its person,
  a run asked to report what it has. The task hears that it is asked to
  finish, as it hears an amendment (8.1): an agent's run is woken by it,
  and a procedure's connector and a person task's holder hear it as they
  hear an amendment. It is cancelled, as above and with the cancel's
  reason, when the grace ends, unless it ended first. The grace's end is
  kept with the task, on a timer of the tasks child's, so a restart
  keeps it. A task that cannot finish on its own, waiting or held, is
  cancelled at once. A cancel made during a grace ends the task no later
  than that grace would: one without a grace cancels it at once. One with
  a grace asks nothing more, since a task is asked to finish once, by
  the first. If its grace ends sooner, the task's grace now ends with
  it, and the task is then cancelled with this later cancel's maker and
  reason, since that cancel is what ends it. If its grace ends no sooner,
  nothing of the task changes.
- **Moving a task.** A party may make themselves the requester of a live
  task they may amend, so that it outlives the task that asked for it,
  as a goal outlives the chat it came from. Its reservation moves to
  their funding (authority.md, section 7), and its next run keeps its
  LLM account only if the account serves them (accounts.md, 4.3); the
  task that asked for it keeps a reference to it, and does not wait for
  it as for a delegate. It becomes the root of a tree of the party's, so
  a move needs their escalations to have a holder, as their batch does
  (section 4).
- **Several operations, one decision.** A party's compound request
  (people.md, 5.1) cancels, moves, amends, releases, messages and stops
  in one decision, all or none: so a conversation is closed in one
  decision, each of its live delegates cancelled or moved to its party,
  then itself cancelled with a grace; a project's goals are reordered at
  once; a held task is amended and released.

## 8. Messages

### 8.1 What a task hears

| Message | From | Woken by default |
|---|---|---|
| a result: done, failed or cancelled | a delegate | the last of its delegates; any failure |
| a question, expecting an answer | a delegate | yes |
| an answer | its requester | yes |
| a decision on its own proposal | the holder that decided | yes |
| an amendment; a cancel's grace, asking it to finish (section 7) | its requester, a task above it, or a party | yes |
| a cancel | the same | always |
| words | a party; a task holding a reference to it | a party's, yes; a task's, by policy |
| news: a connector's event, classified | its subscriptions (8.4) | by policy |
| a notice: a task it references done or held; a proposal passing through; a hold it waits for | its subscriptions; the core | by policy |
| a timer | itself | yes |

Proposals and escalations waiting for its decision are not messages: they
are entries of its own (section 9), which always wake it.

### 8.2 The inbox

- **Durable.** A message is in the inbox once the commit that sent it
  is; it is taken once the commit of the run's turn, the procedure's
  step or the party's view that read it is. A run that dies without
  committing what it read leaves it for the next.
- **Bounded, with every message admitted at one entrance.** An inbox
  holds at most a configured number of messages and bytes, and every
  kind of message is admitted in one of three ways:
  - *room kept:* one result from each live delegate, one answer to each
    open question it asked, one decision on each of its pending
    proposals, and a cancel, its grace's ask to finish taking the same
    room, one ask in a task's life (section 7), so none of them can be
    refused;
  - *merged:* news of one subscription with the news of it already
    waiting, since news is a hint to read afresh; notices likewise; a new
    amendment with one still waiting; timers;
  - *refused at the sender's entrance:* words and questions, past the
    bound: a party is told the inbox is full, an agent's tool call
    answers so.
- **Within what a run reads.** Every message an inbox admits fits a
  run's message bound as it is rendered (smith's `protocol/limits.md`,
  3.9), so no run refuses one as too large: words and questions larger
  are refused at their sender's entrance as too large, never cut (a
  party's by the people child, whose bound is at most the run's;
  people.md, 5.1), and every other kind fits by its own bound.
  Configuration checks both against the deployment's message bound
  (`../protocol/configuration.md`, section 6), and the fleet admits no
  host whose agent takes less (engine.md, section 8).
- **Carried whole and read oldest first,** within the run's inbox. A
  run's start carries the messages waiting in its task's inbox, each
  whole, oldest first, as many as the run's inbox takes (smith's
  `protocol/limits.md`, 2.3); a person's words are never cut. The rest
  wait in the inbox, in order, and are relayed as the run reads, as what
  its policy lets through while it runs is, never more relayed and
  unread than the run's inbox takes (engine.md, 7.1 and 7.4).
- **Refused or unread, a message stays.** A message the run refused (too
  large, its inbox full, a name in use, or the run ending), or left
  unread (after the answer's fence, the last message it read), stays in
  the inbox, untaken, and wakes the task again when the run ends, as it
  would had it arrived then (8.3).

### 8.3 Wake policies

A wake policy is data, from a closed vocabulary:

- **for each kind of message** (8.1): whether it wakes the task, or waits
  for the next activation;
- **for news,** the class the connector gives each event for the task:
  *wakes*, *kept* (news, but nothing the task must act on now), or
  *dropped* (not the task's concern after all, and not kept). The task's
  policy may lower a class, never raise it;
- **batching,** for kinds that allow it: wake once N have arrived, or
  the oldest is older than T, so a watch hearing a storm of alerts wakes
  once. A person's words, a cancel and an amendment are never batched,
  and never wait behind what a batch holds;
- **timers:** wake at a time, or every period.

A proposal or an escalation waiting for its decision always wakes a
task, as a cancel does: no policy may keep a decision from whoever must
make it.

A message a run refused or left unread (8.2) is judged again when the
run ends, as if it arrived then: it wakes the task if its kind does, and
otherwise waits for the next activation.

Each charter brings a default policy, which the task's creator may
narrow or widen: a chat wakes on its person's words and on its
delegates' results; a coordinator on questions, failures, the last of its
delegates done, a person's words and news its connectors class as
waking; most other agents on nothing, since they finish within one run.
A procedure's policy is its owner's: it steps on whatever it reads.

### 8.4 Subscriptions

A subscription is a task's standing interest in what changes:

- **a connector's topic,** such as a service's alerts or the landings on
  a branch: kept by the connector, which names the task by its number and
  classifies each event for it (connectors.md, 5.3);
- **another task's state:** done, held, or its result; for a task it
  references;
- **a timer.**

Subscriptions end with the task. A goal's subscriptions to the topics its
connectors offer for goals are the core's to keep, not its executor's
(core.md, 6.2). Their number per task is bounded.

### 8.5 Who may message whom

- **Along delegation:** a requester and its delegates, both ways.
- **By reference:** a task may message any task it holds a reference to:
  its requester, its delegates, and one it was introduced to. A task
  introduces two tasks it references by giving each a reference to the
  other, within their limits. References are kept with the task, and end
  with it.
- **Parties** message the tasks their role lets them see (people.md,
  section 5).
- **No broadcast.** Who may talk to whom stays visible and bounded.

## 9. Proposals and escalations

A proposal is an action a task or a party may not take alone, waiting
for a holder of the authority it needs; an escalation is a held task
waiting for a decision. authority.md, section 9 says what may be
proposed, what an action needs, and what an acceptance funds.

- **Proposing is asked for.** Nothing becomes a proposal on its own: a
  batch or an amendment beyond its maker's authority is refused, naming
  what it lacked and whether a proposal could be accepted (section 4),
  and its maker proposes it, with reasons: a run with its `propose`
  call (engine.md, 7.3), a procedure in its step (connectors.md, 6.3), a
  party with a `Propose` request (people.md, 5.1). So a person is never
  asked by accident, by a program or by their own client. The one
  exception is a result's follow-ups beyond its task's authority, which
  its verdict's contract allows, and which become one proposal as the
  result commits (5.6).
- **What each may propose.** A task proposes any action: a batch, an
  amendment, an effect, or a widening of its own authority. A party
  proposes a batch or an amendment: a party makes no effect, and their
  authority is their role's, which the project's owners change
  (people.md, 5.2).
- **Their state.** A proposal: its proposer, a task or a party; the
  action, as the call, the step or the request asked for it; its reason,
  in words;
  whether the tasks it makes are to be the accepter's; where it waits;
  and pending, accepted, rejected or withdrawn. An effect's own
  description stays with its connector, under the proposal's number
  (core.md, section 5). An escalation is part of its held task's state:
  one per held task, never more. Both are kept in the store, and pending
  ones in this child domain.
- **Admitted once.** A proposal is admitted at its entrance, the
  `propose` call, the step or the party's `Propose` that asks, against the
  pending proposals allowed per proposer and in the deployment, and
  refused there if they are full; an escalation needs no room beyond its
  held task. Neither takes room in any inbox: each shows as an entry in
  the inbox of whoever it waits for, and those it passes on the way hear
  a notice, which merges.
- **Where they wait.** At the nearest task above the proposer, or the
  held task, whose authority covers the decision (authority.md,
  section 9), skipping procedures, which decide nothing. Above the tree's
  root task is whoever requested it: a party is the next holder there,
  when their authority covers it; past them, or past a root the
  deployment requested, it waits for the parties whose role the project's
  policy names for that kind of decision (people.md, section 4). A
  party's own proposal waits there from the start. Where one waits is
  decided again when that holder ends, is held, or no longer covers it,
  and when it stalls.
- **A proposal nobody may accept is refused.** A proposal is never
  committed to wait for nobody. When the first holder it would wait for
  is the parties the project's policy names (a party's own proposal; a
  task's, when the deployment requested its tree's root, as for a
  standing procedure's or a recurring task's tree, or when nothing in
  its tree covers it) and the policy names no role that may accept its
  kind, it is refused to its proposer as beyond authority, saying that
  no proposal could be accepted, and nothing of it is committed. A
  party's request is refused so at admission (people.md, 5.1); a run's
  `propose` call is answered so; a procedure's step's proposal is
  refused in its decision. A proposal already pending waits as this
  section says.
- **Waking.** It always wakes the task it waits for (8.3), is relayed to
  that task's live run as messages are, and is in the brief of its next
  run (engine.md, section 9); for a party, it appears in their inbox at
  once.
- **Deciding.** A holder accepts, rejects with a reason, or passes it up
  to the next holder that covers it. An accepted proposal is made as the
  proposer's action, with the accepter's authority funding what it
  needs, in the commit that accepts it; the tasks it makes are the
  proposer's (a task's delegates, a party's roots), or, when the proposal
  asked, the accepter's (a chat proposing a goal asks so); a task it
  amends or widens is funded by the accepter from then on, whoever
  proposed it (authority.md, section 7). A rejection is
  a message to the proposer: a task hears it in its inbox, and a party
  finds it in theirs (people.md, section 6). A party's proposal to amend
  a task is the party's, not that task's. A party who does not cover a
  proposal may reject it or pass it up, never accept it. At the parties
  the policy names there is no one further: they accept or reject, and
  the proposal waits for them, however long. Passing a proposal up where
  nobody further may accept it is refused, saying there is no one
  further.
- **Stalled** past its bound, undecided, a proposal or an escalation
  passes up to the next holder that covers it, and past the last, waits
  for the parties the policy names.
- **Withdrawn** when the proposer withdraws it (a run's call, a
  procedure's step, a party's `Withdraw`), when a proposing task closes,
  or when the state it was made against has changed so that it no longer
  applies (a batch naming a task since ended); an escalation ends with
  its hold. The
  proposal a task's result made is the exception: its closing waits for
  it to be decided (5.6), and only a cancel of the task withdraws it.

## 10. Core procedures

Procedures the core owns, stepped by this child domain:

- **Recurring.** A task whose procedure makes the batch it was made
  with, its recurring batch (never one of the configuration's
  templates, core.md, 7.3): at each period, as the period reaches the
  core from the application's root, it makes its recurring batch as its
  own delegates, within its authority, its budget carved again each
  period from the project's spend for that period (authority.md,
  section 7). If the last batch is still live, the period passes, or the
  new batch waits, as its maker chose. A recurring task ends when it is
  cancelled.

The core has no other procedure. A static plan is a batch; waiting for
several tasks is a dependency; anything that needs a condition over
results is a coordinator's judgement. A standing task that makes tasks
from news is a connector's procedure (connectors.md, 6.6). Both kinds of
standing task have their allotment of tasks and spend renewed each
period (authority.md, section 7).

## 11. Limits

Each is configured, refused at the entrance, and part of the engine's
worst case (engine.md, section 13):

- live tasks in the deployment, and per project, and the stubs of ended
  ones;
- goals per project: the live tracked tasks in one project, refused at
  the entrance like live tasks per project, whether a batch, an
  amendment or an accepted proposal tracks them;
- tasks per tree, and the depth of a tree;
- delegates per task, and batch size;
- a spec's and a result's bytes, and inputs per task;
- messages and bytes per inbox, and a message's bytes, within what a run
  reads (8.2);
- subscriptions, references, holds and saved resources per task;
- waiters per resource's queue, and how long a task may wait for holds;
- pending proposals per task, and in the deployment; open questions per
  task;
- tries per class of failure, and their backoff;
- the stall of a person task, of a pending proposal or escalation, and
  of an agent task idle with delegates still live;
- a grace's span: the longest grace a cancel may give.

A live task is never evicted: when the deployment is full, new batches
are refused, and the creator hears why.

A compound request is bounded by the people child's operations per
request (people.md, 5.1), and its worst case is engine.md, section 13's.

## 12. The world

The tasks child domain has a world of its own: its parent, the
executors, the connectors' news, the holds' kinds and the authority
checks scripted, its expectations in a referee (testing.md, section 3).
Its stories: a batch made whole or refused whole; a plan of reports, a
choice and procedure tasks, run in dependency order; a delegate failing
past its tries, held, escalated two levels to a person, released and
finished; a watch woken once by a burst of news; a cancel closing a tree
of three levels with runs live and effects in flight; an amendment
reaching a live run; a person's words left unread as a run ended, waking
its task again; a proposal routed past a procedure to a person and
accepted, and one that stalls and passes up; a chat's goal accepted as
its person's, outliving the chat; an inbox filling, its news merged,
words refused, a result still taken; two tasks wanting the last slot of
a pool, the second waiting and taking it as the first closes; two tasks
each wanting two holds the other wants, neither waiting in a circle; a
pool shrinking while its slots are held and a task waits, admitting
nobody until it drains; a standing task making delegates across many
periods, its allotment renewed each time; a verdict whose follow-ups
go beyond its task's authority, their proposal accepted and the
follow-ups run before the result reaches the requester, and another
whose proposal is rejected, its result delivered at once.

Its stories of what parties and agents ask alike: a party's batch of an
agent's task and a person task, made as an agent's would be; a party's
batch beyond their role refused, naming what it lacked, then proposed
and accepted; another party's proposal rejected, the rejection reaching
that party, not a task; a party refused a root while their escalations
have no holder, and made once the policy names one; a budget given as
`AsAllowed` beside named amounts, resolved to what the pool has left
after them and named in the answer, one that comes to nothing refused,
and a batch where two tasks give it refused; an agent's amendment and a
party's release each named with a revision that has moved, refused as
moved; a party withdrawing their proposal; a goal tracked past goals per
project, refused at the entrance, and one untracked, its projection's
last write made; a conversation closed in one compound request, one
delegate moved to its party, another cancelled, and the conversation
cancelled with a grace in which its run finishes, ended done; another
whose grace ends first, cancelled; one compound request refused whole
when its third operation fails, nothing of it made; a project's goals
reordered in one compound request, its priorities naming no revision; a
task's last activity moving with words, a turn and a delegate's end; a
conversation forgotten once its party has read it, and one kept.

Its referee: a task starts only after its dependencies are done and its
holds taken; batches are made whole or not at all, and compound requests
too; one run at most per task; a resource has one holder, and a pool
admits no holder beyond the slots it is known to have;
no task holds one resource while waiting for another; every result
reaches its requester once, after the task has closed; a cancel ends
every task below it, deepest first, and one with a grace ends its task
by the grace's end; no amendment or release is made against a revision
that moved; nothing becomes a proposal unasked but a result's
follow-ups; no committed message is lost, and none is refused once
committed; every proposal and escalation waits where a holder can decide
it; nothing beyond a task's limits is held, goals per project
included.

## 13. Open questions

- **Wake defaults:** which kinds wake each charter's tasks, and how
  batching is tuned; whether a cheap model should triage news before a
  coordinator wakes. Tuned from use.
- **Dependencies on a failure:** whether a task should be able to depend
  on another's ending, whatever its result, for clean-ups; until wanted,
  a requester makes the clean-up task itself.
- **Adding dependencies by amendment,** which the model forbids so that
  the cycle check stays local to a batch.
- **Priorities among tasks that are not goals,** when hosts or pools are
  scarce: they are ordered by when each became due.
- **Pools across projects:** whether a pool's slots are shared fairly
  between projects, or by goal priority alone.
- **A pending proposal whose holders go:** one that waits for a policy's
  roles when the policy stops naming any for its kind, or that stalls
  past its last holder: who may decide it then, or whether it is
  withdrawn, saying why.
