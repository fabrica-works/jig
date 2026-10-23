# Parties

Provisional, 2026-10-07, revised 2026-10-23. How people, and the
services that act like them, take part: who they are and the credentials
they sign in with, what their role in a project lets them do, the
operations they ask of the engine and what waits for them, the
conversations they hold with agents, the tasks they carry out
themselves, and how they hear of what waits. Parties are requesters and
executors, as tasks are (core.md, section 9); this document is that
party in depth. The client they use is jig's, `jig-client`, which an
application holds under its own presentation (`../client/domain/`). The
LLM accounts a person holds, which pay for runs, are accounts.md's. What
is still open is listed in section 12.

## 1. In one page

- **A party is a person or a service.** People create tasks and talk to
  them, accept proposals, answer questions, steer and watch through a
  client. A service does what its role allows through the same
  operations, with no presentation.
- **Known by a credential jig mints.** jig mints, verifies, expires and
  revokes credentials. The printed link and services' credentials are its
  own ways of signing in; any other is the application's sign-in, which
  ends in a `Mint` on the application's server's word. The core keeps who
  a party is, as a provider and a subject; it does not act on any system
  as them.
- **Roles are per project,** the core's own, possibly seeded from what a
  connector knows of people on an adopted resource. A role is an
  authority and the proposals it may decide (authority.md, section 6).
- **Requests are jig's operations.** Create a batch, propose, withdraw,
  message a task, amend, cancel with or without a grace, release, stop,
  move a task to oneself, decide, answer, take and hand back, mark seen:
  each is checked against the party's role as an agent's call is against
  its task's authority, committed, and answered once durable. Ordinary
  deciding acts retain their original answers under their keys for the
  advertised retention; `Seen` retains no new answer and advances a read
  position only forward (5.1 and section 6). A compound request holds
  several, committed all or none.
- **Proposing is explicit.** A request beyond the party's authority is
  refused, naming what it lacked and whether a proposal could be
  accepted; `Propose` asks.
- **The deployment's acts** are what a connector asks of the core in
  the deployment's name, where no client is there to ask: a person's
  conversation closed, the deployment's key added or account removed,
  a resource adopted, a project made. No party makes them: a connector
  the configuration grants each kind, while it names a first owner,
  keyed as a party's request and answered to it once durable (5.3).
- **A party's inbox** holds what waits for them: questions, proposals,
  escalations, the tasks they carry out, results and the turns of their
  conversations. What is addressed to a role waits for every party in it
  until one acts.
- **A chat is a template,** jig's default for talking to an agent: an
  agent's task flagged conversation and kept, filled in by the client and
  closed by cancelling it with a grace. The task carries its flags; the
  engine never expands or checks a template.
- **The client knows the plumbing:** these operations, tasks as a tree,
  what waits for a party, and live runs; not the shapes of work built
  from them, which are templates and helpers above it. An application
  adds its own kinds of object.

## 2. Structure

The people child domain is a child of the core (engine.md, section 3), a
capability. It keeps:

- **parties:** the identities of those signed in or holding a role, and
  their sign-ins, each named by its number;
- **roles:** each live project's roles and who holds them, loaded at
  start and changed by commits;
- **requests in flight:** each party's pending requests, and each
  connector's pending acts in the deployment's name (5.3), by their
  keys, until answered, a compound request as one;
- **inboxes:** for each party with something waiting, what waits, by
  reference to the task or proposal it is, and their read positions over
  results and over live conversations (section 6).

It knows nothing of tasks beyond their numbers and what the core tells
it waits for whom; every request goes to the core, which routes it to
authority and to the tasks. Parties' watches of their `me` and their
inbox are the views child's (engine.md, section 11).

## 3. Parties and identity

- **An identity is an identity provider and a subject.** An identity
  provider, a *provider* in this document and in the configuration's
  `providers`, is one of the application's ways of signing in, numbered
  in its configuration (`../protocol/configuration.md`, section 3): an
  OAuth server, a company's directory, a chat platform's identity, or
  the deployment itself, for services' credentials. The subject is the
  provider's stable id for the party; a login and a name come with it,
  for display only.
- **An LLM provider is no identity provider.** OpenAI, Anthropic or a
  local model's server is configured and numbered apart
  (`../protocol/configuration.md`, section 3), and signs nobody in to
  jig. An LLM account's subject is the LLM provider's for the account,
  never a party's identity: signing an LLM account in changes no one's
  identity, and one person may hold several (accounts.md, 3.1 and 3.3).
- **jig owns credentials.** It mints a credential for an identity,
  verifies each one presented to its party's sign-in, expires it after a
  configured time, and revokes it when its party signs out, or when an
  owner reissues or removes a service's (below). A sign-in is committed
  by the core, which gives it its number; the credential's digest is
  kept by the clients' component in the store's secrets
  (`../protocol/store.md`, section 9), so a restart signs nobody out.
  The domain names a sign-in only by its number, and says each that ends
  or expires, so the component erases its digest.
  A sign-in expires when either its original monotonic lifetime or its
  saved wall expiry is reached. Both clocks make its people timer due;
  a backward wall correction never extends the original monotonic
  lifetime. Creation and restoration arm both expiry conditions; timer
  scheduling translates the saved wall expiry at the current supplied
  clocks (root.md, section 3; engine.md, section 11).
- **Two ways of signing in are jig's own:**
  - *the printed link,* on one person's machine: the clients' component
    draws a one-time secret at start, which the application's startup
    prints in a URL of its own; presenting it signs in the configured
    first owner it names, and spends it (`../protocol/client.md`,
    section 8). An application enables it by configuration, on loopback
    only;
  - *a service's credential* (below), presented to sign in as that
    service.
- **A sign-in held is kept.** A person signing in by the printed link
  while presenting the credential of a sign-in of their own keeps that
  sign-in, rather than making another beside it:
  - *the clients' component authenticates it* as it does any exchange's
    credential, its digest under its number and its saved expiry
    (`../protocol/client.md`, section 8), and names that sign-in to the
    core as *held*; one unknown, ended or expired is held by no one,
    and signs nobody out;
  - *people decides whether it is the same person's:* a held sign-in
    whose person is the one the identity signs in, live by both its
    clocks, is the sign-in's answer, with its own number and its saved
    expiry, and nothing is written. Anything else (no sign-in held,
    another person's, one past either clock) signs in afresh, as
    without one;
  - *it extends nothing:* the kept sign-in's expiry stays as it was
    saved, and its credential is the one the person holds, so none is
    drawn, kept or handed out;
  - *before capacity:* the held sign-in is checked before the
    deployment's sign-ins are counted, so signing in again by a fresh
    link never fills them;
  - *the numbers drawn go unused:* the candidate person and the sign-in
    number the core drew for the exchange are kept by no one, and the
    core's next commit saves its counters past them, as after a sign-in
    refused as busy. A gap in the numbers is harmless: they name, and
    are never reused or counted; capacity counts the sign-ins held;
  - *checked once, by people:* a held sign-in that ends or expires
    after the clients' component checked it, before people answers, is
    no longer live there, and the person signs in afresh.

  Only the printed link offers a held sign-in: a `Mint` and a service's
  sign-in sign in afresh.
- **Full sign-ins say so.** The deployment holds at most
  `limits.people.sign_ins` sign-ins (`../protocol/configuration.md`,
  section 6). A sign-in that would make one more is answered *full*,
  apart from busy:
  - *what it carries:* that limit and the earliest saved expiry among
    the sign-ins held, the wall moment by which one will have ended
    unless its person signs out sooner. No person, number or count: the
    printed link's holder, the deployment's first owner on its own
    machine, learns when the next room comes, and nothing of whose it
    is; a `Mint` and a service learn only busy (`../protocol/client.md`,
    section 8);
  - *after the held sign-in:* a person keeping their own sign-in (above)
    is never answered full;
  - *only while every sign-in is live:* the deployment's sign-ins are
    counted as held, and one past either clock is held until its timer
    ends it. A sign-in that finds them full while one of them is past
    either clock ends that one there, the one its timer would end next,
    as the timer would (its digest erased, its end said), and takes its
    room: ending what has expired is no eviction. So full is answered
    only while every sign-in is live by both clocks, and its earliest
    expiry is always to come;
  - *it waits for an end:* no wait of seconds frees it, only a sign-in
    that ends: its person's sign-out, its expiry, or an owner's removal.
    Nothing ends, shortens or grows to make room;
  - *nothing is written,* and the numbers drawn go unused, as for busy.
    Full is an answer, never saved, as busy and not ready are not.

  *Busy* stays for what passes or is the deployment's own: a new identity
  while the people table is full, and the bootstrap's holdings while it
  seeds. Neither meets the printed link past bootstrap: its first owner
  is not new.
- **Every other way is the application's sign-in.** It runs on the
  application's own routes, redirects included, and verifies who the
  party is. Its server then gives the clients' component a `Mint` naming
  that identity: the provider's number, the subject, a login and a name.
  The core commits a sign-in for it, and the component answers with its
  credential, which the application carries as it chooses (a cookie, a
  bearer). jig takes the server's word for the identity, so a `Mint`
  comes from the application's server code, never from a client. It
  names a provider the configuration gives as the application's; one
  naming any other, the printed link's or the deployment's, is refused
  (`../protocol/configuration.md`, section 3; `../protocol/client.md`,
  section 8).
- **The domain sees an authenticated identity,** and the kind of party it
  is, never how it was verified. jig is given no identity provider's
  token: what the application's sign-in received is the application's,
  and jig acts on no system as a party (section 12). An LLM provider's
  record is an LLM account's, kept by the protocol layer and lent only to
  pay for runs (accounts.md, 3.4; `../protocol/store.md`, section 9).
- **No acting for.** A credential is one party's own. A bot relaying a
  person's decision holds that person's credential, obtained through the
  application's sign-in, and the decision is the person's request,
  naming the revision the bot showed them.
- **A party record** is made the first time an identity signs in, and
  kept.
- **Services** are parties whose provider is the deployment: an owner
  makes one in a project, with a role, and jig mints its credential. A
  service makes the requests a person makes (section 5), and its inbox
  is read by its program, not shown.
  - *Its identity* is the deployment and the service's number, recorded
    when the service is made, so its sign-ins find it; a sign-in of the
    deployment's naming no service makes no party.
  - *An owner reissues its credential:* a new one replaces it, and the
    sign-ins made with the old end. This is how an owner who lost the
    credential, or fears it leaked, gets one again; the party, its role
    and its history stay.
  - *An owner removes it:* its credential and its sign-ins end, and its
    party is kept, as every party is, for the history that names it.
- **The first owners** of a deployment are named in its configuration;
  everyone else gets a role from a project's owner, or by adoption's seed
  (section 4). The first owners decide the deployment's LLM accounts
  (accounts.md, 3.2).
- **The deployment's roles** are a roster of their own, kept as
  project 0's: a party's role in the deployment, on which the
  deployment's scope is admitted (`../client/domain/contract.md`, 7.5).
  - *The first start* makes each first owner its owner, whether or not
    configuration seeds a project. It is seeded then only: a later start
    reads it from the store, and seeds it into no store that lacks it
    (`../protocol/configuration.md`, section 8).
  - *Project 0 is no project.* Configuration names none
    (`../protocol/configuration.md`, section 3), and a party's projects,
    in their frame, in `me` and in an account's readiness, leave it out:
    it is read only as the party's role in the deployment, `me`'s
    `deployment_role`.
  - *Its roster counts* against people's `projects`, as a project's
    does, and never against authority's: it has no policy.
  - *Its owners make projects* (5.1 and 5.2), and hold no role in a
    project by it: a project's roles are its own roster's, which a first
    start fills with the first owners because configuration names them.

## 4. Roles

Each project has its roles, and its policy says what each may do
(authority.md, section 6). The default roles:

| Role | May |
|---|---|
| owner | everything a maintainer may; rename the project; change the project's policy and roles; adopt resources; make services |
| maintainer | create and steer any task in the project; order its goals; decide any proposal within the project's ceiling; release, cancel, stop |
| member | create tasks, chats and goals among them, within the authority and allotment their role gives; steer and cancel what they started; answer what is addressed to them |
| observer | read and watch |

- **A role is an authority** (authority.md, section 3): what a party in
  it may give a task they create, how much of the project's spend they
  may allot, and which kinds of proposal they may decide. The policy may
  narrow or widen each, and add roles of its own (the person on call),
  within the deployment's rules.
- **Seeded at adoption.** When a resource is adopted, its connector may
  say which parties it knows there and with what permission (a
  repository's collaborators, a team's members). The project's policy
  maps permissions to roles, and those parties are given them, unless
  they already have a role in the project. The party who made the project
  is its owner. After that, roles are the core's own: a permission that
  changes on the system does not change a role.
- **Proposals past the root** of a tree go to the role the policy names
  for that kind of proposal (tasks.md, section 9), maintainers by
  default.
- **Who holds escalations.** A held task's escalation that passes every
  task above it reaches the party who requested its tree, when their
  role decides escalations, and past them the project's escalation role,
  if its policy names one (authority.md, section 6; tasks.md,
  section 9). A party may be the requester of a root only when one of
  the two holds it (5.1).

## 5. What a party does

### 5.1 Requests

A party's requests are jig's plumbing: its primitives as operations, the
same an agent reaches through its tools (engine.md, 7.3), and checked by
the same rules (authority.md, 8.1 and 8.4). Every request is keyed by
its client (a double click makes one request), checked against the
party's role, committed, and answered once durable (engine.md, 5.2). A
task is in a party's tree when they requested it, or a task above it.

Ordinary deciding requests reserve a place for their original answer at
admission and retain it for the advertised retention, success and
permanent routed refusal alike. This capacity is shared engine-wide,
not per party. Unexpired answers are never evicted for another key.
Busy and not ready are not saved. After authentication and validation,
retained-key replay or conflict and pending-key joins or conflicts are
checked before a new request needs room. An identical pending request
joins its waiters; a different ask under that key conflicts.

A new top-level `Seen` uses the same keyed transport and pending checks,
but bypasses completed-answer capacity and saves no `Answer`, on success
or routed refusal. It still needs the existing bounded pending, flight,
waiter, load, journal and output room. Ordinary admission counts retained
answers plus only the ordinary flights that reserve answers, excluding
`Seen`; a bounded scan of pending flights suffices, with no new counter
or limit. Each admitted ordinary flight thus retains its reserved place
through saving its answer.

After a new `Seen` completes, its key is free. A retry is processed
against the current subject and authority, and a different ask under the
freed key is a new request. Legacy retained `Seen` answers still restore,
replay or conflict until their original expiry, occupying their old
answer places without early erasure or reclassification. New `Seen`
requests can progress while those places are full.

The operations as the client sends them are
`../client/domain/contract.md`, 3.2:

| Operation | Needs | Becomes |
|---|---|---|
| `Create`, a batch | member, within their role's authority and their pool (authority.md, 8.4); its dependencies and inputs within their reach, and a holder for its escalations (below) | its tasks, with their dependencies, checked whole and made at once (tasks.md, section 4), each with the fields it names (tasks.md, section 3); the party their requester and their funder; a budget given `AsAllowed` resolved, and what was given answered (authority.md, section 7) |
| `Propose`, an action | member; a batch or an amendment beyond their authority, of a kind a role the project's policy names may accept | a proposal of theirs, waiting for that role (tasks.md, section 9), its number answered |
| `Propose`, a change of settings, in the deployment's scope | a role in the deployment or a project; the revision they saw; an owner who may accept each item (engine.md, 17.6) | a proposal of settings, waiting for those owners, its number answered |
| `Withdraw` | their own pending proposal | withdrawn (tasks.md, section 9) |
| `Message` | the task in their tree, or maintainer | words in its inbox (tasks.md, 8.1), their message number answered |
| `Amend` | the task in their tree, or maintainer, and a change within their authority; the spec's revision they saw, but for a change of priority alone; maintainer, for tracked or priority; dependencies added within their reach, as a batch's | tasks.md, section 7: its spec, instructions, authority, wake policy, a person task's address, whether it is tracked, its priority, its dependencies added or removed; a widened task funded from their pool (authority.md, section 7) |
| `Cancel`, with or without a grace | the task in their tree, or maintainer; a grace within the tasks child's bound (tasks.md, section 11) | the task and its tree closing (tasks.md, section 7); with a grace, the task first asked to finish, and cancelled when the grace ends unless it ended first |
| `Release` | the task in their tree, or maintainer; the hold's revision they saw; for an effect held uncertain, whether to make it again | its hold lifted, the task judged again (tasks.md, 5.5) |
| `Stop` | the task in their tree, or maintainer | its run cancelled and its task held for a party's stop |
| `Move` | they may amend the task; a holder for its escalations (below) | the party its requester, funded from their pool (tasks.md, section 7) |
| `Decide` | the proposal or escalation waiting for them, as the requester of its tree or in a role the policy names; for an escalation, its hold's revision; for a proposal of settings, whether running work moves | accepted, if their authority covers it, and funded from their pool (authority.md, section 9: a batch's tasks, an effect's price, the task an amendment or a widening changes), or, for a proposal of settings, by an owner of each item it names, funding nothing (engine.md, 17.6); rejected, with a reason; or passed up (tasks.md, section 9) |
| `Answer` | addressed to them or their role; a person task's revision as they saw it; for a person task, none of its dependencies pending (section 8) | an answer to a question, or a person task's result (section 8) |
| `Take`, `HandBack` | a person task addressed to a role they hold | held by them, so no other answers it; or back to the role (section 8) |
| `Seen` | the party themselves | their read position moved over results, or over one conversation's turns (section 6) |
| a compound request | one project; each operation's own needs; within operations per request | its operations made in one decision, or none of them (below) |
| adopt a resource, as a home or not; rename the project; change the policy, roles or services | owner; a name no other project holds, in shape (core.md, 6.1) | the project changed (5.2) |
| `ChangeSettings`, in the deployment's scope | owner in the deployment for the deployment's items, and owner of each project whose items it names (section 3 and 4); the revision they saw, the newest; the set within the checks and bounds of engine.md, 17.1 and 17.8; every connector judging an item of its kinds accepting it | the settings changed in one decision, the new revision answered; running work staying or moved, as it asks (engine.md, 17.2 and 17.4) |
| `MoveToNewest`, in the deployment's scope | owner in the deployment; a revision held | every live agent task on it that may move set to the newest, in one decision, those that could not counted in the answer (engine.md, 17.4) |
| `MakeProject`, in the deployment's scope | owner in the deployment (section 3); a name in shape that no other project holds (core.md, 6.1); a policy among the settings (engine.md, 17.1); room in the deployment's projects | a project, numbered next, its policy a copy of the configured one, its maker its owner and only party (5.2) |
| edit or delete a note | the note's scope within their role | the note revised (engine.md, section 10) |
| set a party's pool | owner | the pool opened or resized in the current period, what was spent and reserved kept |
| an account request: sign in, sign out, add, replace or remove a key, declare or check a program's login, order, uses, resume, remove an account | the account's holder, not a role: a person for their own, a first owner for the deployment's (authority.md, 8.4) | accounts.md, section 5: the account's sources, states and uses changed; a value never committed |

- **Words are never cut.** A party's words, in a `Message` or answering
  a question, are within the people child's bound, which is at most what
  a run admits as one message (tasks.md, 8.2; engine.md, section 8); a
  `Create`'s spec is within the tasks child's too (tasks.md,
  section 11). Words past either are refused at admission as too large,
  saying the bound.
- **A request refused** says why, with its facts: the role, the
  authority it lacked and by how much, and, where their role and pool
  decided it, whether a proposal could be accepted; a limit; the
  object's revision since; a task since ended
  (`../client/domain/contract.md`, section 4). Who would accept a
  proposal is the offer's to say (`../client/domain/contract.md`, 5.3).
- **Proposing is explicit.** A `Create` or an `Amend` beyond the party's
  authority is refused, naming what it lacked and whether a proposal
  could be accepted; it never becomes a proposal by itself, so a person
  is never asked by accident, as a task's calls never are (engine.md,
  7.3). A service is a program, so the same holds for it. `Propose` asks,
  with a reason, and the party is its proposer: its decision, a
  rejection included, reaches them. A party proposes a batch, or an
  amendment, a widening of a task's authority among them, or a change
  of settings (engine.md, 17.6); an effect and
  a widening of a proposer's own authority are what tasks propose, since
  a party makes no effect, and their own authority is their role, which
  the policy sets (5.2; authority.md, section 9). Each document offers
  creating and amending as made, proposable and to whom, or refused, with
  bounds (`../client/domain/contract.md`, 5.3), so a client knows which
  before it sends anything; proposing implicitly is sugar above the
  plumbing, never the engine's.
- **A proposal needs someone who may accept it.** A party's proposal
  waits for the roles the project's policy names for its kind
  (tasks.md, section 9). When the policy names none, `Propose` is refused
  at admission as lacking authority, saying that no proposal could be
  accepted; it is never committed as a proposal nobody may decide. A
  proposal of settings waits instead for the owners of its items, in
  the deployment and in the projects it names, and is refused so when
  there are none (engine.md, 17.6). A proposal already pending, and the proposals tasks make, wait as
  tasks.md, section 9 says.
- **A party's root has someone to escalate to.** Every task a party's
  `Create` makes is a root, the party its requester. A party becomes a
  root's requester, by `Create`, by `Move`, or by accepting a proposal
  whose tasks are to be theirs, only when its escalations have a holder:
  their role decides escalations, or the project's policy names an
  escalation role (section 4). Otherwise the request is refused, saying
  so, as a proposal nobody may accept is: a held task never waits for
  nobody.
- **Where a party reaches.** A party's `Create` may name, as a
  dependency, a live task, or, as an input, an ended task's result: a
  task in their tree, or any task of the project for a maintainer, as a
  task's batch reaches its delegates and the tasks it was introduced to
  (tasks.md, section 4 and 8.5); a batch gives its tasks no references.
  Their other operations reach what the table says.
- **What only parties do,** by design: carry out person tasks (`Take`,
  `HandBack`, and `Answer` with a result), stop runs, move tasks to
  themselves, mark what they have seen, make and run projects (5.2), and act
  on the LLM accounts they hold (accounts.md, section 5).
  Procedures decide nothing (tasks.md, section 9).
- **A decision names what it saw.** A release, an amendment, a decision
  about an escalation and a person task's answer carry the revision the
  party was shown; one whose object has changed since is refused as
  moved, saying what it is now. A change of priority alone names none:
  the first commit decides.
- **A hold's revision moves with the hold:** when the task is held anew,
  or its reason changes; never when its escalation passes up. A release,
  and a decision about the escalation, name it, whatever held the task,
  the releasing party's own stop among them. Words release nothing: a
  `Message` adds words to the inbox, and words to a task the party
  stopped come with a `Release` in one compound request (below), naming
  the hold's revision as it would alone. Where the escalation waits is
  checked apart: deciding the escalation needs it to wait for the
  party, and is refused as not waiting for them otherwise; releasing,
  amending or cancelling the held task needs the task in their tree, or
  maintainer, wherever its escalation waits. So a release named with the
  revision from before a pass up is made, not refused as moved.
- **Two parties deciding one thing:** the first commit decides; the
  second is told it was decided, by whom, and how.
- **A compound request** holds several operations under one key, for a
  decision made of several: closing a conversation whose delegates are
  live, each cancelled or moved to the party, and the conversation then
  cancelled with a grace (section 7); ordering several goals at once;
  amending a held task and releasing it; words to a task the party
  stopped, and its release.
  - *What it may hold:* `Cancel`, with or without a grace; `Move`;
    `Amend`, priority included; `Release`; `Message`; `Stop`. The other
    operations keep requests of their own.
  - *One project:* every operation's task is in the request's project.
  - *In order:* its operations are admitted in order, each against the
    state those before it leave and needing what it would alone, with one
    pool counted across them (a move's reservation and the rest).
  - *All or none:* they commit in one decision. The request has one key
    and one answer: all made, or refused, naming the first operation that
    failed and why, and nothing of it made.
  - *Numbered by place:* its operations take consecutive message
    numbers, one each, in the request's order, and the answer names the
    first, so the operation at place *i* took the first plus *i*: a
    `Message`'s words are numbered so, and an operation that adds nothing
    to its task's inbox leaves its number unused. Nothing more of an
    operation is answered (`../client/domain/contract.md`, 3.4).
  - *Revisions:* each operation names its revision where it would alone,
    so a `Release` names its hold's beside words or an amendment as it
    does alone; a change of priority alone names none, and the first
    commit decides.
  - *Bounded* by operations per request, a limit of this child's, never
    so small that closing a conversation with every delegate disposed
    of, or ordering every goal of a project, takes more than one
    request; configuration checks its floor
    (`../protocol/configuration.md`, section 6).
  - *Its worst case* is engine.md's, section 13.
- **Requests about an application's own objects** (an environment's end
  moved, an incident's page) are the application's: its client sends
  them, its root routes them to the connector they concern, and what they
  change in the core is an operation of the table above, judged as the
  party's own. This child keeps them by key as it keeps its own:
  admitted on the party's sign-in and, by the scope the request names,
  their role in its project, their role in the deployment, or nothing
  more for the party's own (`../client/domain/contract.md`, 3.3 and
  7.5); joined when sent twice; and their answer, the connector's
  outcome, kept under the key by token. One it refuses at admission
  (signed out, the party's role in the request's scope, a key sent again
  with another request, a limit) is answered with jig's refusal and never
  reaches its connector; only an admitted request is the connector's to
  decide, and answered with its outcome. A connector's core part carries
  no model-name context: an explicit model selector (ALL or named,
  including empty) is refused at people admission with `Lack::Models`;
  an omitted selector keeps the operation's default.
- **A watch is not a request:** it decides nothing and keeps no answer.
  It is admitted on the party's sign-in and on what their role lets them
  see in its scope, the party's own on the sign-in alone, and ends when
  they leave (engine.md, section 11).

### 5.2 The project itself

A project, its name, its resources and roles, its homes and its policy
are records in the store that owners change through requests like any
other. A rename reaches every client through `me`
(`../client/domain/contract.md`, 5.1) and changes nothing else (core.md,
6.1). A policy change applies to every decision after it, the decisions
of tasks already made included: a narrowing makes their grants beyond it
unusable, and a widening does not widen them. What was committed before
it is made, and pending proposals are judged against it when accepted
(authority.md, section 6, effective authority). An owner who wants work
in flight stopped cancels it. The policy's overrides of the
application's templates (authority.md, section 6) reach only what
clients create after them: a task keeps the values it was made with
(`../protocol/configuration.md`, 3.1).

A project made after the first start (`MakeProject`, 5.1) is made in
the decision that admits the request, in one commit with its answer:

- **its number,** the deployment's next (core.md, 6.1);
- **its name and its policy,** the configured policy as it stands then,
  copied: a later change to the configured policy, or its removal,
  reaches no project made from it, and its owners change it as any
  project's;
- **its roster,** its maker alone, as owner (section 4). Other owners
  of the deployment get no role in it; the maker gives roles, adopts
  resources whose connectors seed them, or makes services;
- **its first period,** its budget the policy's spend per period, with
  no next start, as the smallest period source opens a seeded project's
  (`../protocol/configuration.md`, section 5), and with it the maker's
  pool. The application's root hears that the project was made, and a
  period source that keeps a calendar says its periods from then on, as
  for every project.

A crash keeps the project and its answer, or neither; a retry under the
key then makes it once. Two makes naming one name: the first commit
takes it, and the second is refused as *name taken*.

### 5.3 The deployment's acts

An application sometimes acts where no client is there to ask: at the
end of what a person's conversations were about, at a start that a
restart cut short. A connector then asks the core for one of a few
acts in the deployment's name, as the deployment requests the tasks its
configuration or a connector starts (tasks.md, section 3). An act has
no party, no sign-in and no role: the configuration's grant and the
act's own subject are its whole check.

- **Who may:** a connector the configuration grants that kind of act
  (`../protocol/configuration.md`, section 3), while the configuration
  names a first owner (section 3): the deployment's acts are its owners'
  decisions, made for them by a connector they configured. An act of a
  kind the configuration does not grant its connector is refused *not
  offered*; any act, while the configuration names no first owner, is
  refused as lacking the role, none held. Neither refusal is saved, so
  a later start's configuration decides again.
- **What it may,** each act on its own subject alone:

  | Act | Its subject | Becomes |
  |---|---|---|
  | `Close` | a live conversation of the named project (tasks.md, section 3) whose requester is a person; a grace within the tasks child's bound, or none; a reason within a cancel's | in one decision, the deployment's compound (tasks.md, section 7), section 7's close with every delegate cancelled: each live task the conversation requested cancelled, then the conversation cancelled with the grace, each with the reason; answered as a compound request (5.1) |
  | `AddKey` | an LLM provider whose sources offer a key; the key, carried as a request carries one (accounts.md, section 5); one of the deployment's accounts to join, or a new one | the key added, the deployment's (accounts.md, 3.2 and section 5) |
  | `RemoveAccount` | an account the deployment holds | the account removed (accounts.md, section 5) |
  | `Adopt` | one of the acting connector's own resources, as an owner's adoption names one, into a live project | adopted as an owner's adoption is, the connector asked and answering as it does a party's (connectors.md, section 12), the project's roles seeded from the parties the connector knows there, and no party made its owner |
  | `MakeProject` | as `MakeProject`'s (5.1), and an owner of the deployment, to be its maker | the project made (5.2), that owner its maker and only party |

  A subject outside its act's is refused, naming it: a task that is not
  a live conversation of a person, an account the deployment does not
  hold, another connector's resource, a maker who is no owner of the
  deployment.
- **Keyed as a party's request** (5.1): the connector names a key, in
  its own key space, apart from every party's and every other
  connector's, and its answer is kept beside parties' answers
  (`../protocol/store.md`, 4.4), with the same retention, room, joins
  and conflicts. The same act under its key joins it while it is
  pending, and once decided is answered from its saved answer, made
  once; another act under the key is a key conflict. A saved answer is
  given again whatever the configuration grants since, as a party's is
  whatever their role is now, and a restart restores it whatever the
  configuration names, until the retention ends it. Busy and not ready
  are not saved, and the connector sends the act again.
- **Owned memory:** each retained act charges the larger of the close's
  reason (`words`), the adoption's whole owned payload (`amendment_bytes`,
  including its slice headers), and the project name (`NAME_BYTES`), plus
  the larger of the configured host-failure list and a refusal's 64 task
  numbers. No act carries party model names or a sign-in address. Grants
  charge the configured entries, one per connector, each with at most one
  entry per act kind (`size_of::<ActKind>()` times the number of kinds).
  The people constructor checks these configured bounds.
- **Answered to its connector once durable** (engine.md, 5.2), through
  the root, with the outcome a party's request of its kind gets; each
  send, a joined one among them, is answered once.
- **Across a restart:** a connector keeps in its own records each act it
  has still to hear answered, under its key, committed with the
  decision that first asks for it, and sends each again at its start,
  once the core is ready, until it hears the answer. An act made before
  the restart is answered from its saved answer; an act the restart lost
  is made then. So a restart between an act's keep and its answer makes
  it once and answers it once. Past the retention it is judged afresh:
  a conversation closed since is refused as ended, an account removed
  since as unknown, which its connector takes as done.
- **The deployment, not a party:** an act makes no party, sign-in,
  inbox or role, and is in no `me`. A task it cancels is cancelled by
  the deployment, with its reason. The value an `AddKey` carries is
  kept by the application first, as a client's request's is
  (`../protocol/client.md`, section 5), and one the act leaves unused
  is said unused before its answer, for the application to erase. A
  project it makes has a party as its owner, the deployment's owner the
  act names.

## 6. A party's inbox

What waits for a party, across every project they hold a role in, each
entry naming the task or proposal it is and its project:

- **questions** from tasks they asked for, conversations included;
- **proposals** routed to them, or to a role they hold;
- **escalations** that reached them: a held task nobody below could
  decide;
- **person tasks** addressed to them, or to a role they hold (section 8);
- **results** of the tasks they asked for, and the turns of their
  conversations they have not read.

- **Derived from the tasks,** not kept apart: an entry is there while its
  proposal is pending, its person task is active, its question is
  unanswered, or its result unread. The only state of the inbox's own is
  each party's read positions, one over results and one per conversation
  over its turns, moved by their `Seen` (5.1), never by a read. Each mark
  durably advances its position to `max(old, through)`. Processing checks
  the current subject and authority each time: a retry after completion
  may answer `Unknown` after history was forgotten, or `Role` after
  authority was lost, even after earlier success. It never recreates
  forgotten history or promises the earlier processing's answer. Each
  admitted caller and joined waiter gets one terminal reply, held behind
  the same commit barrier. Restart restores committed positions or their
  committed absence; new completed marks leave no retained answers.
  A question waits for the person until it is answered, not until its
  task has read it. The asked task keeps each question it heard and its
  first answer (`../protocol/store.md`, 15.7). The person's inbox, a question's
  live document and an answer's admission read those lines, so a task's run
  reading a question leaves it open for its person. The task answering
  it itself closes it as the person's answer does.
- **A conversation** is a task its creator flagged so (tasks.md,
  section 3), whatever its charter, tracked or not (tasks.md, section
  7): its turns are entries for its requester, while that
  requester is a party, with their read position and an unread count.
  One position per conversation, its requester's: a move to another
  party erases the old requester's in the same decision, and starts the
  new one's with nothing read. No other task has a read position over its
  turns, so their number stays within the tasks flagged conversation,
  and those in memory within the live ones, one each.
  - *While the task is live,* its position is this child's, in a family
    keyed by party and task, loaded at start.
  - *When it ends,* the decision that ends it moves the position to this
    child's family of ended read positions, keyed by task and party and
    loaded on demand; the child holds none of them in memory, a read of
    the conversation or a task query's row joins it by a point read, and
    the inbox's census reads the family through at start
    (`../protocol/store.md`, 4.4, 7.2 and 7.4).
  - *A mark on an ended conversation,* a `Seen` over its turns, loads
    its record and its position, and is decided on them: the position
    moves to the larger of the two, never back. One such mark loads at a
    time: while it does, the party's next `Seen` over that
    conversation's turns, or over results that would forget it, is
    answered busy (engine.md, 5.3). Its loads are held behind the
    commits before them, so a mark sent after another was decided sees
    that one's position, and no mark saves a position over a newer one,
    or saves one for a task already forgotten.
  - *It is erased with the task:* by retention when the task is kept, or
    at once when it is forgotten once read.
- **A task's retention class** says what becomes of it once it has ended
  (tasks.md, section 3):
  - *kept:* its records stay to the deployment's retention horizon, and
    whole while none is set (`../protocol/store.md`, 11.1). jig's chat
    template keeps, so closed conversations stay readable;
  - *forgotten once read:* once it has ended, its requester has read its
    result and, for a conversation, its turns, and no live task names it,
    its record and its read position are erased in one decision, so no
    restart sees one without the other, and nothing read comes back
    unread. The `Seen` after which it is read forgets it, while the
    party's inbox holds it (the bound below), in the `Seen`'s own
    decision: the mark and the erase commit together, so no restart sees
    the task read and not forgotten. A `Seen` over results loads nothing
    for it: the inbox's entry for an unread result carries what its
    forget needs (the task's class, its label and listing, and, for a
    conversation, its last turn and whether its turns are read). A
    `Seen` over an ended conversation's turns loads its record and its
    position before it is decided, and forgets in that decision.
    Retention's next pass forgets the others: one the inbox no longer
    held when it was read, or one a live task named then. Its other
    remains go at retention's next pass (`../protocol/store.md`, 7.4 and
    11.1). A task whose requester is not a party is kept, whatever its
    class: no party reads it.
- **Unread counts.** A conversation's unread turns are counted from its
  requester's position and carried on its task line; `me` counts what
  waits for the party, and their unread updates
  (`../client/domain/contract.md`, 5.1).
- **Across projects,** since a party who works in several decides in one
  place; a client filters it by project, kind or root
  (`../client/domain/contract.md`, section 6, and 7.1). It is not a task
  query: proposals, escalations and questions are entries, not tasks.
- **What is addressed to a role** is in the inbox of every party holding
  it, until one of them acts, when it leaves every one.
- **Bounded:** the people child domain holds each party's entries up to a
  limit; a client pages through the rest, loaded from the store. What
  needs the party is counted exactly, since it is derived from live
  tasks; updates unread are counted up to a limit.

## 7. Chats

- **A chat is a template,** jig's default for a conversation with an
  agent, sugar over the primitives (core.md, 7.3): template 1, label 1,
  in the application's configuration, which an application keeps,
  changes or drops. Filled in, it is a `Create` of one agent task, its
  spec the party's first words, flagged conversation and kept, with the
  fields `../protocol/configuration.md`, 3.1 gives template 1. Its
  executor is an agent under the application's charter for chats, which
  a project's policy may override: its instructions for chats, its
  models, the tools it gives, and a wake policy that wakes it on its
  requester's words and on its delegates' results.
- **The engine never expands or checks it.** The client fills the
  template, as `me` serves it merged with the project's policy's
  overrides (authority.md, section 6; `../client/domain/contract.md`,
  3.6), into an ordinary `Create`, which the engine checks as one
  written by hand (5.1; `../protocol/configuration.md`, 3.1). A client
  that ignores it makes only its own lists inconsistent.
- **Permissions stay primitive.** A party may hold a chat when their role
  may create with the chat charter, within its spend and depth, and their
  escalations have a holder (5.1); no permission names a template
  (authority.md, section 6).
- **Turn by turn, live.** The party's words, each a `Message`, wake it,
  or reach its live run at once; its replies stream to the client as the
  run reports them, and each turn is committed as it ends (engine.md,
  7.2), so the conversation the client shows is the transcript in the
  store.
- **Waiting, then parked.** A chat waits for the next words holding its
  host's slot; idle past its charter's threshold it parks, and the next
  words resume it from its transcript, on any host its charter allows.
- **What a chat does:** answers; delegates; hands work to a procedure; or
  proposes a goal, which its requester accepts in the conversation, as a
  proposal in their inbox, and which is theirs once accepted, not the
  chat's (tasks.md, section 9).
- **What it waits on** is the engine's, computed with the tree and the
  inbox (tasks.md, section 3); **its status** (its phase, its run and
  what it waits on, read together) is a fact jig's helpers derive from
  its documents (`../client/domain/client.md`, section 19), never the
  engine's state.
- **A party's chats** are a task query by requester and label 1, live
  and ended (`../client/domain/contract.md`, 6.1), each line carrying its
  unread count (section 6).
- **Closing, through a helper.** jig's helper for chats, in `jig-client`
  (`../client/domain/client.md`, section 19), composes one compound
  request (5.1): each live delegate cancelled or moved to the party, as
  they say, then the chat cancelled with the template's grace, which
  asks it to finish, as an amendment it is woken by, and cancels it if
  it has not ended when the grace ends (tasks.md, section 7). Any client
  may compose the same request itself; the engine knows no close.
- **Closed by the deployment,** where no client is there: a connector
  granted it makes the deployment's `Close` (5.3), the same close with
  every delegate cancelled, none moved, since no party says which to
  keep. The core composes it from the conversation's live delegates in
  the decision that admits it, and the application gives the grace,
  its template's: the engine still reads no template.

## 8. Person tasks

A task whose executor is a party, or a role (core.md, 3.2):

- **Made by** an agent's `delegate` (a choice between two reports), a
  procedure (an approval it waits for), or a party's `Create`.
- **Its contract** is a choice among the options its spec lists, an
  approval of a state its procedure names, or an answer in words; and,
  always, that it cannot be done, with a reason, which is a failure
  (tasks.md, section 3).
- **An approval** is of the state as its procedure named it: bytes the
  core carries with the contract and never reads, and a part its
  connector shows the state by. The answer names the state it approves;
  declining is "cannot do it". It is admitted as any answer is, naming
  the person task's revision, and is refused as moved unless its state
  equals the contract's, byte for byte: the core compares the two, and
  never reads what they say. The result reaches the procedure that
  asked, as any delegate's does, and its connector reads the state.
- **Shown** with its spec, the results it depends on, and what its
  contract asks; the party's `Answer` is its result.
- **Addressed to a role,** it waits for any party holding it; one who
  takes it (`Take`) holds it, so two do not answer at once, and may hand
  it back (`HandBack`). A service may hold a role a person task is
  addressed to, and answer it through its requests.
- **Stalled,** past its bound, it is news for its requester, which may
  amend, cancel or address it to someone else.
- **Kept** by a dependency an amendment added (tasks.md, 5.7), it stays
  in its party's inbox, shown waiting on its dependencies, and an
  `Answer` is refused, naming them; once they are done, it is answered
  as ever. A take stands while it is kept. A task that depends on a
  person task goes on once the person task is answered.

## 9. Notifications

- **In a client,** live: a signed-in party's inbox changes reach their
  open watches through the views, once their commits are durable.
- **Beyond the client:** a page to the person on call, a mail, a chat
  system's message, each as a connector of the application's whose
  effects are messages to parties, keyed and made once like any other
  (connectors.md). The core says what waits for whom; the application's
  policy says which entries notify, and through which connector.

## 10. Below the domain

What the clients' component, on the engine's side, and the client below
its surface owe this domain:

- **Requests, reads and watches,** as jig's messages and the
  application's, carried to the component by the application's
  transport, which the component never sees
  (`../client/domain/contract.md`, section 10; `../protocol/client.md`).
- **Credentials:** minted on a printed link's secret, a service's
  credential, or the application's server's `Mint`; each presented one
  verified to its sign-in; a sign-in's digest erased when the core says
  it ended or expired. The domain names a sign-in by its number
  (`../protocol/client.md`, section 8).
- **Sign-in expiry timers** account for both clocks as section 3 says.
  A root omitting an otherwise empty closure still checks its current
  deadlines; refused admission leaves an expiry due (root.md, section 3).
- **Turns** carried as the bytes the domain keeps, decoded below the
  client's surface by `jig-protocol-transcript`
  (`../client/domain/client.md`, section 10).
- **Keys** for requests, made by the client, carried with each request
  (`../client/domain/client.md`, section 6).

## 11. The world

The people child domain has its world, with its parent scripted. In the
core's world (engine.md, section 15), scripted parties sign in and act
through operations: a member creates a chat from jig's template, which
proposes a goal they cannot fund, routed to a maintainer who accepts it;
a member's `Create` beyond their authority is refused, naming what it
lacked and that a proposal could be accepted, and then proposed and
accepted; a party's root is refused where their role decides no
escalation and the policy names no escalation role; a person answers a
choice between two reports; two maintainers decide one proposal, and the
second is told; a person stops a run, then writes to its task and
releases it in one compound request; a conversation with two live
delegates is closed, one cancelled and one moved to its person, and
cancelled with a grace it finishes within, and another outlives its
grace; a compound request is refused at its second operation, and
nothing of it is made; a conversation ends, and its read position is
read back from the store after a restart; a task forgotten once read is
erased with its read position; an observer is refused a request; a
service creates a goal its role allows; an ordinary deciding request
sent twice is made once across a restart; roles are seeded from an
adopted resource's parties; an application's server mints a credential
for an identity it verified, and a mint naming a provider not of kind
application, the
deployment's among them, is refused; a person task given a dependency by
its requester, its answer refused naming it, then answered once the
dependency is done; a member's amendment adding a dependency beyond
their reach, refused; a person signing in again by a fresh link while
holding a live sign-in of their own keeps it, its number and expiry,
with nothing written, and signs in so more times than the deployment
has room for sign-ins; and one holding another person's sign-in, or
their own ended or expired between the clients' check and people's
answer, signs in afresh; a sign-in past the deployment's room is
answered full, with its room and its earliest saved expiry, nothing
written, while one holding their own live sign-in keeps it, and signs in
once a sign-in ends; a sign-in finding the room full while one sign-in
is past its wall expiry, its timer not yet run, ends that one and signs
in; a new identity past the people table's room is answered busy; a
connector granted the deployment's acts closes a person's conversation
with two live delegates, both cancelled and the conversation cancelled
with a grace it finishes within, removes one of the deployment's
accounts, adds a key as the deployment's, and adopts one of its own
resources; an act sent again under its key, after a restart between
its commit and its answer, is answered from its saved answer with
nothing made twice, and one the restart lost is made then; acts are
refused, nothing made, while the configuration names no first owner,
of a kind their connector is not granted, and on a subject outside
their act's: a person's account, a task that is no conversation,
another connector's resource.

Its referee: every request is answered once, after its commit; a
compound request is made whole or not at all; no request is acted on
beyond the party's role, and none becomes a proposal unasked; no party
becomes a root's requester while its escalations have no holder; no
act in the deployment's name is made but by a connector granted its
kind, while a first owner is configured, on its act's subject; what
waits for a role leaves every inbox once one party acts on it; a
person's words reach their task, or the task ends; nothing read comes
back unread, across a restart or a task's end.

## 12. Open questions

- **Acting as a person:** whether an approval should also be posted on a
  system as the person who gave it, so that the system's own protection
  requires it independently. jig holds no provider's credential; an
  application's sign-in may keep one, and its connector would then need
  to know which party gave the approval.
- **Who may see what** among a project's parties: whether a chat and its
  transcript are its requester's alone, or their role's, or the
  project's; what an observer reads. A role in the project lets a party
  watch and read all of it.
- **Roles that span projects,** as the inbox does.
- **Guests:** someone with no identity at any provider, invited to one
  conversation. An application's sign-in may mint an identity for an
  invitation; which role lets its holder take part in one conversation,
  and see nothing else, is open.
