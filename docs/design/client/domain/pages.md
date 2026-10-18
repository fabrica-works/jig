# Pages

Provisional, 2026-10-09, revised 2026-10-18. The reference UI's pages:
jig's optional pages, in `jig-client-view`, for `ops` and jig's test
application: how they are organised, what a party finds where, and what
they do there. They are built on the public client alone (`jig-client`
and its helpers, client.md, sections 4 and 19), and they are jig's
default arrangement of jig's primitives, which an application on the
reference UI uses as they are, rearranges, or composes into a tree of
its own, around its own pages. An application that does not take the
reference UI presents jig's objects from jig's documents itself (section
9). This document says what each page shows and offers, never how it
looks: no layout and no style. Section 12 says what a page is made of
(the frame's parts, the sections and `after_page`'s parts, each with its
region name), how an application arranges them, and what each section
opens (12.5). The cards that every page reuses are objects.md; the
reference UI's state (its place, the working set, the requests it sends,
drafts and the confirmation) is view.md, section 18; what each document
holds is contract.md. What is still open is listed in section 11.

## 1. In one page

- **The reference UI's, and optional.** These pages are
  `jig-client-view`'s, which `ops` and jig's test application take. They
  read jig's documents through `jig-client`, ask with jig's operations,
  and make chats and goals with jig's helpers, as any application's
  presentation may (client.md, section 19): nothing they show needs
  more than the API, so they prove the API is enough. An application
  that presents jig's work its own way takes nothing from them
  (section 9).
- **Three things people do.** They **talk** to agents (section 4),
  **decide** what waits for them (section 3), and **watch and steer** the
  work (sections 5 and 6). jig's pages are organised around these, not
  around jig's parts.
- **A page is an arrangement.** jig's pages are made of the frame's
  parts, sections and cards. What jig promises is kept below them: by
  the engine, which refuses a decision whose revision moved and answers
  a request once its commit is durable (contract.md, 5.2); by the
  client, which keeps each request until its answer is durable
  (client.md, section 6); and by the reference UI's state, which holds
  each object once and opens a confirmation for every decision, whatever
  node it was made on (view.md, 18.3 and 18.5). No promise depends on
  where a section or a card is, next to what, or in what order, so jig's
  pages are its default arrangement: an application takes them as they
  are, shows more sections together, arranges its own from the same
  primitives, or presents jig's objects itself (section 12).
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
- **Lists are task queries.** Every list of tasks a page shows (the
  conversations, a project's goals, a project's tasks) is one task
  query: live tasks first, watched; ended ones read below them a page at
  a time, through the engine's indexes of ended tasks (12.5;
  contract.md, 6.1). A page lists nothing the API does not serve.
- **Shapes are sugar.** A chat is started with `shapes::chat` and closed
  with `shapes::close_chat`, a goal set with `shapes::set_goal`: helpers
  filling the application's templates, as `me` serves them for a project
  (contract.md, 3.6), into jig's operations (client.md, section 19). A
  page knows a chat and a goal by their template's label and the task's
  flags, and asks nothing of the engine that names a shape.
- **One object, one card, many places.** A proposal shows in the inbox,
  in the conversation of the chat that made it, and on its proposer's
  page: the same card, with the same actions, deciding the same thing
  once (objects.md, section 2).
- **Every request is a decision.** It is pending until the commit that
  answers it is durable, and never shown as done before; refused, it says
  why; beyond what the party may do, the page says before it is sent
  whether it can be proposed, and to whom, and then sends a proposal, or
  that it cannot, and sends nothing; sent twice, it is made once.
- **What is committed is the truth; a run's text is provisional.** Pages
  show the engine's committed state, delivered as it changes; what a run
  writes is shown as it comes, marked so, and gives way to what was
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

- **The frame** is made of parts (section 12.1):
  - what needs the party, as a count, and updates not yet read, as a
    quieter one, as `me` counts them (contract.md, 5.1);
  - the project they are in, and a way to switch;
  - going to a task by its number;
  - the link's state, and whether what the page shows is behind
    (client.md, 8.2; view.md, 18.2), and who is signed in, with a way to
    sign out (view.md, 18.6);
  - the navigation: jig's pages and the application's.

  In jig's default arrangement what needs the party and the link's
  state are on every page, since a person must always know both, and so
  is who is signed in, since a person must always be able to sign out
  (view.md, 18.6); so are the other parts. Each is a primitive an
  application places where it likes, or omits (12.1); omitting one is
  the application's choice, and takes from the person what it shows:
  - *the link part* omitted, with no presentation of the link's state of
    its own, leaves the person not knowing whether what they see is
    live, behind or offline (client.md, 8.2);
  - *the party part* omitted, with no sign-out of its own, leaves the
    person no way to sign out, nor to learn that a sign-out did not reach
    the engine (client.md, section 12);
  - *what needs the party* omitted leaves them no count of what waits
    for them outside the inbox.
- **Places.** A place is where the person is: one of the reference UI's
  pages, with what it names (the inbox's project filter, the
  conversations' project, a task and its section, a project and its
  section, the deployment), or a place of the application's, which the
  reference UI knows only as elsewhere. It is the reference UI's value,
  `jig_client_view::address::Place`, and never the API's: the engine
  knows no page. Where the person was is among the reference UI's own
  records, which the application keeps for it (view.md, 18.8), so a
  reload, a sign-in or a terminal's next start comes back to it.
- **Paths are the application's.** A place's path in a browser, or its
  words on a command line, is printed and parsed by the application's
  codec. The reference UI's codec (`jig_client_view::address`) gives
  jig's places their paths, at the root, as
  `../../protocol/browser.md`, section 9, lists them: total, bounded
  step code, since a path is a person's input, and any path it does not
  know is a place with no page, shown as missing (section 8). An
  application takes it as is, under a prefix, with its own segment
  names, or writes its own, and keeps its paths apart from the prefix
  it mounts jig's default HTTP mapping at (`../../protocol/client.md`,
  17.1).
- **Home and navigation are the application's choice.** The application
  says which of jig's pages its frame shows, in what order, and which
  is home; the inbox by default (view.md, 18.1). Its codec says which
  place the path `/` is: the reference UI's codec, at the root, makes it
  the home. jig's navigation shows its own entries after them (12.1); an
  application that wants them among jig's builds its own navigation.
  `ops` makes the inbox home, and puts Incidents after it. A page that
  cannot apply is not shown: Conversations when no project of the
  party's has a chat template in `me`, the Deployment page to a party
  without a role on the deployment.
- **Links are by number.** Every task, wherever it is named (a chip, an
  entry, a history line, a turn), links to its page; its number is its
  address, never reused (`../../domain/tasks.md`, section 3). So does
  every proposal, through its proposer's page.

## 3. The inbox

What waits for the party (`../../domain/people.md`, section 6), shown
across every project they hold a role in. It is the `inbox` subject, the
people child's own index, not a task query: proposals, escalations and
questions are entries, not tasks (contract.md, 5.1 and 7.1).

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
- **Reading updates** is a request, a `Seen`: opening the updates moves
  the party's read position over results to the newest shown, and
  opening a chat moves its own position over that chat's turns
  (contract.md, 3.2).
- **Counted:** what needs the party, exactly; updates unread, up to a
  limit, past which the frame says "more than" it.
- **Nothing else is the inbox's own:** no archive, no snooze, no flag.
  What is done is in the history of the task it was about.
- **Live and bounded.** An entry appears as the commit that makes it
  waits for the party, and leaves as it stops waiting. The engine holds
  each party's entries up to a limit; the page reads the rest a page at a
  time.

## 4. Conversations

A chat is a task made from a project's chat template, as `me` serves it
merged for the project (contract.md, 3.6): an agent's task with a
conversation, under the template's label, whose requester is the person
(`../../domain/people.md`, section 7). Its page is a task page whose
activity, the conversation, comes first.

- **The list:** one task query (12.5): the person's tasks of the chat
  template's label, across projects or in one, by last activity; live
  ones first, watched; ended ones read below them a page at a time,
  through the engine's index of ended tasks by requester (contract.md,
  6.1). Each line is the chat's task line (contract.md, 5.1): its name,
  its project, what it is doing in words, its turns not yet read, what
  it waits on, its live delegates counted, and its spend.
- **Starting a chat:** a project and the first words. `shapes::chat`
  fills the project's chat template into a `Create` (client.md, section
  19): its executor the template's charter, and its budget the merged
  template's (jig's default `AsAllowed`: as much as the person's role
  and pool allow, the answer saying what was given;
  `../../protocol/configuration.md`, 3.1). The form says both and asks
  neither. A chat may also be started from what it is about (a task's
  page, a held task's card, a goal), with the thing named in its first
  words. Answered with the chat's number once durable, its page opens
  with its first run due. Refused, it says why: the person's pool, the
  project's live tasks, nobody to hold the chat's escalations in the
  project, a limit.
- **The conversation** (objects.md, section 10): turn by turn, across
  every run the chat has had: the person's words with their state, the
  chat's replies, its work folded to a line each, its calls to the engine
  as the cards of what they made, what it heard besides the person, and
  where its runs began and ended. Long conversations read from their end,
  a page at a time.
- **What it is doing,** always, in words: the chat's status, which the
  helper `facts` reads from its phase, its run and what it waits on,
  said with `words`. Which status applies is `facts`'s (client.md,
  section 19); the page says:

  | The chat's status | The page says |
  |---|---|
  | starting | starting |
  | writing | writing |
  | a tool | which tool, when its host names it, and for how long |
  | working | working, and for how long |
  | waiting for you | waiting for you, and for what |
  | waiting for them | waiting for them, by name |
  | parked | parked; your words resume it |
  | retrying | retrying, when, and why: "host lost, try 2 of 3" |
  | held | why, with its card |
  | closing; ended | closing; ended, and when |

- **Words** may be written while it is not closing or ended: a `Message`.
  A running chat hears them at its next turn, a waiting one wakes, a
  parked one resumes. They do not interrupt the turn being written; to
  stop it at once, the person stops it. Words to a chat the person
  stopped release it too: one compound request, the words and the
  release, which names the hold's revision the page showed (contract.md,
  3.4 and 5.2). Sent words are not edited or withdrawn.
- **Stop, close and cancel** are three requests, kept apart in name and in
  what each says before it is sent (objects.md, section 11). Closing is
  `shapes::close_chat`'s compound request.
- **Beside the conversation:** its project, its delegates live and ended
  as a small tree of chips, the goals it proposed (the person's, once
  accepted), its spend and what it has left, and its connectors' panels.

## 5. The task page

### 5.1 Its header

- **What it is:** its number, its title (its spec's first line), its
  project, whether it is tracked (a goal), and its priority if so.
- **Who carries it out:** an agent and its charter ("an incident lead");
  a procedure and its connector ("infrastructure's remediate"); a party
  or a role.
- **Where it stands:** its phase in words (objects.md, section 3).
- **Where it comes from:** its lineage, from the party or the deployment
  that rooted its tree down to it, each a chip.
- **What may be done:** the requests that fit its phase and the party's
  role (objects.md, section 11), as the task's document offers them
  (contract.md, 5.3).

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
  it writes alone, and which run holds the writer slot.
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

- **The board** is one task query (12.5): the project's live tracked
  tasks by priority, each a line: its chip and phase in words; its
  progress, its plan's tasks by phase; its spend against its budget, and
  its deadline; held below it, as a count. Ended goals follow, read a
  page at a time, latest first, through the engine's index of ended
  tasks by project: those of the goal template's label, which the
  template marks listed.
- **Moving a goal** up or down is a request, pending like any other: one
  compound request of priority-only amendments, one for each goal whose
  priority changes, committed together. It names no revision, and the
  first commit decides (contract.md, 3.4). The board shows its arrows
  where the project's document offers reordering (contract.md, 5.3),
  read beside the board's one query: it watches no goal's document,
  and its lines carry no acts. The board says what priority
  does: it orders what competes for one resource, as each connector
  uses it (`../../domain/core.md`, 6.2).
- **Setting a goal** from the board, for a goal the person knows well
  enough to skip a chat: what it is, in words; who carries it out, among
  the executors the project's goal template allows, a charter or a
  procedure the person's role may create with; its budget, from the
  person's pool, saying what is left; its deadline and its priority.
  `shapes::set_goal` fills the goal template with them (client.md,
  section 19). `me` offers creating in the project with bounds
  (contract.md, 5.3):
  the executors offered and the pool left; the largest budget and the
  latest deadline the person may set alone; and the largest budget and
  the latest deadline any goal in the project may have. The form
  compares the draft with them as it is written, and says before it is
  sent whether the goal will be made, within the first, and the helper
  gives a `Create`; or, beyond it, what the offer says: proposable, and
  to whom, and the helper gives a `Propose`; or refused, and why, as when
  nobody in the project may accept the proposal, and nothing is sent.
  Past the second, or with an executor not offered, it says the goal
  cannot be set, and sends nothing.

### 6.3 Tasks

One task query (12.5): the project's live tasks, filtered by phase class,
by executor kind, by goal (the tree it roots), or by whether the party
asked for them (the requester), ordered by their last change. Ended
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
  what each may give, allot and decide, its requirements, who decides
  what past a tree, and its overrides of the application's templates for
  this project. A change shows before and after, and says that it
  applies to what is decided after it and never takes back what a task
  was given (`../../domain/people.md`, 5.2).
- **Roles:** who holds which role; seeded at adoption, changed by a
  project's owner. Services: the project's services and their roles;
  making one.
- **Spend:** the project's period, spent, reserved and left; each
  person's pool in it; where it went, by goal and chat, largest first;
  past periods, read a page at a time.
- **Recurring tasks:** each with its recurring batch in words, its
  period, its budget per period, its last batch and when it next runs.
  The engine keeps no calendar: periods are the application's to say,
  and a period says when the next begins only if its source knows
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

- **Starting:** while the link is starting, until `me`'s first delivery
  (client.md, 8.2).
- **Signing in:** while the link is signed out (client.md, 8.2 and
  section 12). The page says how to sign in: by the printed link,
  opening the link the application's startup printed; and by the
  application's own ways of signing in, in a slot the page leaves for
  them, which the application's view fills with its own nodes, each
  leading to its own sign-in's route (contract.md, section 2; view.md,
  4.1). jig runs no sign-in of its own beyond the printed link and a
  service's credential. Once signed in, the person comes back to the
  place the reference UI kept (view.md, 18.6).
- **Missing:** a place the reference UI has no page for, or a task the
  party may not see or that does not exist; the two are not told apart.
- **Out of date:** while the link is out of date, the engine serving no
  version of jig's that the client speaks (client.md, 8.2; contract.md,
  section 8): the page says so, and how to update. What is pending stays
  in the client's small store (client.md, section 11); what the client
  sends of it, and when, is client.md's, 8.2.
- **Someone else's work:** the client restored requests kept for a
  party other than the one `me` names, and keeps them parked (client.md,
  section 11): the page says so, and keeps them hidden. The reference UI
  counts the kept requests the client root routes to it (root.md, section
  8) whose party, as `client.pending(key)` gives it, is not `me`'s; the
  page shows while that count is not zero, and leaves when that party
  signs in again, or once each of their requests is answered unknown,
  past the engine's retention.

## 9. What an application adds

- **Or nothing of the reference UI.** An application may present jig's
  work wholly its own way, on skein's UI kit or anything else, from
  jig's documents through `jig-client`: its lists task queries, its
  shapes jig's helpers or its own, its confirmations its own, a
  decision's facts read from the documents (contract.md, 5.4), and
  jig's check run in its worlds (testing.md, 6.2). What follows is for
  an application on the reference UI.
- **Its pages,** one family's or several's, at its own places, with
  jig's frame's parts where it places them: for `ops`, Incidents (open,
  then resolved, with each one's service, severity, lead task and
  timeline; an incident with no lead yet says so) and Environments (each
  pool, its slots, each environment's cost and end).
- **Its parts in jig's slots:** panels, results, effects' descriptions,
  holds' details, resources and kinds in words (root.md, section 6).
- **Its cards on jig's pages,** through those slots, and jig's cards on its
  pages: an incident's page shows its lead task's chip and the proposals
  waiting in its subtree as jig's cards, and its plan as jig's plan
  section (root.md, section 7); or its own presentation of jig's objects,
  from jig's documents (objects.md, section 2).
- **Its navigation, its home, and its ways of signing in** in the
  sign-in page's slot (sections 2 and 8).
- **Its arrangement** of jig's pages: the sections a task's page and a
  project's show together, and where each part and section goes
  (section 12.3).
- **jig's sections on its pages,** through slots: a task's plan or
  activity, the inbox's needs, the conversations (root.md, section 7).

## 10. An incident, as the client shows it

The alert of `../../examples.md`, 7.1, as P, the person on call, meets it:

| Step | Where | P sees | P does |
|---|---|---|---|
| 1 | Incidents (ops's page) | a new incident, checkout errors, with I, its lead task, as a chip, running | opens it |
| 2 | I's page | I's conversation: logs read, metrics read, and R, a remediation of checkout, delegated, as a chip | watches |
| 3 | the inbox, and I's plan | a proposal card from R: restarting checkout in production, which neither R nor I may do, in infrastructure's words (parts), funded by P's acceptance | accepts it in I's plan, on R's line; the inbox entry leaves with it |
| 4 | R's page | infrastructure's panel: restarting, then waiting for healthy (a part); R holds the checkout service | nothing |
| 5 | the incident's page | the incident resolved: I done, with its report, which went to W, its requester | reads it there |

Had someone restarted the service by hand while the restart waited for
P (`../../examples.md`, 7.3), R would have found it healthy and finished
without restarting it, and P would have seen the proposal leave the
inbox, withdrawn, with R's result saying why.

## 11. Open questions

- **Who may see what:** whether a member reads another's chats and
  transcripts, what an observer sees (`../../domain/people.md`,
  section 12). Each page shows what the engine lets the party read.
- **Digests:** what happened while the party was away, beyond updates.
- **A plan as a graph,** rather than "after" on each line.
- **Comparing attempts** of one task side by side.
- **Naming a chat:** a chat is named by its first words; whether its
  person names it, or the chat proposes a name.
- **A board across projects,** for a party with several.

## 12. Parts, sections and arrangements

What a page is made of, and how an application arranges it. Each part
and section is a primitive, a function of `jig-client-view` (view.md,
4.1) an application calls where it likes or not at all, and carries a
region name (12.4); what each section opens is 12.5. jig mandates
nothing in an application's tree but skein-ui's outline (view.md, 3.1).

### 12.1 The frame's parts

| Part | What it shows | Function | Region name | In jig's default |
|---|---|---|---|---|
| the link | the link's state (starting, live, offline, signed out, out of date), and whether the page is behind (client.md, 8.2; view.md, 18.2) | `frame::link` | `jig-link` | in jig's banner |
| what needs the party | what needs them, counted exactly, and updates unread, up to a limit, each leading to the inbox; before `me`'s first delivery, that nothing is known yet | `frame::needs` | `jig-needs` | in jig's banner |
| who is signed in | the party, and signing out (view.md, 18.6) | `frame::party` | `jig-party` | in jig's banner |
| the projects | the projects the party holds roles in, the one they are in marked current, each leading to its page | `frame::projects` | `jig-projects` | in jig's banner |
| going to a task | a field taking a task's number | `frame::go_to_task` | `jig-go-to-task` | in jig's banner |
| the navigation | jig's pages the application offers, in its order, then its own entries in a slot (section 2) | `frame::navigation` | `jig-navigation` | a landmark of its own |

- **jig's banner** (`frame::banner`) is jig's default banner: the banner
  landmark with the first five, in that order, on every page. An
  application may place each part in a banner of its own, another
  landmark, or nowhere (section 2 says what a person loses).
- **After the page,** three primitives, each building nothing when it
  has nothing to show; `after_page` is jig's default, calling the three
  last, on every page:

  | Part | What it shows | Function | Region name |
  |---|---|---|---|
  | the owed cards | each card the page's working set holds deciding, leaving, or unknown whether made (view.md, 18.3 and 18.4), that no section of the page and no card the application's page names shows (view.md, 4.1) | `pages::owed` | `jig-owed` |
  | the confirmation | the one open (view.md, 18.5) | `confirmation::confirmation` | `jig-confirmation` |
  | the notices | the notices, and dismissing them (view.md, 18.7) | `notices::notices` | `jig-notices` |

  Omitting one, with no presentation of its own, takes from the person
  what it shows:
  - *the confirmation* omitted leaves every decision on jig's cards
    unconfirmable: the reference UI sends nothing until its confirmation
    is confirmed (view.md, 18.5);
  - *the owed cards* omitted leave a request the person made, still
    deciding, leaving or unknown whether made, shown nowhere on a page
    that does not show its object;
  - *the notices* omitted leave the person unaware of what has no card:
    a request answered for an object the page does not show, a refusal
    with nothing to show it on, the link going and coming back
    (view.md, 18.7).

### 12.2 Sections

Each section is a value of the reference UI's `Section` (12.5), built by
`jig_client_view::sections::section` as a region with its level-2
heading. A header goes with its page's sections and needs nothing of its
own.

| Page | Section | `Section`, or its header's function | Region name |
|---|---|---|---|
| the inbox (section 3) | its header: what needs the party, counted, and the filters | `sections::inbox::header` | `jig-inbox-header` |
| | needs you | `Inbox { group: Needs, project }` | `jig-inbox-needs` |
| | updates | `Inbox { group: Updates, project }` | `jig-inbox-updates` |
| conversations (section 4) | the list, and starting a chat | `Conversations { project }` | `jig-conversations` |
| a task (section 5) | its header (5.1) | `sections::task::header` | `jig-task-header` |
| | its sections, one at a time | `sections::task::tabs` | `jig-task-tabs` |
| | overview | `Task { task, section: Overview }` | `jig-task-overview` |
| | activity | `Task { task, section: Activity }` | `jig-task-activity` |
| | plan | `Task { task, section: Plan }` | `jig-task-plan` |
| | history | `Task { task, section: History }` | `jig-task-history` |
| | budget and authority | `Task { task, section: Budget }` | `jig-task-budget` |
| | connectors' panels | `Task { task, section: Panels }` | `jig-task-panels` |
| a project (section 6) | its header: its name | `sections::project::header` | `jig-project-header` |
| | its sections, one at a time | `sections::project::tabs` | `jig-project-tabs` |
| | its page (6.1) | `Project { project, section: Overview }` | `jig-project-overview` |
| | goals (6.2) | `Project { project, section: Goals }` | `jig-project-goals` |
| | tasks (6.3) | `Project { project, section: Tasks }` | `jig-project-tasks` |
| | notes (6.4) | `Project { project, section: Notes }` | `jig-project-notes` |
| | settings (6.5): the five below, each a region nested in it | `Project { project, section: Settings }` | `jig-project-settings` |
| | resources; policy; roles and services; spend; recurring tasks | `Project { project, section: Resources }`, and `Policy`, `Roles`, `Spend`, `Recurring` | `jig-project-resources`, `jig-project-policy`, `jig-project-roles`, `jig-project-spend`, `jig-project-recurring` |
| the deployment (section 7) | hosts, LLM accounts, connectors, capacity | `Deployment` | `jig-deployment` |

Regions named inside sections, so a stylesheet places them too:

| Inside | Region | Region name |
|---|---|---|
| an agent's activity | the conversation, a log | `jig-conversation` |
| | its composer | `jig-composer` |
| conversations | starting a chat, a form | `jig-new-chat` |
| goals | setting a goal, a form | `jig-new-goal` |
| notes | searching them, a search | `jig-notes-search` |
| the deployment | hosts; LLM accounts; connectors; capacity | `jig-hosts`, `jig-accounts`, `jig-connectors`, `jig-capacity` |

The pages not about the work (section 8) are jig's default, each in the
main (`pages::not_the_work`): `jig-starting`, `jig-signing-in`,
`jig-missing`, `jig-out-of-date`, `jig-someone-else`. An application may
build its own from the reference UI's page (view.md, 18.2).

### 12.3 Arrangements

- **jig's default arrangement** of every page (`jig_client_view::jig`):
  the banner, with the link, what needs the party, who is signed in, the
  projects and going to a task, in that order; the navigation; the main,
  with the page's heading, its header, its tabs, and its sections; then
  the owed cards, the confirmation and the notices. A task's page and a
  project's show the place's own section, one at a time (a task's plan
  at its own place); the inbox shows its header and both groups;
  conversations and the deployment their one section.
- **More sections together.** An application may have every task's page,
  or every project's, show sections beside the place's own: its
  arrangement names them (12.5), every section of a task on one page for
  instance. The place's section is the one marked current, and jig's
  main shows them all in 12.2's order.
- **The tabs** list every section the task or the project offers, each
  leading to its place, the place's own marked current; jig's main
  leaves them out when the page shows every section its subject
  offers.
- **Its own arrangement.** At a place of jig's, an application's view
  root may build the main and any asides itself, from the page's
  heading, headers and sections and its own nodes (the budget in an
  aside; the composer pinned below the thread, by its stylesheet), and
  its banner from jig's frame's parts and its own (a brand, its own
  entries; the projects part only where its parties hold roles in more
  than one project).
- **What every tree keeps:** skein-ui's outline (view.md, 3.1), which
  the referee checks on every tree a person sees (testing.md, 6.3).
  jig adds no rule to it.
- **What jig's default keeps,** which the reference UI's tests check on
  each of jig's pages (testing.md, section 2): the link, what needs the
  party and who is signed in, in the banner, first; `after_page`'s parts
  last; jig's objects through their cards, whole (objects.md, section
  2); the pages not about the work, in the main.
- **What jig offers an application presenting its objects itself:**
  - a section only where the reference UI shows it on the page:
    elsewhere, its function builds nothing, whoever calls it;
  - jig's objects, through jig's cards or presented by the application
    itself (objects.md, section 2);
  - the link told, where the application shows jig's link part or its
    own presentation of the link's state (testing.md, 6.3).
- **On its own pages,** an application shows jig's sections and cards
  through slots (root.md, section 7), or presents jig's objects itself,
  with jig's frame's parts and `after_page`'s parts where it places
  them; jig's default places them as on jig's pages.

### 12.4 Region names

- **Language-free and stable.** Every region of 12.1 and 12.2 carries its
  region name, which a shell and a stylesheet place it by, never by the
  order of nodes or by its words. A name changes only by a change to
  this design.
- **Owned.** jig's start with `jig-`. An application names its own
  regions from a closed set of its own, starting with its own name and a
  hyphen, so one stylesheet holds both. In the tree a region name is its
  owner and a number; `jig_client_view::regions::JigRegion` holds jig's,
  each with its name.

### 12.5 What a section opens

- **A page is made of sections,** each a value of the reference UI's,
  `jig_client_view::Section`, naming its subject (12.2):

  ```
  Section::Inbox { group, project }     group: Needs or Updates; project: the inbox's filter, or none for every project
  Section::Conversations { project }    the party's chats, and starting one, in that project or any
  Section::Task { task, section }       section: a place's TaskSection
  Section::Project { project, section } section: a place's ProjectSection
  Section::Deployment
  ```

  `Ui::sections()` lists the page's sections, in 12.2's order. Each
  section reads its subject from itself, never from the place, and the
  view builds it from the reference UI's state and the client's
  documents for that subject.
- **What each opens,** through the client, under the reference UI's own
  handles (view.md, 18.2): what is live is watched, and what is past is
  read, a page at a time, as the person asks (contract.md, sections 6
  and 7):

  | Section | Watches | Reads |
  |---|---|---|
  | the inbox's groups | `inbox`, with the section's project | its entries past the window |
  | the conversations | `tasks{query}`: the party as requester, the chat template's label, the section's project if any, live, by last activity | the same query, ended, by when they ended: the index of ended tasks by requester |
  | a project's goals | `tasks{query}`: the project, tracked, live, by priority | the project's ended tasks of the goal template's label, by when they ended: the index of ended tasks by project; the project's document, for its offer of reordering |
  | a project's tasks | `tasks{query}`: the project, live, by last change, the section's filters as the query's keys (phase class, executor kind, root, requester) | |
  | a task's every section | `task{n}` | |
  | its plan | `tree{root}`, the task its root | ended branches, as they open |
  | its activity | `run{n}`, while a run is live | `activity{n}`, from its end |
  | its history | | its history, for the task or its subtree |
  | a project's overview | `inbox`, filtered by the project; the goals' query, its first rows | the project's document, for its spend |
  | its settings and notes | | the project's documents and its notes, again when a watch says they moved |
  | the deployment | `deployment` | |

  Beside them, every page watches `chips` for the tasks it names in
  passing, and the reference UI watches `me` always (view.md, 18.2).
- **Opened once per page,** each watch and read opened once for the page
  however many sections need it (a task's watch, for each of its
  sections), and closed when the page leaves. The working set is their
  union, each object held once whichever sections show it (view.md,
  18.3).
- **Each section's windows are its own:** every window a page holds has
  a handle no other window of the page has, so reading more in one
  section never reads another's, and a page read goes to the window
  that asked.
- **One subject each.** A page's sections name at most one task, one
  project and one inbox filter; a project's overview shows the inbox's
  needs of its project, so it names that filter too. They share the
  reference UI's state for that subject (one task's header, plan,
  history and conversation; one project's; the inbox's; the
  deployment's), as its objects share the working set. A section naming
  a second one is not shown, and the view's slot for it says it cannot
  be shown (view.md, section 4).
- **Each section once.** A project's `Settings` holds its five
  (`Resources`, `Policy`, `Roles`, `Spend`, `Recurring`): a page that
  would show `Settings` and one of the five shows `Settings` alone.
- **At a place of jig's,** the page shows the place's own sections (the
  inbox's two groups, with the place's filter; a task's or a project's
  section at the place; the conversations or the deployment) and,
  beside them, those the application's arrangement adds.
  `jig_client_view::Arrangement` holds two lists, of `TaskSection` and of
  `ProjectSection`: the sections every task's page, and every
  project's, shows beside the place's own. The application gives it
  when it makes the reference UI, as it gives the navigation (view.md,
  18.1), which refuses a list longer than its kind's sections, naming
  one twice, or naming `Settings` with one of its five. jig's own
  arrangement adds none: one section per place. A task's section its
  document does not offer is not shown, wherever it was named.
- **At an application's place,** the page shows the sections the
  application's page names (`Named`, view.md, 18.1; root.md, section 7).
  Each list stands for the whole of what the page names, and an empty
  one names none. The reference UI opens and closes what the difference
  needs, under handles of their own, so naming objects anew never closes
  a section's watch, nor naming sections an object's.
- **What a page says of itself, a section says on an application's
  page:** that its watches are behind, that rows past its window were
  left out, that a limit refused what it asked, and that its subject is
  missing (one the party may not see, or none). Each section of jig's
  there says it in its own region, where at a place of jig's the page
  says it (view.md, 18.2; section 8).
