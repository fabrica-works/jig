# Pages

Provisional, 2026-10-09. jig's pages: how a client is organised, what a
party finds where, and what they do there. They are jig's, as the
primitives they show are, and every application's client has them,
around its own. This document says what each page shows and offers, never
how it looks: no layout and no style. The cards that every page reuses
are objects.md; what each page reads and watches is contract.md. What is
still open is listed in section 11.

## 1. In one page

- **Three things people do.** They **talk** to agents (section 4),
  **decide** what waits for them (section 3), and **watch and steer** the
  work (sections 5 and 6). jig's pages are organised around these, not
  around jig's parts.
- **The task page is the one universal page.** Every task, whatever its
  executor (an agent, a procedure, a person), has a page of the same
  shape: what was asked, where it stands, what it did, what it spent,
  and what may be done to it. A chat and a goal are tasks; their pages
  are task pages that lead with their conversation and their plan.
- **jig knows the primitives; the application adds its parts.** jig's
  pages know tasks, executors, messages, authority and connectors, and
  nothing of the shapes of work made from them. What a connector's
  resources and procedures look like (a remediation on its way to
  healthy, an environment's readiness, a change on its way to landing) is
  the application's, shown in the slots jig's pages leave for it
  (root.md, section 6).
- **One object, one card, many places.** A proposal shows in the inbox,
  in the conversation of the chat that made it, and on its proposer's
  page: the same card, with the same actions, deciding the same thing
  once (objects.md, section 2).
- **Every request is a decision.** It is pending until the commit that
  answers it is durable, and never shown as done before; refused, it says
  why; beyond what the party may do, it is either refused or proposed to
  someone who may, as the engine does for that request, and the page says
  which before it is sent; sent twice, it is made once.
- **What is committed is the truth; a run's text is provisional.** Pages
  show the engine's committed state, delivered as it changes; what a run
  writes is shown as it comes, marked so, and replaced by what was
  committed.
- **Held is where problems surface.** A held task says why in words, and
  offers the actions that fit its reason (objects.md, section 5).
- **People steer at the level of goals, plans and outcomes.** The
  mechanics stay on the systems: a page links to what a connector shows
  there, and never asks a person to do a system's job to keep work
  moving unless a rule says so.
- **Bounded, like the engine.** Lists are paged, large trees fold into
  counts, history is read when asked for.

## 2. The map and the frame

```
Inbox            what waits for the party, across projects
Conversations    their chats; a new one
Project ▸
  Goals          the goals, by priority: progress, spend, what is held
  Tasks          the project's live tasks, by phase, executor or goal
  Notes          by scope: reading, correcting, deleting
  Settings       resources, policy, roles, spend, recurring tasks
Task N           the universal page, from anywhere
Deployment       hosts, accounts, connectors, capacity
… the application's pages: for ops, Incidents and Environments
```

- **The frame** is on every page:
  - what needs the party, as a count, and updates not yet read, as a
    quieter one;
  - the project they are in, and a way to switch;
  - going to a task by its number;
  - whether the client is live, behind or offline (client.md, 8.2), and
    who is signed in;
  - the navigation: jig's pages and the application's.
- **Home and navigation are the application's choice.** The application
  says which of jig's pages its frame shows, in what order among its
  own, and which is home; the inbox by default. `ops` makes the inbox
  home, and puts Incidents after it. A page that cannot apply is not
  shown: Conversations without a charter for chats, the Deployment page
  to a party without a role on the deployment.
- **Links are by number.** Every task, wherever it is named (a chip, an
  entry, a history line, a turn), links to its page; its number is its
  address, never reused (`../../domain/tasks.md`, section 3). So does
  every proposal, through its proposer's page.

## 3. The inbox

What waits for the party (`../../domain/people.md`, section 6), shown
across every project they hold a role in.

- **Two groups.**
  - *Needs you:* proposals and escalations to decide, questions to
    answer, person tasks to carry out. Each holds some work up.
  - *Updates:* results of what the party asked for, and replies in their
    chats. Nothing waits on them.
- **Ordered for what waits.** Needs you, oldest first: the longest waiting
  holds work up most, and each entry says how long it has waited and,
  where its stall applies, when it will pass up. Updates, newest first,
  those not yet read marked new.
- **Filtered** by project, by kind of entry, and by goal: an entry belongs
  to a goal when its task is in the goal's subtree.
- **Derived, not kept.** An entry is there while its object waits: a
  proposal pending, a question unanswered, a person task active, an
  update unread. Deciding it, on this page or any other where its card
  shows, takes it away.
- **Results only of what the party asked for:** their chats, their goals,
  the tasks they made themselves. What a goal's delegates finish goes to
  the goal, and shows on its page.
- **What agents decide stays with them.** A proposal or an escalation
  reaches a party only once no task above it could decide it, so the
  inbox holds what only a party can.
- **Deciding in place.** Each entry is its object's card, with a line of
  context: its project, and its task's lineage up to its goal or chat
  ("shop ▸ I incident: checkout errors ▸ R restart checkout"). Its actions
  are on the card; opening it gives the task's page, with the same card.
- **Addressed to a role,** an entry shows for every party holding it,
  saying so ("for the person on call"), until one acts; a person task is
  taken first, so that two do not answer at once (objects.md, section
  7).
- **Reading updates** is a request: opening the updates moves the party's
  read position over results to the newest shown, and opening a chat
  moves its own position over that chat's replies (contract.md, 3.2).
- **Counted:** what needs the party, exactly; updates unread, up to a
  limit, past which the frame says "more than" it.
- **Nothing else is the inbox's own:** no archive, no snooze, no flag.
  What is done is in the history of the task it was about.
- **Live and bounded.** An entry appears as the commit that makes it
  waits for the party, and leaves as it stops waiting. The engine holds
  each party's entries up to a limit; the page reads the rest a page at a
  time.

## 4. Conversations

A chat is an agent's task whose requester is a person
(`../../domain/people.md`, section 7). Its page is a task page whose
activity, the conversation, comes first.

- **The list:** the person's chats, across projects, each with its name,
  its project, what it is doing in words, replies not yet read, its live
  delegates and its spend; by last activity; live ones first, ended ones
  read below them a page at a time.
- **Starting a chat:** a project and the first words. Its charter is the
  project's for chats and its budget its role's default
  (`../../domain/people.md`, 5.1); the form says both and asks neither.
  A chat may also be started from what it is about (a task's page, a
  held task's card, a goal), with the thing named in its first words.
  Answered with the chat's number once durable, its page opens with its
  first run due. Refused, it says why: the person's pool, the project's
  live tasks, a limit.
- **The conversation** (objects.md, section 10): turn by turn, across
  every run the chat has had: the person's words with their state, the
  chat's replies, its work folded to a line each, its calls to the engine
  as the cards of what they made, what it heard besides the person, and
  where its runs began and ended. Long conversations read from their end,
  a page at a time.
- **What it is doing,** always, in words:

  | The chat is | The page says |
  |---|---|
  | due, preparing or claimed | starting |
  | running, writing | writing |
  | running a tool | which tool, and for how long |
  | running, waiting with its slot | waiting for you |
  | idle, parked | parked; your words resume it |
  | backing off | retrying, when, and why: "host lost, try 2 of 3" |
  | held | why, with its card |
  | closing, then ended | closing; ended, and when |

- **Words** may be written while it is not closing or ended: a running
  chat hears them at its next turn, a waiting one wakes, a parked one
  resumes. They do not interrupt the turn being written; to stop it now,
  the person stops it. Words to a chat the person stopped release it
  too, in the same request. Sent words are not edited or withdrawn.
- **Stop, close and cancel** are three requests, kept apart in name and in
  what each says before it is sent (objects.md, section 11).
- **Beside the conversation:** its project, its delegates live and ended
  as a small tree of chips, the goals it proposed (now the person's), its
  spend and what it has left, and its connectors' panels.

## 5. The task page

### 5.1 Its header

- **What it is:** its number, its title (its spec's first line), its
  project, whether it is a goal, and its priority if so.
- **Who carries it out:** an agent and its charter ("an incident lead");
  a procedure and its connector ("infrastructure's remediate"); a party
  or a role.
- **Where it stands:** its phase in words (objects.md, section 3).
- **Where it comes from:** its lineage, from the party or the deployment
  that rooted its tree down to it, each a chip.
- **What may be done:** the requests that fit its phase and the party's
  role (objects.md, section 11), as the task's document says.

### 5.2 Its sections

| Section | Agent | Procedure | Party |
|---|---|---|---|
| overview: what was asked, its result, what it hears | yes | yes | yes |
| activity: what it did | its runs and conversation | its connector's panel | the question, and the answer |
| plan: its subtree | yes | what it made | rarely |
| history | yes | yes | yes |
| budget and authority | yes | yes | its funding only |
| connectors' panels | for the resources it names | its own panel first | for what its question names |

A section with nothing in it is not shown.

- **Overview.** What was asked: its spec's words, whole; the resources it
  reads and writes, by name (parts); its inputs as chips with their
  results; for an agent, its charter; for a party, the question. What
  counts as done: its result contract in words. Its result once it has
  one, by its contract (objects.md, section 8). Its dependencies as
  chips with their phases. What it hears: its requester, its
  references, its subscriptions and its wake policy in words ("wakes on
  your words and its delegates' results"). What it holds: the resources
  it writes alone, and which run holds the writer slot now.
- **Activity.** For an agent, its conversation and its runs: each run's
  attempt, why it ran, how it began (resuming or fresh), how it ended,
  what it spent; failed runs kept, folded. For a procedure, its
  connector's panel, a part. For a party's task, its card, then who
  answered and how.
- **Plan.** Its delegates, and theirs, as a tree of chips, each with its
  executor, phase, spend and dependencies on its siblings; a proposal
  waiting in the subtree on its proposer's line. Ended branches fold to
  their line; a level with many tasks folds into counts by phase ("12
  done · 2 running · 1 held"), opened on demand; a folded line carries
  the count of what is held below it, so nothing held is hidden.
  Filtered: held only, live only. Live tasks are watched; ended ones are
  read as their branches open.
- **History:** each task made (with its batch), amended (before and
  after), held (why), released, cancelled or moved, by whom (a party, a
  task, the deployment), when, and the reason given. For a coordinator it
  is the plan's story. Read a page at a time, for the task alone or its
  subtree.
- **Budget and authority:** its budget, spent by itself, by its ended
  delegates, reserved for its live ones, and left; who funds it; where it
  went, by run and by delegate, largest first; its deadline; its
  authority in words: its tools, its grants (each a kind on a resource
  pattern: parts), what it may delegate, the note scopes it writes.
- **Connectors' panels:** a slot for each connector with something to
  say of the task (root.md, section 6).

## 6. A project

### 6.1 Its page

What needs a glance, each part opening onto its own page: what needs the
party in this project (the inbox, filtered); its goals, the top of the
board; its spend this period; and what is wrong, from its connectors, as
parts.

### 6.2 Goals

- **The board** is the project's live goals by priority, each a line: its
  chip and phase in words; its progress, its plan's tasks by phase; its
  spend against its budget, and its deadline; held below it, as a count.
  Waiting goals follow, then those ended recently, read a page at a
  time.
- **Moving a goal** up or down is a request, pending like any other.
  The board says what priority does: it orders what competes for one
  resource, as each connector uses it (`../../domain/core.md`, 6.2).
- **Setting a goal** from the board, for a goal the person knows well
  enough to skip a chat: what it is, in words; who carries it out, from
  the charters the deployment offers for goals; its budget, from the
  person's pool, saying what is left; its deadline and its priority.
  The project's document offers it with bounds (contract.md, 3.2): the
  charters offered and the pool left; the largest budget and the latest
  deadline the person may set alone; and the largest budget and the
  latest deadline any goal in the project may have. The form compares
  the draft with them as it is written, and says before it is sent
  whether the goal will be made, within the first; or, beyond it, what
  the offer says: proposed, and to whom, or refused, and why, as when
  nobody in the project may accept the proposal. Past the second, or
  with a charter not offered, it says the goal cannot be set, and sends
  nothing.

### 6.3 Tasks

The project's live tasks, filtered by phase, by executor, by goal, or by
whether the party asked for them, ordered by their last change. Ended
tasks are reached through their goal or chat, their history, or their
number.

### 6.4 Notes

What agents learnt (`../../domain/engine.md`, section 10), by scope: the
deployment, the project, a resource pattern, a goal. Each note shows its
description, its body, who wrote it, the tasks it names and when it last
changed. Correcting one names the revision read, and a note changed since
is refused as moved, with the newer one shown. Deleting one, likewise.
Searching by description.

### 6.5 Settings

- **Resources:** each adopted resource with its role (owned, fork,
  context), as a part its connector shows; adopting one is a project
  owner's request, made through its connector's form in a slot, as the
  application's own request (root.md, section 6).
- **Policy,** in words: its ceiling, its spend per period, its roles and
  what each may give, allot and decide, its requirements, and who decides
  what past a tree. A change shows before and after, and says that it
  applies to what is decided after it and never takes back what a task
  was given (`../../domain/people.md`, 5.2).
- **Roles:** who holds which role; seeded at adoption, changed by a
  project's owner. Services: the project's services and their roles; making one.
- **Spend:** the project's period, spent, reserved and left; each
  person's pool in it; where it went, by goal and chat, largest first;
  past periods, read a page at a time.
- **Recurring tasks:** each with its template in words, its period, its
  budget per period, its last batch and when it next runs. The engine
  keeps no calendar: periods are the application's to say, and a period
  says when the next begins only if its source knows
  (`../../domain/authority.md`, section 7). Where the current one said
  so, the line says when the next period begins; otherwise, that the
  task runs at the next period, with the current period and when it
  began.

## 7. The deployment

For a party with a role on the deployment:

- **Hosts:** the engine's own slots and each worker: connected or lost,
  and how long its grace still holds; slots used and free; the runs it
  hosts, as chips; a worker refused at its hello, and why.
- **LLM accounts:** each usable or not, and why; the runs using it; the
  tasks waiting for it.
- **Connectors:** each as a part: its outbox, effects waiting and those
  uncertain, its request budget, what its system last said.
- **Capacity:** live tasks against their limits, for the deployment and
  each project; pending proposals; what was refused for want of room,
  recently.

Lost workers, refused hellos and refusals for want of room are the
engine's diagnostics, kept in memory within its limits: after the engine
restarts, they start empty.

## 8. Pages that are not about the work

- **Starting:** until the frame's first delivery.
- **Signing in:** the application's providers; coming back to where the
  person was (client.md, section 12).
- **Missing:** an address the client has no page for, or a task the
  party may not see or that does not exist; the two are not told apart.
- **Out of date:** the engine does not serve this client's version
  (contract.md, section 8): the page says so, and how to update; what is
  pending stays in the small store until the client is updated.
- **Someone else's work:** the small store kept another party's pending
  requests (client.md, section 11): the page says so, and keeps them
  hidden. It shows while `jig-client` still counts kept requests of a
  party other than the one signed in, and goes when that party signs in
  again, or once each of their requests is retired as unknown, past the
  engine's retention.

## 9. What an application adds

- **Its pages,** one family's or several's, at its own addresses, under
  jig's frame: for `ops`, Incidents (open, then resolved, with each
  one's service, severity, lead task and timeline; an incident with no
  lead yet says so) and Environments (each pool, its slots, each
  environment's cost and end).
- **Its parts in jig's slots:** panels, results, effects' descriptions,
  holds' details, resources and kinds in words (root.md, section 6).
- **Its cards on jig's pages,** through those slots, and jig's cards on its
  pages: an incident's page shows its lead task's chip, the proposals
  waiting in its subtree, and its plan, as jig's cards (root.md,
  section 7).
- **Its navigation and its home** (section 2).

## 10. An incident, as the client shows it

The alert of `../../examples.md`, 7.1, as P, the person on call, meets it:

| Step | Where | P sees | P does |
|---|---|---|---|
| 1 | Incidents (ops's page) | a new incident, checkout errors, with I, its lead task, as a chip, running | opens it |
| 2 | I's page | I's conversation: logs read, metrics read, and R, a remediation of checkout, delegated, as a chip | watches |
| 3 | the inbox, and I's plan | a proposal card from R: restarting checkout in production, which neither R nor I may do, in infrastructure's words (parts), funded by P's acceptance | accepts it in I's plan, on R's line; the inbox entry goes with it |
| 4 | R's page | infrastructure's panel: restarting, then waiting for healthy (a part); R holds the checkout service | nothing |
| 5 | the incident's page | the incident resolved: I done, with its report, which went to W, its requester | reads it there |

Had someone restarted the service by hand while the restart waited for
P (`../../examples.md`, 7.3), R would have found it healthy and finished
without restarting it, and P would have seen the proposal leave the
inbox, withdrawn, with R's result saying why.

## 11. Open questions

- **Who may see what:** whether a member reads another's chats and
  transcripts, what an observer sees (`../../domain/people.md`,
  section 12); deferred while the client is built for one person. Each
  page shows what the engine lets the party read.
- **Digests:** what happened while the party was away, beyond updates.
- **A plan as a graph,** rather than "after" on each line.
- **Comparing attempts** of one task side by side.
- **Naming a chat:** by its person, or proposed by the chat; today it is
  its first words.
- **A board across projects,** for a party with several.
