# jig's domain

Provisional, 2026-10-07, revised 2026-10-15. The design of jig's domain
layer: the core an application's engine composes, the contract its
connectors meet, what its root does, where its agents run, and how all
of it is tested. jig as a whole, and why it is a kit, is `../README.md`;
the client's domain is `../client/domain/`, designed separately. The
mechanics are those of skein's `programming-model.md`.

## 1. Reading order

1. **core.md:** the model: the five primitives, decisions and the store,
   the owner keeping the value, projects and goals, plans and
   coordinators, applications and their systems, parties, and what jig
   promises. Read it first.
2. **tasks.md:** the hub: tasks, batches, lifecycle, holds, inboxes and
   wake policies, proposals and escalations.
3. **authority.md:** what a task may do: authority as a value, budgets
   and spend, checks, proposals, requirements and their verdicts.
4. **connectors.md:** the contract every connector meets, and what
   crosses between it and the core: jig's API toward applications.
5. **engine.md:** the core as a whole: its children, its vocabulary
   toward the root, commits and the store, the restart script, agents'
   runs and the engine's tools, the fleet, briefs, notes, views,
   accounts.
6. **root.md:** what an application's root does: the one shape, the
   journal, routing, translation, putting outputs together, and what it
   never does.
7. **people.md:** parties: identity and credentials, roles, their
   requests as jig's operations, inboxes, chats as a template, person
   tasks, notifications.
8. **hosts.md:** agents and their hosts: the contract, the three shapes,
   the agent's two kinds, both smith's, the one host for runs wherever
   their agents run, smith's payloads it carries, workspaces, and what
   the core takes from smith.
9. **testing.md:** the worlds, the conformance world and its referee.

`../examples.md` is the example application, `ops`, which each document
uses for its examples.

## 2. Structure

```
an application's engine
<application>-domain            the root: the application's (root.md)
├── jig-domain-core             the core (engine.md)
│   ├── jig-domain-tasks        the hub (tasks.md)
│   ├── jig-domain-authority    policy (authority.md)
│   ├── jig-domain-people       parties (people.md)
│   ├── jig-domain-fleet        hosts, placement, attempts
│   ├── jig-domain-accounts     LLM accounts
│   ├── jig-domain-brief        briefs
│   ├── jig-domain-notes        notes
│   └── jig-domain-views        parties' watches
├── <connector>…                the application's (connectors.md)
├── jig-domain-host             hosted runs on the engine's slots, if it runs agents (hosts.md)
├── smith-inline-agent          their agent, smith's: its domains in this process
└── …                           the application's own child domains

an application's worker, if it has workers
<application>-worker            the worker's root: the application's
├── jig-domain-host             hosted runs (hosts.md)
├── smith-host-domain           their agent, smith's: processes; or smith-inline-agent, as the root chooses
└── <workspace>                 the application's

skein-lib's journal             the commit barrier every root holds (root.md, section 4)
```

The crate graph is the role graph: no jig crate depends on an
application's; the core and its children depend on skein-lib alone; the
hub depends on skein-lib alone too, and carries smith's payloads as
bytes; the agent capabilities are smith's crates, and jig ships none.
Which crates link smith is hosts.md, 6.8. Nothing in jig names a
system.

## 3. Conventions

- **A bare file name** names a document of this directory: `tasks.md`.
  jig's other documents are named by their path from here
  (`../README.md`, `../examples.md`).
- **Code cites a section** at module or type level, as `domain/tasks.md`,
  6.2, never stamped on every field.
- **skein's foundation documents** are named by their own file names:
  `programming-model.md`, `testing-strategy.md`, `notes.md`. skein's
  design documents are named with skein's name: skein's `lib.md`.
- **smith's documents,** in smith's `docs/design/`, are named with
  smith's name and their path there: smith's `domain/run.md`, smith's
  `protocol/channel.md`.
- **An application** appears only as an example, and says so: `ops`
  most often, jig's test application where a second example helps.
- **Each document** says in one page what it is, then its structure, its
  parts, what it owes and is owed below the domain, its world, and what
  is open.

## 4. Names

| Name | What it is |
|---|---|
| application | a program built on jig: its roots, its connectors, its workspaces, its client's objects |
| deployment | one engine, its store, its hosts, its configuration; an id made at its first start |
| engine | the application's process where work is decided: its root, jig's core, its connectors |
| root | the application's top-level domain in a process (root.md) |
| core | jig's mid-level domain inside the engine (engine.md) |
| journal | skein-lib's container that numbers commits and holds outputs until durable (root.md, section 4) |
| decision | what one event changes and asks for, gathered into one commit |
| hand-off | a question the core asks a connector within one decision, through the root |
| the door | the journal's way out for outputs that decide nothing |
| project | resources with roles, homes, a policy, parties' roles, notes (core.md, 6.1) |
| task | a unit of work someone asked for (core.md, 3.1) |
| goal | a tracked task: a priority, projections, goals' topics, a notes scope (core.md, 6.2); jig's goal template gives its defaults |
| plan | the subtree of tasks one task delegated (core.md, 7.1) |
| batch | the tasks one decision makes, checked whole (tasks.md, section 4) |
| input | an ended task's result a new task names |
| closing | a task settling before it ends: its run, its delegates, its effects, its releases |
| executor | who carries a task out: an agent, a procedure, a party |
| charter | how an agent executor is set up, and which hosts may run it |
| result contract | what counts as done: a report, a verdict, a connector's result, a delivery, a choice, an approval |
| chat | jig's default template for a conversation with an agent: an agent task flagged conversation and kept, whose requester is a party (people.md, section 7) |
| template | an application's recipe for creating a task of one shape, numbered in its configuration and overridden by a project's policy; the client fills it into a `Create`, and the engine never sees one (core.md, 7.3; people.md, section 7; `../protocol/configuration.md`, 3.1) |
| label | an opaque number the application gives a task, through a template or by hand, which lists filter on and no engine rule reads (tasks.md, section 3) |
| conversation, listed, retention class | a task's flags: its turns are entries for its requester, with a read position; once ended, it is indexed by its project; kept, or forgotten once read (tasks.md, section 3; people.md, section 6) |
| last activity, waiting on | when a task last heard words, committed a turn or had a delegate end; what it waits for, computed by the engine: a party, its delegates, news, or nothing (tasks.md, section 3) |
| coordinator | an agent whose charter is to delegate and supervise |
| standing procedure | a procedure subscribed to topics that makes tasks as news arrives (connectors.md, 6.6) |
| run, attempt | one activation of an agent task on a host; one try at it, fenced by a number that only grows |
| tries | the failed attempts counted per class, which a release resets |
| transcript, turn | an agent task's conversation, kept turn by turn; one LLM turn, committed as it ends |
| window | smith's: the turns from the last that opens a window (a window turn, or a conversation's first) are the current window, all a resumed run is given (engine.md, 7.2) |
| a completion's retry | a turn's completion asked for again within one run, counted from zero by smith; a later retry's live text replaces an earlier one's (hosts.md, 6.5). Not one of a task's tries |
| call | a run's tool call, named by the run, decided once |
| brief | the context a run is given for why it runs |
| message, inbox | what a task hears once started; its durable, bounded store of them |
| message refused, the unread fence | a message a run did not take, with smith's reason (too large, full, a name in use, ending), told inside the run's next turn or its answer; the answer's last message read, after which every message relayed, and not refused, went unread. Either stays in the inbox (hosts.md, 6.4 and section 9) |
| wake policy | which messages wake a task, and how they batch |
| subscription, topic | a task's standing interest; a connector's subject |
| reference | a task another may message |
| party | a person or a service that takes part (people.md) |
| provider | one of the application's ways of signing in, numbered in its configuration; an identity is a provider and a subject (people.md, section 3) |
| credential, sign-in | what jig mints for an identity, verifies, expires and revokes; the signed-in session it names, numbered by the core (people.md, section 3) |
| the printed link, the application's sign-in, `Mint` | jig's one-time secret on loopback; any other way of signing in, on the application's own routes; its server's word that it verified an identity, on which jig mints (people.md, section 3) |
| operation | one of jig's primitives as a request: `Create`, `Propose`, `Withdraw`, `Message`, `Amend`, `Cancel`, `Release`, `Stop`, `Move`, `Decide`, `Answer`, `Take`, `HandBack`, `Seen`, and the project's own (people.md, 5.1) |
| compound request | one keyed request holding several operations, admitted in order and committed in one decision, all or none, within operations per request (people.md, 5.1) |
| grace | how long a task cancelled with one has to finish before it is cancelled (tasks.md, section 7) |
| `AsAllowed` | a budget given as much as fits under its creator at admission, capped; the answer says what was given (authority.md, section 7) |
| escalation role | the role a project's policy names to hold escalations past every holder in a tree (authority.md, section 6) |
| a party's role | what a party may do in a project: owner, maintainer, member, observer, or the policy's own |
| a resource's role | owned, fork, or context, in a project |
| authority | what a task may do (authority.md, section 3) |
| grant | an effect or a read of a connector's kind on a pattern of resources |
| fit | whether a delegate's authority goes under its creator's, given what it has left |
| funder | who a task's budget was reserved from: a task, a party's pool, a project's period |
| price | what a priced effect costs at most, in the deployment's unit |
| effective authority | a task's authority within the rules, policy and adoption access as they stand when it decides |
| proposal, escalation | an action beyond its proposer's authority; a held task; each waiting for a holder |
| rules, policy | the deployment's, a project's: the root of authority |
| requirement, judge, verdict | facts an effect needs; the connector that judges them; met, wait or refuse |
| guarded, observed | a requirement the effect's own system checks as it applies it; one judged only when the effect is decided, within a freshness |
| connector | an external system as the engine sees it: the application's |
| resource | what has an identity on a system, named by a connector's number and a path |
| hold, pool slot | a resource a task writes alone; one of a pool's counted slots (tasks.md, section 6) |
| writer slot | the one run that may write a held resource at a time |
| effect, outbox, key | what the application does on a system; its entries waiting to be made; what finds them again |
| recovery class | how an effect of a kind whose outcome is uncertain is resolved: keyed, conditional, idempotent, unrecoverable (connectors.md, 4.3) |
| retry deadline | when an uncertain attempt may be tried again, committed with the attempt |
| procedure | engine code as an executor, a connector's or the core's |
| projection, home | what a connector writes of a goal for people to read; where, per project |
| owned, participating | what the application writes alone; what it writes among others |
| drift | a change the application did not make to what it owns and relies on |
| host, hub | where a run runs: the engine's own slots, or a worker; `jig-domain-host`, the one hub hosting runs on either (hosts.md, section 6) |
| agent capability | the hub's agent, one of smith's two crates speaking smith's parent face: `smith-host-domain`, agent processes, one per run (hosts.md, 5.1); or `smith-inline-agent`, smith's domains in the host's process, the engine's hub's agent and a worker's by its root's choice (hosts.md, 5.2) |
| smith's payloads | the charter, a resumed run's transcript, turns' bodies, call names and results: typed in smith's agent capabilities, bytes in the hub, translated by hand-out to the root's protocol layer; smith's facts are decoded the same way into jig's typed activity (hosts.md, 6.8) |
| workspace, item | what a run works in; what a connector names for it to hold |
| workstream | the key a workspace is cached under: the holding task's number |
| delivery | a run's hand-over through its workspace, made by the application's workspace |
| saved state | unfinished work a run's host saved, which the next run starts from |
| token | a name the core holds for a value a connector keeps (core.md, section 5) |
| conformance world | the world every application runs its domain on, with jig's referee (testing.md, section 5) |
| test connector | jig's connector for its own worlds |
| smith | the agent, and the kit it is built with, in its own repository |
| session | an agent's conversation with an LLM: smith's |

## 5. Decisions this design takes

Questions `../README.md` left open, and others met while writing, with
where each is answered:

- **Payloads the core orders but does not read:** the owner keeps the
  value; the core holds tokens and sizes, and the root puts outputs
  together (core.md, section 5; root.md, section 7).
- **Requirements across connectors:** requirements are judged by the
  connector whose facts they are, as verdicts the core asks for through
  the root, whichever connector makes the effect (authority.md,
  section 10; connectors.md, section 7).
- **Scarce resources:** holds are exclusive or pooled, and a taken one
  refuses or waits, as its connector says; a task takes every hold at
  once or waits with none (tasks.md, section 6).
- **Parties that are not people:** a party is a person or a service; a
  service signs in with a credential jig mints, holds roles, and makes
  the requests a person makes (people.md, section 3).
- **Credentials are jig's, sign-in flows the application's:** jig mints,
  verifies, expires and revokes credentials; the printed link and
  services' credentials are its own ways of signing in, and any other is
  the application's, on its own routes, ending in a `Mint` on its
  server's word; the core knows an identity as a provider and a subject,
  and jig adds no acting for (people.md, section 3).
- **Hosting runs inside the engine:** the same hub as on a worker, a
  child of the application's root, whose slots the fleet places on like
  a worker's, with `smith-inline-agent` as its agent, with no workspace
  (hosts.md, sections 5 and 6; engine.md, section 8).
- **The worker:** jig ships the hub; the worker's root and its
  workspaces are the application's (hosts.md, sections 4 and 7).
- **Costs beyond LLMs:** a connector may price kinds of effect, charged
  at the decision (authority.md, section 7).
- **An effect waiting for a verdict:** never committed; the asker asks
  again (connectors.md, 4.5).
- **Restart:** a script the core runs, step by step, through the root
  (engine.md, section 6; root.md, section 8).
- **Once, by recovery class:** each kind of effect is keyed, conditional,
  idempotent or unrecoverable, as its system allows, and the promise of
  once is stated per class; an uncertain unrecoverable effect holds its
  task for a person (connectors.md, 4.3; core.md, section 11).
- **Retry deadlines are absolute,** committed with each attempt before it
  is sent, so restarts never postpone an uncertain effect for ever
  (connectors.md, 4.3).
- **Guarded and observed requirements:** a guarded one holds when its
  effect is applied, an observed one when it is decided, within a
  freshness; a policy may require a guard (authority.md, section 10;
  connectors.md, section 7).
- **A hard ceiling on recorded spend:** a run reserves each completion's
  most before making it and winds down on its reserve when the rest is
  spent, as smith's budget does (smith's `domain/run.md`, 9 and 9.1), and
  the core places no run smith would refuse for its budget; a priced
  effect is charged its maximum; only turns lost with a worker escape
  it, within a stated bound (authority.md, section 7; hosts.md,
  section 8).
- **Standing work renews by period:** a standing task's allotment of
  tasks and spend is carved again each period (authority.md, section 7).
- **Periods are the application's to say:** the engine keeps no
  calendar; a period reaches it from the root, saying when the next
  begins if its source knows (authority.md, section 7; engine.md, 4.1).
- **Pools that shrink** keep their holders and admit nobody until they
  drain; the promise is about admission (tasks.md, 6.6).
- **Effective authority:** a narrowing of rules, policy or access reaches
  every later decision, existing tasks' included; committed effects are
  made; pending proposals are judged at acceptance (authority.md,
  section 6).
- **Committed turns survive;** what a worker loses before a commit is
  done again by the next attempt (hosts.md, section 1).
- **What the conformance world catches** is stated as what its scenarios
  reach, with fakes reporting what they observed independently of the
  root's translation (root.md, section 1; testing.md, section 5).
- **One host:** runs are hosted by one hub, `jig-domain-host`, in the
  engine and on workers alike, its agent a capability of two kinds, both
  smith's (agent processes, `smith-host-domain`; or
  `smith-inline-agent`, in the engine, or on a worker by its root's
  choice), translated with the same functions, its link to the core its
  root's. jig ships no agent of its own. The run's lifecycle is built
  and tested once (hosts.md, sections 1 and 4 to 6).
- **One process runs workers' agents as child processes,** as between
  hosts: the engine and its workers share the process, and each worker's
  agents run in processes of their own, which the application's binary
  may serve as a command of its own (hosts.md, section 3).
- **smith's payloads are bytes in the hub, typed in the agent:** the
  root that composes the two hands each payload out to its protocol
  layer, which decodes or encodes it with smith's codec functions, as a
  host tool's call is decoded; neither the hub nor the core links smith
  (hosts.md, 6.8).
- **Every message ends once:** read, refused or unread. The start
  carries the messages waiting in the task's inbox; a refusal travels
  inside the run's next turn, or its answer, and is committed with the
  turn's read messages; a message the run refused or left unread stays
  in the inbox; every kind of message fits the deployment's bound, and
  the fleet admits no host whose agent takes less, so no run refuses one
  for its size (hosts.md, 6.4 and sections 2 and 9; tasks.md, 8.2 and
  8.3).
- **An application's credentials are lent as accounts are:** named by
  number and generation, their values its protocol layer's, named by the
  connectors for a run's workspace and granted with the assignment
  (engine.md, 12.1; connectors.md, section 10).
- **A run's text where its host has it:** activity from every host
  carries no content; a run's text, and a tool begun with its call, come
  from a host whose agent runs in its process, and from agent processes
  once smith's channel carries them, under the same contract (hosts.md,
  6.5).
- **Level-triggered views:** a watch delivers its subject's current
  document, one delivery in flight, changes coalesced, never queued; only
  a run's text as it is written is a stream, from an offset; an
  application's subjects are watched the same way (engine.md,
  section 11). This and what follows are what the client's design asks
  of the domain (`../client/domain/contract.md`, section 9).
- **What a party reads is durable:** a read's answer and a watch's
  delivery wait for the last commit made before them (engine.md, 5.2).
  What marks a watch changed is named by each commit (engine.md,
  section 11).
- **The inbox across projects,** its read positions (one over results,
  one per task flagged conversation) moved by `Seen`; a live
  conversation's held by the people child, an ended one's in the store,
  erased with its task (people.md, section 6).
- **Retention is a task's class:** kept, or forgotten once read, its
  record and read position erased in one decision; jig's chat template
  keeps (people.md, section 6).
- **Decisions name what they saw:** a release names its hold's revision,
  an amendment its task's spec's, a person task's answer its own; each
  moves only when that thing changes, and a decision whose revision
  moved is refused as moved (people.md, 5.1). A hold's moves when the
  task is held anew or for another reason, not when its escalation
  passes up; an approval names the state it approves too (people.md,
  section 8).
- **A proposal nobody may accept is refused,** whoever makes it: a
  party's at admission, a run's `propose` in its answer, a procedure's
  step in its decision, saying so, rather than made to wait for ever
  (people.md, 5.1; tasks.md, section 9).
- **A party's requests are jig's plumbing:** its primitives as
  operations, the same an agent's tools reach, through the same check,
  only the source differing (people.md, 5.1; authority.md, 8.1 and 8.4).
  Shapes of work are templates and helpers above them, never the
  engine's.
- **Compound requests:** several operations under one key, admitted in
  order and committed in one decision, all or none, within operations
  per request: closing a conversation with its delegates' disposals,
  ordering goals, an amendment that releases, words that release a
  stopped task (people.md, 5.1). A release of an uncertain effect says
  whether to make it again.
- **Proposing is explicit,** for parties as for tasks: a request beyond
  authority is refused, naming what it lacked and whether a proposal
  could be accepted, and a proposal is asked for (authority.md,
  section 9; people.md, 5.1).
- **A party's root has someone to escalate to:** a party becomes a
  root's requester only when their role decides escalations or the
  project's policy names an escalation role (people.md, 5.1;
  authority.md, 8.4).
- **Templates are the application's, never the engine's:** numbered in
  its configuration, their defaults overridden per project by its
  policy, never their flags, label or tracking, served in `me`, and
  filled in by the client; permissions stay primitive (people.md,
  section 7; authority.md, section 6).
- **`AsAllowed`,** the one piece of plumbing templates need: resolved by
  the engine at admission, the answer saying what was given
  (authority.md, section 7).
- **Refusals with facts,** and **acts offered** in each object's
  document, so a client never computes authority; creating and amending
  are offered with bounds, made, proposable and to whom, or refused, and
  judged again when they come (`../client/domain/contract.md`, 3.2, and
  sections 4 and 5).
- **An application's keyed requests** kept by the people child as jig's
  are, their outcomes by their connector (people.md, 5.1;
  connectors.md, 5.5).
- **An application's subjects, reads and requests name their scope,** a
  project, the deployment, or the party's own: the core admits on the
  party's role there, the party's own on the credential alone, and the
  connector judges what the party may see (engine.md, section 11;
  connectors.md, 5.5; people.md, 5.1).
- **A procedure hears what its step made:** the task its step's batch
  was given, within the same decision, for its connector to record
  durably (connectors.md, 6.3).
- **A task's activity kept** in commit order, the words parties wrote
  among it (engine.md, 7.2).

## 6. What is open

Each document lists its own; the ones that reach across:

- the deployment's unit of spend, and costs over time (authority.md);
- generating a root's boilerplate rather than copying it (root.md);
- fairness for runs on the engine's loop (hosts.md);
- how fresh a verdict's facts must be (connectors.md);
- how many commits the focused suite crashes at (testing.md).
