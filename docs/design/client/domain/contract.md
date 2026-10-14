# The client and the engine

Provisional, 2026-10-09, revised 2026-10-14. What crosses between a
party's client and the engine, at the level of the two domains: who the
party is, the keyed requests and their answers, the documents the engine
gives, the reads and watches that bring them, and versions. It is to
clients what `../../domain/hosts.md`, section 2 is to hosts: the
contract both sides keep, whatever carries it. Section 9 lists what the
engine's domain owes the client, as changes to its documents. What is
still open is listed in section 11; what waits, in 7.4.

## 1. In one page

- **Keyed requests, answered once durable.** Everything a party does that
  changes something is a request, keyed by its client, checked against
  the party's role, committed, and answered only once that commit is
  durable; sent twice under its key, it is made once and answered alike
  (`../../domain/people.md`, 5.1).
- **Decisions name what they saw.** A request deciding about something
  that changes names the revision of that thing as the client showed it:
  a hold's, a task's spec's, a person task's, a note's (5.2). The engine
  refuses one whose revision has moved, saying what it is now.
- **Refusals carry facts:** what was lacking and by how much, which limit,
  what the object is now, how it ended.
- **Documents per owner.** The engine gives the client typed documents
  about one subject each: jig's carry jig's values, and name an
  application's values as parts, which the application's own documents
  carry (section 5).
- **Documents say what the party may do.** Each object's document carries
  the acts the engine would admit from this party now, so a client never
  computes authority; the engine checks again when the act comes. An act
  whose answer depends on what the party writes is offered with bounds:
  the facts a draft is compared with, and what happens beyond them (3.2).
- **Reads decide nothing, and show only what is durable.** A read is
  answered from the engine's live state or a page of its store, needs no
  key, and its answer waits until every commit it reflects is durable, so
  a party never sees state a crash would lose.
- **Watches are level-triggered.** A watch delivers its subject's current
  document whenever it has changed since the last delivery, one delivery
  in flight. Nothing is queued, so nothing is missed: a slow client sees
  fewer deliveries. A run's live text is the one stream, read from the
  offset the client has (section 7).
- **An application's requests and documents** cross on the same terms and
  the same connection, kept by the people child by key like jig's, each
  naming its owner by its number (root.md, section 15), and routed by the
  engine's root to that owner: the connector they concern.

## 2. Parties and signing in

- **Who:** a party is known by a provider of the application's and a
  subject there; signing in is the protocol layer's, and the domain sees
  an authenticated identity and the kind of party it is
  (`../../domain/people.md`, section 3).
- **A sign-in is numbered,** and every request, read and watch carries it
  below the domain; the engine refuses one whose sign-in has ended, and
  the client goes to sign in (client.md, section 12).
- **The frame's document** (section 5) says who the party is, so a client
  never decides it from what it kept.

## 3. Requests

### 3.1 The keyed request

- **The key** is sixteen bytes the client draws, scoped to the party:
  the same key from two of their sign-ins is one request
  (`../../domain/people.md`, 5.1).
- **Answered once durable:** the answer waits for the commit of the
  decision it reports, as every output that someone outside could act on
  does (`../../domain/engine.md`, 5.2).
- **Kept for a retention:** a durable answer is saved under its key, and a
  request sent again within the retention gets it again. The frame says
  how long the retention is, so a client never sends a request again
  past it (root.md, section 4). Busy and not ready are never kept: the
  client sends the request again, under its key.
- **One key, one ask:** a key sent again with a different ask is refused
  as a conflict.

### 3.2 jig's requests

What a party asks, what it names, and what it becomes. The requests of
`../../domain/people.md`, 5.1, by its names, with what this design adds
marked new.

| Request | Names | Becomes |
|---|---|---|
| start a chat | a project; the first words | a chat, its number answered |
| write to a task | a task; the words; whether to release it too, when the party stopped it (new) | words in its inbox, their message number answered; released, if asked |
| answer | a question: its task and message, and the words; or a person task: its revision (new), and the answer by its contract (for an approval, the state it approves, new) or that it cannot be done | an answer to the question; the person task's result |
| take or hand back a person task | a task addressed to a role the party holds | held by them, or back to the role |
| decide a proposal | its proposer and number; accept, reject with a reason, or pass up | as `../../domain/tasks.md`, section 9 has it; or decided first, by whom and how |
| decide an escalation | a held task; its hold's revision; release, leave held with a reason, or pass up | likewise |
| release | a held task; its hold's revision (new); for an uncertain effect that cannot be recovered, whether to make it again (new) | its hold lifted, judged afresh |
| amend | a task; its spec's revision (new); the change; whether to release it too (new) | amended, or proposed where it widens beyond the party; and released, if asked and still held |
| stop a run | a task | its run cancelled, the task held for the party |
| cancel | a task; a reason | it and its subtree closing |
| close a chat (new) | a chat; for each live delegate, cancelled or made the party's | the delegates disposed of, the chat asked to finish, cancelled after a grace |
| make themselves a task's requester | a task; a reason | the party its requester, its budget moved to their pool |
| set a goal | a project; its spec, charter, budget, priority; a deadline (new) | a goal; or a proposal, beyond the party's allotment |
| prioritise goals | a project; goals and their priorities | changed together |
| mark seen (new) | how far the party has read: results, or one chat's replies | their read position over that moved |
| edit or delete a note | a note; the revision recalled | changed, or refused as moved |
| adopt a resource | the application's own request, its form a part in a slot (`root.md`, section 6); the engine's root makes the people request on the party's behalf | the project changed |
| change the policy, roles or services; set a party's pool | as `../../domain/people.md`, 5.2 | the project changed |

- **Proposed, not refused.** Setting a goal beyond the party's
  allotment and an amendment that widens beyond their authority become
  proposals in the same request, waiting for the roles the project's
  policy names for them (`../../domain/tasks.md`, section 9); the task's
  or the project's document says so among the acts it offers, so the
  client says it before the request is sent. Where the policy names
  nobody who could accept the proposal, the request is refused as
  lacking authority, saying that no proposal could be accepted
  (section 4): it is never made a proposal that waits for nobody. This
  decides a party's request at admission only: a proposal once made, or
  one a task makes, waits as `../../domain/tasks.md`, section 9 says.
  Any other request beyond the party's authority is refused, saying
  what was lacking.
- **Offered with bounds.** Whether setting a goal or an amendment is
  made, proposed or refused depends on what the party writes: a budget,
  a deadline, a grant. Its document cannot say which for every draft, so
  it offers the act bounded, with the engine's facts a draft is compared
  with, in two bounds, and what happens beyond the first:
  - *a goal's:* the charters offered for goals, and the party's pool
    left; within, the largest budget and the latest deadline the party
    may set alone, from their role and their pool; and, past that, the
    largest budget and the latest deadline any goal may have, from the
    project's spend per period, its policy's ceiling and the
    deployment's rules;
  - *an amendment's:* within, the whole authority the party may give
    alone (their role's, its budget no more than their pool has left,
    its tasks no more than the task has left), the budget they may add
    to the task, and the grants they hold, listed; and, past that, the
    policy's ceiling, its budget no more than their pool has left;
  - *beyond the first bound:* proposed, to the roles the project's
    policy names for that kind of proposal, in their order; or, where
    it names none, refused as lacking authority, saying that no proposal
    could be accepted.

  The client compares the draft with those facts before the request is
  sent, and says which it will be. A goal is made when its budget is no
  larger and its deadline no later than the first bound's, a goal with
  no deadline being beyond any bound that has one; past the second
  bound, or with a charter not listed, it is refused as not offered, and
  the client sends nothing. An amendment that widens no authority is
  made. One that widens anything, even while it narrows something else,
  is judged whole: the authority it leaves the task must fit the first
  bound, the budget it adds be no more than the party may add, and each
  grant it adds be among those listed; past the ceiling it is refused as
  not offered, and the client sends nothing. Between the two bounds, it
  is what the offer says beyond the first, and a refusal there is said
  in place too, sending nothing. That is the engine's word read, not
  authority computed: the engine judges again when the request comes, by
  the rule that made the offer. No read assesses a draft.
- **An approval names its state.** A person task's contract may be an
  approval of a state its procedure names, which its connector shows by
  a part (objects.md, section 7). The answer names the state it
  approves; declining it is "cannot do it", with a reason. It is
  admitted naming the person task's revision, as every answer is, and
  refused as moved unless the state it names equals the contract's,
  byte for byte: the engine compares the state and never reads it. The
  result reaches the procedure that asked, whose connector reads it.
- **A watch is not a keyed request.** It changes nothing and needs no
  answer kept: it is admitted on the party's sign-in and role, and ends
  when its client leaves (section 7).

### 3.3 An application's requests

A request about an application's own object (silence this alert, extend
this environment) is the application's (`../../domain/people.md`, 5.1):

- **keyed alike,** in the same key space, by the same table in the client
  (root.md, section 4);
- **admitted by the people child** by key, as jig's are: duplicates
  joined, the party's sign-in and their role in the request's project
  checked; a key sent again with another request is told apart by a
  digest of the request's encoding, which the protocol layer gives the
  domain with it;
- **refused at admission with jig's refusal:** a request the people
  child refuses there (its party's role in the request's project, a key
  sent again with another request, a limit), or one whose sign-in has
  ended, is answered as jig's are (section 4) and never reaches its
  connector. The client root routes that answer, by the request's owner,
  to the family that asked, which shows it as a refusal, never as its
  object gone;
- **routed by the engine's root,** by the owner it names, to the
  connector it concerns, which decides it within the same decision.
  What it asks of the core on the party's behalf (an amendment, a task,
  a release) is judged as the party's own request would be, against
  their role and authority; what it changes of its own is the
  connector's to allow, by its policy's roles;
- **answered once durable,** once admitted, with an outcome of the
  connector's own: only an admitted request is the connector's to
  decide. The people child keeps the key and a token; the connector
  keeps the outcome under that token, durably, for the retention, and
  gives it again when the request is sent again, as the core asks a
  connector for any value it does not own (`../../domain/core.md`,
  section 5).

## 4. Answers

Every request has one answer, of one of these kinds:

- **done,** with its outcome: what was made (a chat's number, a word's
  message number, a proposal's number), or what was decided; and, when
  another party decided first, who, how and when;
- **refused,** with its facts:
  - *role:* the party's role here does not allow it;
  - *authority:* what was lacking, typed (a budget and how much is left,
    a grant, a kind of task, a depth, a deadline, tasks left), and
    whether a proposal could be accepted: a goal or an amendment beyond
    the party, when the project's policy names nobody who could accept
    its proposal, is refused so, saying none could (3.2);
  - *limit:* which one (the project's live tasks, the inbox full, the
    words too long);
  - *moved:* the object's revision now, and what changed;
  - *ended:* the object has ended, and how;
  - *unknown:* nothing by that name the party may see;
  - *not offered:* the deployment does not offer it (no chat's charter in
    this project);
  - *key conflict:* the key names another ask, which is a client's bug;
- **busy** or **not ready:** nothing was made and nothing kept; send it
  again, under its key, after a backoff;
- **signed out:** the sign-in has ended; sign in, then send it again.

A client hears one more, which no engine sends: **no answer**, when the
connection failed before one came; whether the engine has the request is
unknown, and the key makes sending it again safe.

The client's protocol layer gives each answer to the request table as an
envelope it can act on without reading what is inside (root.md,
section 4): *final* for done and refused, with the outcome decoded into
its owner's vocabulary; *again* for busy and not ready; *lost* for no
answer; *signed out*.

## 5. Documents

### 5.1 What they are

- **A document** is a typed value about one subject, from the engine's
  committed state, as the party may see it. Its owner is jig, or the
  application: jig's documents carry jig's values only.
- **jig's documents:**
  - *the frame:* the party, their kind, the projects they hold roles in
    and their roles; what needs them, counted exactly, and updates
    unread, counted up to a limit; the retention of its answers (3.1); the charters it offers for chats
    and goals in each project;
  - *a task:* its header, overview, budget and authority, its hold and
    its escalation if held, the proposals waiting in its subtree, how far
    its activity has committed, and the acts the party may make on it;
  - *a chip:* a task's number, title, executor's kind, phase; a closing
    phase says what the task waits for: its delegates still closing,
    counted, and the connectors it waits on to settle, by number. Their
    effects are the connectors' own, and jig's documents never count
    them (objects.md, section 3);
  - *a tree:* a task's live subtree, as chips, with the proposals waiting
    on each, folded beyond the watch's window into counts by phase, with
    counts of what is held below;
  - *an inbox:* the party's entries, each naming its object and its
    project, in its groups, within a window;
  - *a proposal, an escalation, a question, a person task, a result:* each
    object as objects.md says it shows, with the acts the party may make
    on it;
  - *an activity line:* one of a task's words, run boundaries, turns,
    results heard, news heard, decisions, in commit order (client.md,
    section 10); a turn as its bytes, within the page's byte bound, which
    the client's protocol layer decodes, with no read of its own, into
    the client's turn or into a turn that does not decode, with why
    (client.md, section 10): a turn that alone passes the page's bound
    comes with its size instead, and does not decode, as too large; one
    the engine could not read whole, as malformed;
  - *a conversation's line:* one of a party's chats, for their list;
  - *a goal's line,* a project's, its policy, roles and spend, a note;
    a project's document carries its recurring tasks' lines, each saying
    when it next runs: when the next period begins, where the period's
    source said so; otherwise at the next period, with the current
    period and when it began (`../../domain/authority.md`, section 7);
  - *the deployment:* hosts, accounts, capacity.
### 5.2 Revisions

- **One for each thing a party decides about,** which moves only when
  that thing changes, never with spend, a delegate's phase or a run's
  progress:
  - *a hold's,* on a held task: what holds it, and why. It moves when
    the task is held anew or its reason changes, never when its
    escalation passes up. A release and a decision about its escalation
    name it;
  - *a spec's,* on a task: its words, parameters, wake policy,
    dependencies, authority and instructions; an amendment names it;
  - *a person task's:* its question, options and the results it shows;
    an answer names it;
  - *a note's,* as the notes child keeps it; a correction or a deletion
    names it.

  A proposal never changes once made, so deciding it names its number;
  where it waits may move, and a party it no longer waits for is refused
  as no longer the one it waits for. Stopping and cancelling name no
  revision.
- **Where an escalation waits is checked apart from its hold.** Deciding
  an escalation (releasing the task from it, leaving it held, passing it
  up) needs the escalation to wait for the party, and is refused as not
  waiting for them otherwise. Releasing, amending or cancelling the held
  task needs the task in the party's tree, wherever its escalation waits
  (`../../domain/people.md`, 5.1): a release that names the hold's
  revision from before a pass up is made, not refused as moved.

### 5.3 What else they carry

- **Acts offered.** Each object's document says which acts the party may
  make on it now, and for those it may not, why (role, authority), as
  the engine judges it. Each offered act says what it will do: made;
  proposed, and to whom; or, for setting a goal and amending, bounded,
  with the facts a draft is compared with and what happens beyond them
  (3.2). The client offers what the document says, and the engine judges
  again when the act comes.
- **Parts.** Where a jig document names an application's value, it
  carries the part's reference (root.md, section 6): its kind, its owner,
  and jig's key (a resource's path, a kind's number, a task's or a
  proposal's number). Its owner is the connector whose value it is:
  every part jig's documents name is a connector's, so the number a part
  carries (`connector`, in `jig-domain-client`'s `Part`) is its owner
  number (root.md, section 15). The value is in the application's own
  documents.
- **Times are wall times,** which the store keeps, so a document means the
  same after a restart.
- **Bounded:** each document within its limits, in bytes and rows.
- **Texts are UTF-8,** words for showing. A source's text is UTF-8 where
  it enters the engine: a party's words, refused at their request's
  entrance when they are not; a provider's login and name, from its
  identity; a run's output; an application's words through its own
  documents. A text cut to its bound is cut at a character's boundary,
  never inside a character, and says it was cut. So every text a
  document carries is UTF-8, as the wire's text fields check
  (`../../protocol/client.md`, section 3), and the engine never makes a
  document its wire refuses.

## 6. Reads

A read asks for one document, or one page of a list, and is answered
once, with the document or why there is none. It decides nothing and
needs no key. Its answer is made from the engine's state as it is, which
may run ahead of the store by the commits in flight, so it is held as an
output of a decision that made no commit: tagged with the last commit
made, and released once that is durable (`../../README.md`, 6.2). What a
party reads is therefore never lost by a crash.

| Read | From |
|---|---|
| the frame | live state |
| a task; a proposal; an escalation; a question; a person task | live state, or the store once ended or decided |
| a result | the store, or live state while a stub names it |
| a task's ended delegates, a page | the store |
| a task's history, a page, for it or its subtree | the store |
| a task's activity, a page, from its end | the store: words, run boundaries, turns, what it heard |
| an inbox's group, a page, filtered | live state, then the store past the engine's limit |
| a party's conversations, a page | live state, then the store |
| a project's goals, live then ended, a page | live state, then the store |
| a project; its notes, a page | live state; the notes child |
| the deployment | live state |
| an application's read, naming its owner and the scope (7.2) | its owner, the connector, through the engine's root |

- **Pages** are within a row and a byte limit, and carry a cursor for the
  next, or say there is none (`../../domain/engine.md`, 5.3).
- **Refused** for what the party may not see, as unknown; when the
  engine's loads are full, as busy; when the sign-in has ended, as
  signed out (client.md, section 12).

## 7. Watches

### 7.1 Subjects

| Subject | Its document | Who may watch |
|---|---|---|
| the frame | the frame | the party, always |
| a task | the task's document | a party who may see it |
| a tree | a task's live subtree, within a window | likewise |
| chips | a bounded set of tasks a page names, as chips | likewise, each |
| the inbox | the party's own, across projects, within a window | the party |
| a project's goals | its live goals, by priority | a party with a role in the project |
| a project's tasks | its live tasks, filtered by phase, by executor, by goal, or by whether the party asked for them, ordered by their last change, within the watch's window, saying whether there are more | a party with a role in the project |
| a party's conversations | their live chats | the party |
| a run | a task's live run: its attempt, its provisional text and current call (7.3) | a party who may see the task |
| the deployment | hosts, accounts, capacity | a party with a deployment role |
| an application's subject, naming its owner and the scope | the application's document | a party with a role in that scope; what of it they may see, as its owner, the application's connector, says (7.2) |

The objects a card shows apart from these (a proposal, an escalation, a
question, a person task, a result, a project, a note) are read
(section 6), and read again when a watch that names them says they
moved (objects.md, section 2).

### 7.2 Level-triggered

- **A delivery is a whole document,** the subject's current state within
  the watch's window, computed when the delivery is made, not when the
  change happened, and held until every commit it reflects is durable,
  as a read's answer is (section 6).
- **One delivery in flight per watch.** A change while one is in flight
  marks the watch changed; when the client has taken that delivery, the
  next is computed from the state as it is then. Changes in between are
  coalesced, never queued.
- **Nothing is missed:** the next delivery has every change. A slow
  client sees fewer deliveries, a fast one more; neither holds the engine
  up, and what the engine keeps per watch is a mark and a window, never a
  backlog.
- **The first delivery** is the subject's document when the watch is
  admitted.
- **Ended,** each saying why: the client left; its subject ended and left
  the engine; the party may no longer see it; the sign-in ended; the
  engine is stopping.
- **Admitted** against the party's sign-in and what they may see, and the
  watches the engine allows per party and in all; past them, refused as
  busy.
- **An application's subject names its owner and its scope,** as an
  application's read does: its owner by its number, the scope a project
  or the deployment. The core cannot read a connector's
  subject, so it admits on what it holds: the party's sign-in, their role
  in that scope, and the views' limits, as for jig's subjects. The
  connector, asked for the document, judges that the subject is in that
  scope and what the party may see of it, and answers unknown or not
  allowed otherwise. The engine's root only routes.
- **What marks a watch changed:** each decision's commit names the tasks,
  projects and parties it touched. A task touched marks the watches of
  that task, of every tree along its lineage, of chips naming it, of its
  project's tasks, of its project's goals if it is one, and of its
  requester's conversations if it is a chat; a party's entries touched
  mark their inbox and their frame. A connector marks its own subjects,
  through the root, as its state changes.

### 7.3 A run's live text

The one stream: what a run writes before its turn commits is not state
the engine keeps (`../../domain/engine.md`, section 11), but a person
reads it as it comes. Its deliveries decide nothing and follow no
commit, so they leave through the journal's door (`../../README.md`,
6.3), marked provisional.

- **The run's document** is its attempt, the turn being written, named by
  its attempt, its number within it and its completion's retry as the
  host reports them, that retry's text so far, and the call it is making,
  within a bound on bytes.
- **Read from an offset:** each delivery carries the turn's name, the text
  after the offset the client has, and the offset it ends at; the client
  takes it and is sent the rest when there is more. A new turn's text,
  or a new retry's, starts at zero under its own name, so it never resets
  another turn's; a later retry replaces the earlier retries of its turn.
- **Committed turns are not in it.** A turn's committed form reaches the
  client only through its task's activity, read once the task's document
  says the activity has moved, and held until durable (section 6). The
  client keeps a turn's provisional text until then (client.md,
  section 10).
- **Past its bound,** the text is cut from its start, at a character's
  boundary, and the delivery says so; the turn, once committed, has it
  all.
- **The watch follows the task,** not one attempt: a new run's text
  follows the last, and its beginning is in the document.
- **Only from a host that gives it.** A run's text and the call it is
  making come from hosts that hand them up (`../../domain/hosts.md`,
  6.5): a run in an agent process gives neither, and its document holds
  no text and no call, while its turns reach the client as they commit.

### 7.4 Later: an application's streams

An application's data that its people follow as it comes (a log
followed, a metric's latest points) is a stream of the application's:
a subject naming its owner and its scope, read from an offset within a
window, as a run's text is (7.3). Its deliveries decide nothing and are
never a document of the engine's committed state: they leave through
the journal's door (`../../README.md`, 6.3), marked provisional, and
show marked so (view.md, section 3). It adds a kind of subject to the
application's own and a use of the journal's door, with no change to
7.1's subjects or 7.2's rules, and waits for an application whose
people follow its data as it comes. jig's browser parts carry such a
stream as any watch, with no change of their own
(`../../protocol/browser.md`, section 16).

## 8. Versions

- **Each owner's documents and requests are versioned,** jig's by jig and
  each application's by the application; the client's protocol layer
  encodes each with its version.
- **The engine serves a range of versions,** which the hello of its
  stream of deliveries says (`../../protocol/client.md`, section 9); the
  frame carries none. A
  browser's client comes from the engine and matches it; a phone's or a
  terminal's may not.
- **Outside the range,** the client shows that it is out of date and how
  to update (pages.md, section 8), keeps what is pending in its small
  store, and sends nothing more; updated, it sends what is pending, under
  its keys.
- **How long a range is, and how a change of jig's reaches every
  application,** is jig's versioning, still open (`../../README.md`,
  section 14). Until upgrades are designed, each range is one version
  (`../../protocol/README.md`, section 7).

## 9. What the engine's domain owes the client

Changes this design asks of `../../domain/`, adopted there with it
(`../../domain/README.md`, section 5):

1. **The requests of 3.2 marked new:** words that also release; a
   release, an amendment and a person task's answer naming the revision
   they decide; an approval naming the state it approves; amending and
   releasing as one; closing a chat with its delegates' disposals; an
   uncertain effect's release saying whether to make it again; a goal's
   deadline; marking seen, with a read position for results and one per
   chat for its replies
   (`../../domain/people.md`, 5.1 and section 6).
2. **An application's keyed requests,** admitted and kept by the people
   child as jig's are, told apart by a digest, judged as the party's when
   they ask the core for something, their outcomes kept by their
   connector for the retention (3.3; `../../domain/connectors.md` gains
   it).
3. **Refusals with facts,** as section 4 lists them.
4. **Revisions** of holds, specs and person tasks, in their documents and
   in the decisions that name them (5.2 lists them).
5. **Acts offered** in each object's document, saying where an act would
   be proposed rather than made, and offering setting a goal and amending
   with bounds (section 5; 3.2).
6. **The inbox across projects:** the party's entries in every project
   they hold a role in, needs counted exactly and updates up to a limit
   (`../../domain/people.md`, section 6).
7. **A task's activity kept in commit order,** including each party's
   words as written, so that a conversation reads whole after its words
   are taken (`../../domain/engine.md`, 7.2).
8. **The reads of section 6,** their answers held until the last commit
   made is durable (`../../domain/engine.md`, 5.2 and 5.3).
9. **Level-triggered views** with the subjects of 7.1, what marks them
   changed (7.2), a run's text by turn, retry and offset, and an
   application's subjects through the same watches, each naming its scope
   (`../../domain/engine.md`, section 11). The views child keeps, per
   watch, its subject, its window, whether a delivery is in flight and
   whether its subject changed since; the subject's owner (the core, or a
   connector through the root) computes each document when a delivery is
   made, and the delivery is held as a read's answer is. Older passages
   that describe views as a best-effort stream (`../../domain/core.md`,
   section 11; `../../README.md`, 6.3) say so too.
10. **A watch is not a keyed request** (3.2).

## 10. Below the domain

What the client's protocol layer and the engine's owe this contract,
which the client's protocol designs (`../../protocol/client.md`):

- **One connection per client,** carrying jig's documents and the
  application's, composed in each end's protocol layer: requests and
  reads as exchanges, watches' deliveries on one stream, each tagged with
  its watch.
- **HTTP for exchanges, server-sent events for deliveries,** in the
  browser and over skein's HTTP client alike, as the platform allows.
- **Each request carries its key and its sign-in;** the engine's protocol
  layer checks the sign-in's secret and hands the domain its number.
- **`jig-protocol-client-wire`** encodes jig's documents and requests for
  both ends, from one schema, with their versions; each application's own
  wire crate does the same for its own.
- **Turns** cross as their bytes, inside activity lines, and are decoded
  by `jig-protocol-transcript` in the client's protocol layer.
- **Sign-in exchanges** are each provider's, in the engine's protocol
  layer and the client's shell.
- **The browser's client is served by the engine,** from its one origin
  (`../../protocol/browser.md`, section 12).

## 11. Open questions

- **Bundling parts:** whether a jig read's answer should carry the
  application's parts it names, put together by the engine's root, to
  save a client the second round trip.
- **Windows of a large tree:** how a tree watch's window follows what a
  person opened, so that a tree of hundreds of tasks costs what is shown.
- **Who may see what** among a project's parties, deferred while the
  client is built for one person (`../../domain/people.md`,
  section 12).
- **Search** over tasks, results, notes and transcripts, which the engine
  has only over notes' descriptions.
