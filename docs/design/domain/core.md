# The model

Provisional, 2026-10-07, revised 2026-10-23. What jig's domain is
beneath its parts: an engine core that runs work done by agents,
procedures and people, keeps it in its own store, and drives external
systems through an application's connectors. Every other document in
this directory builds on it, and it is the first to read (README.md
gives the order and the names). The mechanics are those of skein's
`programming-model.md`. What is still open is listed in section 12.

## 1. In one page

- **A generic agentic engine.** jig runs work: it keeps what was asked,
  who does it and what came of it; it runs agents on hosts, steps
  procedures, asks people, and routes to each what it needs to hear. It
  is to be as flexible as a coding agent at a terminal, with what such an
  agent lacks: work that outlives processes and machines, many agents at
  once, rules no agent can loosen, and effects on other systems made
  once.
- **Five primitives** (section 3): the **task**, what is asked; the
  **executor**, who does it: an agent, a procedure or a person; the
  **message**, everything a task hears once it has started;
  **authority**, what a task may do, narrowed whenever it delegates; and
  the **connector**, an external system. Everything else is built from
  them: plans, coordinators, chats, goals, reviews, incidents.
- **Structure belongs to the work.** A plan is the tasks one task has
  delegated, and a coordinator is an agent whose charter is to delegate
  and supervise (section 7). Agents make structure with tools, as a
  coding agent spawns sub-agents; the core makes it durable, schedules
  it, and holds it to its authority.
- **The core owns its state.** Projects, tasks, their inboxes and
  results, transcripts, proposals, notes, the outbox and the settings'
  changes live in the application's store, which only its engine reads
  and writes. The engine
  holds bounded working sets, rebuilt from the store and the connectors
  after a restart. A connector's system keeps what is the system's.
- **A decision commits once** (section 4): its change of state and the
  effects it asks for, in one commit. Nothing leaves the engine before
  the commit it follows from is durable. Effects are made from the
  outbox, keyed, and each kind of effect declares how one whose outcome
  is uncertain is resolved, which is what its promise of once rests on.
- **The owner keeps the value** (section 5). The core holds what it acts
  on, and nothing of a connector's vocabulary: it names a connector's
  values by token and size, and the application's root puts each output
  together from its owners' parts.
- **External systems are the application's connectors.** Each gives
  resources, effects, events, procedures, requirements' verdicts, reads
  and brief sections, and the core knows none of them but by number
  (connectors.md). The application's root knows them all.
- **What the application does on a system belongs to it** (section 8).
  It writes only to what it owns, and takes part as one participant in
  what others own. Anything else that changes is drift, found by reading
  afresh and reconciled, never coordinated with.
- **People are parties** (section 9). They talk to agents, set goals,
  accept proposals, answer questions, steer and watch, through a client.
  jig's API is the primitives, as plumbing; a shape of work such as a
  chat is sugar over them, built only from plumbing.
- **Mechanics are code, judgement is an LLM's.** Procedures carry out
  what needs no judgement. An LLM is woken for what does.
- **Settings change at runtime** (section 13). Charters, templates, the
  policies projects are made from and the rules are the binary's
  defaults with the store's changes over them; a change is one decision,
  and makes a revision. New work takes the newest; a task's runs keep
  the charter of the revision it was made in, until it moves.
- **Bounded, and complete in its domain.** What the engine holds grows
  with the work in progress, never with history. Every entity has a
  limit, refused at the entrance. A world of domains and fakes runs
  everything the core does, with no protocol and no io
  (programming-model.md, section 4).

## 2. The system

```
people    a client: chats, goals, proposals to accept, questions to answer, watching
engine    the application's process: its root, jig's core, its connectors; the store's only client
store     the application's durable storage: jig's records and its connectors'
hosts     where agents run: inside the engine, or on workers that dial it
agents    smith: LLM work, one run at a time per host slot
systems   what the application's connectors drive
```

- **One engine per deployment.** A deployment is one engine, its store,
  its hosts and its configuration, with an id made at its first start.
  The engine is the only writer of the store and the only client of
  every connector's system, so it never needs a compare-and-swap against
  itself. Scaling out is open (section 12).
- **The engine decides; hosts host; agents think.** Agents reach the
  engine only through tools their host relays (hosts.md).
- **The store is assumed.** It keeps durably what the engine gives it,
  commits several changes as one, applies commits in order, and answers
  bounded reads (engine.md, section 5).
- **People reach the engine through a client** of the application's,
  built on jig's, `jig-client` (people.md; `../README.md`, section 10).

## 3. The five primitives

### 3.1 Task

A unit of work someone asked for (tasks.md):

- **its spec,** in words and in typed parameters, and its **result
  contract**, which says what counts as done (tasks.md, section 3);
- **its requester:** the task, the person or the deployment that asked
  for it. Requesters link tasks into a tree, rooted at a party or at the
  deployment;
- **its dependencies:** the tasks it starts after. A dependency means
  only "after it is done", and its result reaches the task: an agent's
  brief carries it. Dependencies link tasks into a graph without cycles;
- **its executor** (3.2), **its authority** (3.4) and **its wake
  policy** (3.3);
- **its write holds:** the resources it writes alone (tasks.md,
  section 6);
- **its lifecycle:** waiting, active, closing, then done, failed or
  cancelled; held for a decision at any point before it ends;
- **its result,** delivered to its requester as a message, and kept.

Tasks are created in **batches.** A batch, with the dependencies among
its tasks, is checked whole and made at once, so that a plan is never
half made.

### 3.2 Executor

Who carries a task out. A task has one:

- **An agent:** an LLM, set up by a charter: instructions, models,
  tools, the resources its workspace holds, the outcome it must produce.
  Its transcript belongs to its task and is kept turn by turn in the
  store. It is activated as a run on a host, which ends by parking or
  finishing; between runs, nothing of it is live (engine.md, section 7).
- **A procedure:** engine code, pure: given the task's state, what it has
  heard and the facts it reads afresh, it says which effects to make,
  which tasks to create, amend or cancel, and whether it is done.
  Procedures belong to connectors (connectors.md, section 6), or to the
  core, such as a recurring task (tasks.md, section 10). They cost no LLM
  turns.
- **A person:** a party who answers through a client: a choice, a
  decision, an approval, an answer (people.md, section 8).

All three can delegate, each within its authority: an agent with its
tools, a procedure in its code, a person through the client.

### 3.3 Message

Everything a task hears once it has started:

- results and questions from the tasks it delegated;
- amendments, cancels, answers and decisions from its requester;
- a person's words;
- news from what it subscribed to: a connector's events, another task's
  state, a timer.

What waits for its decision, a proposal or an escalation from below, is
not a message but an entry of its own, which always wakes it (tasks.md,
section 9).

Each task has an inbox, durable in the store, and a **wake policy**,
which decides when its executor runs for what has arrived: a run for an
agent, a step for a procedure, a notification for a person. A wake costs
an agent an LLM turn, so news is kept, unless its connector finds it is
not the task's concern, and only some of it wakes. What does not wake a
task waits in its inbox, and its next run hears it (tasks.md, section 8).

### 3.4 Authority

What a task may do (authority.md):

- the tools its agent has, and the effects and reads it may make, on
  which resources: restart which services, write which branches, create
  what where;
- what it may spend: LLM completions and priced effects, in the
  deployment's unit, and time;
- what it may delegate: which kinds of task, how many, how deep, with
  what authority.

**Delegation only narrows it:** a task gives its delegates at most what
it has, and what it gives of its budget is no longer its own to spend.
The root of every tree is the deployment's rules and the project's
policy, which no task loosens.

**Beyond its authority, a task proposes.** An action beyond its
authority is refused; it becomes a proposal when the task proposes it,
to whoever holds the authority, up the tree: a task above it, or a
person (tasks.md, section 9). They accept it, and it is made with their
authority, or reject it, and the task hears why. Acceptance, escalation
and widening a plan's reach are all this one mechanism.

### 3.5 Connector

An external system, as the engine sees it (connectors.md):

- **resources:** what has an identity there (a service, an environment,
  a repository, a branch), named by paths, which a task names and a
  workspace may hold. A resource a task writes is held by that task
  alone while it writes, or waits its turn for a pool's slot;
- **effects:** what the application does there, made through the outbox
  (section 4), and priced when they cost;
- **events:** what changed there, delivered to the tasks that
  subscribed. They are hints: the state they point at is read afresh;
- **procedures:** the mechanics of working there, as executors (3.2);
- **requirements:** the facts an effect needs, judged by the connector
  whose facts they are, as a verdict;
- **reads and brief sections:** what an agent may read of it, and what a
  brief carries about it.

The set of connectors is closed in each application: each is a child of
its root, which knows them all. The core knows them by number, and its
tasks, inboxes and authority know none of them.

## 4. Decisions, the store and the outbox

- **A decision commits once.** A run's answer, a procedure's step, a
  person's request, a tool call: each is decided against the engine's
  state, and what it changes in every part it touched (tasks made, a
  result kept, an inbox read, a proposal accepted, an outbox entry
  added) and the effects it asks for go to the store in one commit.
- **Nothing leaves before its commit.** An effect is made, a run is
  assigned, a host's answer is acknowledged and a person is answered only
  once the commit it follows from is durable. The engine's state may run
  ahead of the store by the commits in flight, which are bounded; if the
  engine dies meanwhile, nothing anyone saw depended on what was lost,
  and a restart starts from the last commit.
- **The journal holds it.** The application's root gathers each
  decision's writes and outputs into one decision of skein-lib's journal,
  which numbers the commit and releases the outputs once it is durable
  (root.md). The core decides what each output waits for; the journal
  keeps it waiting.
- **The outbox makes each effect, keyed.** An effect committed is an
  entry in its connector's outbox, with a key derived from what the
  effect is for, never from when it was asked. Its connector makes it.
  How one whose outcome is uncertain is resolved depends on what the
  system offers for its kind: a key the system keeps, a condition it
  checks, a write that may be repeated. An effect with none of these is
  never repeated on the engine's own: its task is held for a person
  (connectors.md, 4.3).
- **Facts are read afresh.** What a connector's system owns is read again
  before a decision that depends on it, as level-triggered state. Events
  are hints that something changed, never the state itself.
- **A restart reads the store,** then has each connector read afresh what
  its system owns of the live work, and look for what uncertain effects
  may have made, before deciding anything new. The core runs it as a
  script (engine.md, section 6).
- **A commit that fails stops the engine.** The store is assumed to do
  what the engine needs; when it cannot, the engine stops rather than act
  on state it could not keep, and a restart starts from the last commit.

## 5. The owner keeps the value

The core and the application's connectors are siblings under the root,
and siblings share no types (programming-model.md, 4.5). So:

- **The core holds what it acts on,** in its own vocabulary: tasks,
  inboxes, authority, budgets, holds, transcripts as bytes it never reads
  inside, its tools' records.
- **What it orders but does not read, it names.** A brief section a
  connector gathers, the answer to a connector's read, a procedure's
  parameters, a connector's result, what a run left on a connector's
  resources, a workspace's items: each stays with the connector it
  belongs to, keyed by the task's number, a run's attempt, a call's name
  or a token. The core holds the key and, where it budgets, the declared
  size.
- **The root puts outputs together.** An assignment, a call's answer, a
  brief: each is made, in the step the core decides it, from the core's
  part and the parts each connector hands over, and goes into the
  journal whole (root.md, section 7).
- **Values arrive typed,** decoded by the application's protocol layer
  into the vocabulary of whoever owns them: an agent's call for a
  connector's read reaches the root already as that connector's request.

This is skein's binding rule (programming-model.md, 4.2: bindings are
echoed tokens) applied between siblings: the core echoes what it does
not own.

## 6. Projects and goals

### 6.1 Projects

A project is a body of work people steer, kept in the store:

- **its number,** from 1, by which the store, addresses and every
  request name it, never changed: 0 names no project, but the
  deployment's roles (people.md, section 3);
- **its name,** by which people know it, unique in the deployment
  (below);
- **its resources,** each adopted from a connector with a role: *owned*
  or adopted to write into; a *fork* of what others own, where the
  application owns its copy and takes part upstream; or *context*, read
  and never written (connectors.md, section 12). A project's resources
  may span connectors and systems;
- **its homes:** for each connector that projects goals, where it does
  (a repository whose issues show goals, a page that shows incidents);
- **its policy:** the most any task in the project may do, the
  requirements its effects need, its spend per period (authority.md,
  section 6);
- **its parties and their roles** (people.md, section 4);
- **its notes** (engine.md, section 10).

Every task belongs to one project, which its delegates inherit.

A project begins in one of two ways:

- **Seeded** at the deployment's first start, from configuration
  (`../protocol/configuration.md`, sections 3 and 8).
- **Made** after it, by a party whose role in the deployment is owner
  (people.md, 5.1): with the name its maker chose, a copy of a
  policy among the settings (section 13), and its
  maker as its owner and only party (people.md, 5.2). Its number is the
  deployment's next, past every project seeded or made, and never given
  again.

Once it is, a project is the same either way: nothing records how it
began, and a restart restores both from the store alone
(`../protocol/store.md`, 4.4).

A project's name:

- **Every project has one,** and no two projects of a deployment have
  the same: two names are the same when they are equal under Unicode's
  simple case folding, so "Tally" and "tally" collide. A client can
  then show a person's projects apart, and take a project by its name
  as surely as by its number.
- **Its shape:** UTF-8 text of 1 to 64 bytes, with no control character
  and no white space at either end, and never decimal digits alone, so
  that a name is never mistaken for a number.
- **Given, where none is chosen:** a project seeded with no name is
  named "Project N", N its number (`../protocol/configuration.md`,
  section 3). A seed's names, configured and given, are checked
  together, and a collision refuses the start. The store keeps the
  name with the project's policy (`../protocol/store.md`, 4.4). A
  project made after the first start is never given one: its maker
  chooses it (people.md, 5.1).
- **Renamed by an owner** (people.md, 5.1): the engine refuses a name
  another project holds, as *name taken*, or one out of shape, as it
  refuses a made project's (`../client/domain/contract.md`, section 4). A rename changes nothing
  else: the number, the addresses and the tasks stay.
- **Not a resource's:** a name is the project's own, never read from a
  resource or a home, which may be renamed, replaced or adopted
  elsewhere. An application may offer a resource's name as a
  suggestion; the name chosen is then the project's.

### 6.2 Goals

A goal is a tracked task, with a priority: the task a person asks for,
or that a chat proposes and a person accepts, marked so. It is a role a
task takes, not a sixth primitive, and the mark is plumbing: the task
carries it (tasks.md, section 3), and the engine's own behaviour turns
on it, so it is the engine's to keep, never a template's (section 9). A
task is tracked when it is made, or by an amendment, and untracked
likewise (tasks.md, section 7). A goal a chat proposes is requested by
the person who accepts it, not by the chat, so it outlives the
conversation it came from (tasks.md, section 9). A tracked task gets:

- **a priority** among the project's goals, which orders what competes
  for one resource: the queues of write holds (tasks.md, 6.2), and what
  connectors order (a landing queue, a pool's slots);
- **projections:** what each connector that projects goals writes of it
  for people to read (connectors.md, section 8);
- **news** of what its subtree's work changed, through the topics its
  connectors offer for goals, which the core subscribes it to, merged in
  one place of its inbox (tasks.md, 8.2);
- **a scope for notes,** which its subtree reads and writes;
- **a place among its project's goals:** task queries filter on tracked
  and order by priority (engine.md, section 11), and goals per project
  bound how many are live (tasks.md, section 11).

**The goal a task serves** is the nearest tracked task at or above it in
its tree, itself included; its priority is the one that orders the
task's waits.

**A goal's defaults are a template.** What a goal is made with when a
party sets one is jig's goal template (`../protocol/configuration.md`,
3.1), which an application keeps, changes or drops. The engine's
behaviour above turns on the mark, never on the template.

Any task may be tracked; most are not. A chat, a triage or a repair is
made untracked; an incident, or "give the payments team a staging
environment", may be made tracked. A conversation tracked later stays a
conversation (tasks.md, section 7).

## 7. Plans and coordinators

### 7.1 What a plan is

A plan is three things, kept apart:

1. **The strategy:** why this approach, in what order, what was dropped
   and why. It is text, and it is the LLM's: its transcript, its notes, a
   goal's projection. It changes freely.
2. **The structure:** tasks and their dependencies. It is the engine's
   state, and the engine runs it.
3. **The authority** to run it: the budget, the kinds and number of
   tasks, the resources. It is the task's authority (3.4).

So **a plan is the subtree of tasks one task has delegated.** Revising it
is creating, amending or cancelling tasks, each with its reason, which
the store keeps as the plan's history. Planning everything up front is
one batch (3.1): checked whole and, beyond its creator's authority,
refused, then proposed as one, for a person to accept (tasks.md,
section 9).

### 7.2 What a coordinator does

A coordinator is an agent whose charter is to break a goal down,
delegate, watch its inbox, revise, and report. The core knows nothing
particular of it: it is an agent's task with delegates, woken by what
its wake policy lets through. It may be the chat a person talks to, or a
fresh agent the chat delegates the goal to, which keeps a long
conversation out of the supervision. The choice is the charter's.

A coordinator woken rarely and far apart starts fresh by default, from a
brief (its goal, the plan's state and its notes) and the messages
waiting in its inbox. A chat resumes its transcript's current window,
and the store keeps the whole. Either is its charter's choice
(engine.md, 7.4).

### 7.3 Shapes of work

The same primitives make each of these:

1. **Done alone.** A person asks a chat a question, and it answers.
2. **Delegated once.** A chat's agent creates one task, hears its
   result, and tells the person.
3. **Handed to a procedure.** An agent decides what to do and delegates
   the doing to a connector's procedure: a restart verified by health, a
   change landed.
4. **A static plan.** Tasks with dependencies and no LLM over them.
5. **Coordinated.** An agent supervises and revises, for the goals that
   need it.
6. **Standing.** A procedure subscribed to a topic makes tasks as news
   arrives: a watch that triages alerts.
7. **Recurring.** At each period, a recurring task makes its recurring
   batch, the batch it was made with (tasks.md, section 10).

These are structures: what the work's tasks look like. A shape a party
starts by name (a chat, a goal, an application's experiment or incident)
is sugar over the primitives (section 9), in three forms:

- **a template:** data, the defaults that turn "a chat with these
  words" into a complete batch (tasks.md, section 4). What each
  carries, and jig's two defaults, chat (template 1) and goal
  (template 2), which an application keeps, changes or drops, are
  `../protocol/configuration.md`, 3.1; a project's policy overrides
  some of its fields (authority.md, section 6);
- **a helper:** a function composing plumbing requests, such as one that
  fills a template into a `Create`, or one that closes a conversation as
  one compound request (tasks.md, section 7);
- **facts and models:** pure functions over the documents a client
  holds, such as a chat's status, from its phase, its run and what it
  waits on, or a conversation's thread.

- **The engine never expands or checks a template.** The client fills
  one into an ordinary request, which the engine checks as it checks one
  written by hand. A template grants nothing: whatever it makes, a
  `Create` could make within the creator's authority, and permissions
  stay the primitives' (authority.md, section 6), so no role "may use" a
  template. A client that ignores one only makes its own lists
  inconsistent; authority and limits still hold. The engine keeps the
  configuration's templates, and each policy's overrides, only to serve
  each project's merged templates in the party's own document, `me`
  (engine.md, 11.1).
- **A task keeps the values it is made with.** A template changed later
  touches no task already made.

### 7.4 Two levels of delegation

An agent delegates in two ways, and chooses between them:

- **sub-agents within its run** (smith's `domain/run.md`, 5.3): cheap,
  ephemeral, sharing its workspace and its budget, and ended with the
  call that asked for them;
- **tasks:** durable and scheduled, with an executor, a workspace and an
  authority of their own, surviving restarts.

An agent reaches the engine through tools its host relays, each a
party's operation or a task's own, checked against its task's authority
(engine.md, 7.3).

## 8. Applications and external systems

### 8.1 What belongs to the application

- **The rule.** What the application does on an external system belongs
  to it: what it creates, and the changes it makes to what it owns. It
  writes only to what it owns, with two exceptions its connectors define:
  writing into shared resources through their own mechanics (landing
  into a shared branch, scaling a shared service within a policy), and
  taking part as one participant in objects others own (8.2).
- **Ownership is the store's.** Each connector records every object it
  made, with the key it made it under. On the system, ownership may also
  show (an author, a prefix, a label), but the store's record is what
  counts.
- **Projections are written, never read back.** What a connector writes
  for people to read is a projection of the store's state: keyed, made
  through the outbox, and never parsed back as state, beyond finding its
  own creations by their keys.
- **What the system owns is read afresh** before each decision that
  depends on it (section 4).

### 8.2 Owned and participating

Each object the application deals with is one of two kinds:

- **Owned,** the default: the application is its only writer. What comes
  back from it is only the system's facts.
- **Participating:** others write too. The connector reads what they
  write into the inbox of the task that deals with the object, and writes
  as one participant among several. A write that goes out may need a
  person's acceptance.

### 8.3 Plugging in

- **Systems are adopted as they are.** Making their objects is an
  optional convenience above that, never something the core assumes.
- **Nothing is required of a system** beyond the access the application
  is given. Policy is the enforcement; whatever protection a system has
  is a second net.
- **What a system offers decides how strong its promises are.** An
  effect is made at most once across uncertainty only if its system
  keeps its key or checks its condition; otherwise its task is held for a
  person when its outcome is uncertain. A requirement holds when an
  effect is applied only if the effect's own system checks it then
  (connectors.md, 4.3 and section 7).
- **What the application may do there is read at adoption,** and the
  project's policy keeps within it (connectors.md, section 12).

### 8.4 Drift

Another party may change what the application relies on: someone
restarts a service by hand, deletes an environment, pushes to a branch
the application owns. No workflow is built on this, but it must be safe:

- facts are read afresh before each decision; effects carry the state
  they were decided on where the system can check it;
- a change the application did not make to what others own, such as a
  landing it did not make, is news like any other;
- a change to an object the application owns holds the task that relies
  on it, and the client says why. Nothing is written over
  (connectors.md, section 11).

## 9. People

People are parties, as tasks are. Each has an inbox, and creates,
proposes, messages, amends, cancels, releases, moves, stops, decides and
answers within the authority their role in a project gives them
(people.md). A service may be a party too, signed in with a service's
credential (people.md, section 3).

- **A chat** is an agent's task whose requester is a party, made from
  jig's chat template (people.md, section 7): flagged a conversation,
  each of their words wakes it, and it resumes its transcript. It is how
  a person usually starts work: the chat answers, delegates, or proposes
  a goal.
- **A party's inbox** holds what waits for them: questions from the
  tasks they asked for, proposals they may accept, held tasks escalated
  to them, tasks whose executor they are, results, and the turns of
  their conversations.
- **The client knows the primitives,** not the shapes of work built from
  them. What a client asks of the engine, and reads from it, is jig's
  API, and the API is *plumbing*: the primitives as operations, subjects
  and reads, all in general form (`../client/domain/contract.md`,
  sections 3, 6 and 7). `jig-client` carries the plumbing, and knows no
  page and no shape (`../README.md`, section 10).
- **Shapes are sugar** (7.3): templates, helpers, and facts and models.
- **The one rule: sugar is built only from plumbing.** Whatever must be
  atomic (several operations in one decision, people.md, 5.1),
  whatever only the engine can compute (what a task waits on, what a
  pool has left, a budget given `AsAllowed`), and whatever the engine's
  own behaviour turns on (tracked and priority, a task's flags) is
  plumbing, in general form and never shaped. What sugar cannot be built
  from is a primitive missing, added in general form; sugar never needs
  an operation, or knowledge in the engine, of its own.
- **jig adopts sugar** an application wrote as
  `../client/domain/client.md`, section 19 says. Correctness (revisions,
  once, atomicity) is never sugar. Any application may refuse or redo
  jig's sugar; the plumbing changes rarely.
- **A party does what an agent does.** Their operations pass the same
  checks, only the source of their authority differing (authority.md,
  8.1 and 8.4), and both ask explicitly to propose (tasks.md,
  section 9). What stays asymmetric, by design, is what only parties do
  (people.md, 5.1); procedures decide nothing.
- **Presentation is the application's:** its screens, words, look,
  confirmations and navigation, its server and its URL space
  (`../README.md`, section 10). An application adds its own connectors' objects as
  subjects and requests of its own, through the same client.

## 10. A goal, as tasks

In `ops`, jig's example application (`../examples.md`), alerts fire on
the checkout service in production. P is the person on call:

```
P, the person on call
└─ W   watch the shop's services         observability's watch, a procedure subscribed to alerts
   ├─ T1  triage alerts 7–9              agent in the engine, a cheap model: noise
   ├─ T2  triage alert 12                agent: incident
   └─ I   incident: checkout errors      agent, an incident lead's charter; tracked
      ├─ R1 roll back checkout           infrastructure's remediate; holds the checkout service
      │                                  in production: a proposal P accepted
      └─ Q  why did 4.2 fail?            person: a developer's answer, for I's report
```

- **W is standing** (7.3): a procedure, not an agent. A storm of alerts
  wakes it once; it makes a triage task per batch.
- **T1 and T2 are cheap,** and end with verdicts. Only T2's makes I.
- **I is a goal,** tracked: people follow it in the client, its priority
  orders its remediations against other incidents' for the same
  service, and its notes' scope keeps what was learnt.
- **R1 is beyond I's authority** in production, so I's `delegate` is
  refused and I proposes it (tasks.md, section 9). The proposal passes
  over W, a procedure, to P, who accepts it with P's authority.
- **R1 is a procedure:** it holds the service, makes the rollback, and
  waits until the service is healthy, a fact the observability connector
  judges (connectors.md, section 7).
- **Q is a person task:** I asks a developer a question, and the answer
  reaches its report.
- **Had P closed W** while I ran, I would be cancelled with it; had P
  made I their own (tasks.md, section 7), it would go on.

## 11. What jig promises

The referees of jig's worlds hold the core, and every application's
wiring, to these, seen from outside (testing.md). They hold below the
API: whatever an application builds above jig's client, and whatever
transport carries its messages, nothing above them can break them.

- **Authority holds.** No effect is made, no task created and no spend
  recorded beyond what the deployment's rules and the project's policy,
  as they stand when it is decided, and the chain of authority above the
  task allow, except by a proposal a holder of that authority accepted.
  Every guarded requirement held when its effect was applied; every
  observed one was met, within its freshness, when its effect was
  decided. Admission and authority are the engine's, for a party's
  request as for an agent's call.
- **Spend is bounded.** Each completion's maximum is reserved before it
  is made and each priced effect is charged its maximum when decided, so
  recorded spend never exceeds a task's budget. The only spend beyond it
  is what a worker spent on turns it lost before they were committed,
  bounded by the turns it may keep unacknowledged.
- **Once, as each kind allows.** An effect whose system keeps its key or
  checks its condition is made at most once, across restarts and late
  arrivals included; an idempotent set may be repeated, to the same final
  state; an effect of any other kind whose outcome is uncertain is not
  repeated without a person's decision. Every task in a batch is made, or
  none is; every operation of a compound request is made, or none is. A
  party's request is made once under its key, however often it is sent,
  and answered once its commit is durable.
- **A decision names what it saw.** An amendment, a release, a decision
  on an escalation and a person task's answer name the revision they are
  decided on, and a decision on a proposal its number, since a proposal
  never changes; one whose object has moved is refused as moved, saying
  what it is (`../client/domain/contract.md`, 5.2).
- **A credential is its party's.** A request presented with a credential
  jig verified is that party's, and everything after it is checked
  below: admission, authority, revisions (people.md, section 3).
- **Nothing ahead of its commit.** Nothing a party was answered, a host
  was assigned or acknowledged, or a system was asked to do, depended on
  state the store did not keep. What people read and watch waits for the
  commits it shows; only a run's text, as it is written, runs ahead of
  the store, and is marked provisional.
- **Order.** A task starts only after its dependencies are done and
  closed; one run at most writes a resource at a time; a pool admits no
  holder beyond the slots it is known to have, and one that shrank admits
  none until it drains; an effect lands exactly the state its decision
  saw, where the system can check it; a task ends only after its
  delegates.
- **Nothing is lost.** A result reaches its requester; a person's words
  reach the task they were written to, or that task ends; a proposal is
  decided, withdrawn, or still waiting where a holder can see it; nothing
  read comes back unread, across its task's end and restarts (people.md,
  section 6).
- **Nothing is written over.** No effect writes to an object the
  application does not own except as a participant or through its
  connector's own mechanics, and none overwrites a change it did not
  make.
- **Bounded.** What the engine holds stays within its limits, whatever
  the history in the store; every story ends within a bound.
- **A run reads its revision.** What a run is given of its charter is
  the version its task's revision read when the run was prepared,
  whatever changed since; what it may do is the authority that stands
  when it decides. A change of settings is committed whole or not at
  all.

Where these stop, above the API, the application holds:

- **confirming a deciding act,** with its facts shown. jig gives a
  decision's effect as data in its documents (what accepting a proposal
  grants and costs, what releasing does, what cancelling ends), and its
  test kit's check, never mandated, finds in an application's worlds a
  decision whose revision was in no document the client delivered to
  the application before it was sent (`../client/domain/testing.md`,
  6.2);
- **keeping a credential from use across sites,** as keeping it out of
  its logs is: its server's, with jig's default HTTP mapping's example
  guards and attack stories to use where they suit
  (`../protocol/browser.md`, 12.2 and section 15).

## 12. Open questions

- **Agents as identities** that serve many tasks, such as a service's
  standing operator. The model allows them; a long-lived task serves in
  their place.
- **Procedures written by agents,** deterministic workflows like a coding
  agent's scripts. Until they are wanted, procedures are Rust, owned by
  connectors or the core.
- **Wake policies:** what wakes a task, and whether a cheap model should
  triage news before a coordinator wakes. Tuned from use (tasks.md,
  section 13).
- **Scaling out:** one engine per deployment until it binds.

## 13. Settings

What configures work, beyond what a process needs before it reads its
store, is the deployment's settings: its **charters**, its
**templates**, the **policies** its projects are made from, its
**rules**, and an application's own kinds (engine.md, section 17).

- **Defaults, and changes over them.** The binary holds the defaults;
  the store holds what was changed, setting by setting. A setting never
  changed follows its default, so a release that improves one reaches
  every deployment that kept it. A reset erases a change.
- **A change is one decision** (section 4): a set of settings, checked
  whole as a start checks them, against the revision it names, and
  refused as stale when another is the newest. The deployment's owners
  change the deployment's settings, and a project's owners an
  application's settings kept per project; a project's own policy stays
  its owners' (people.md, 5.2). Anyone else proposes, and an owner's
  acceptance is the same decision (tasks.md, section 9).
- **A revision is the settings as a change left them.** Each task
  records the newest when it is made, and its runs read their charter
  from it: its instructions, models, tool families, hosts, waiting,
  resume and budget. Templates, policies and the rules are read only as
  things are made or decided, so the newest is the one.
- **Running work keeps its charter.** A change reaches a running task
  only when it moves: a conversation at its next run, other work when
  a change or an owner moves it, each only if the newest still offers
  its charter. A removed charter lives while a task holds it; a task
  that cannot move finishes where it is.
- **What a task may do is never pinned:** its authority is the rules'
  and its project's policy's as they stand (authority.md, section 6).
- **Bounded:** revisions held and the charter versions they read are
  limits, sized at the start (engine.md, 17.8).
