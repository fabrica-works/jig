# The store

Provisional, 2026-10-12, revised 2026-10-23. What lies beneath the
engine's store vocabulary: skein's key-value store, embedded in the
engine's process, holds every record jig's core and the application
keep, transcripts are files beside it, and secrets are in a second
store. jig ships the component that does this, `jig-protocol-store`; the
application hands it its own records as bytes it encodes. The domain's
half (commits, loads, what is kept) is `../domain/engine.md`, section 5,
and what the rest of the protocol layer relies on is `README.md`,
section 9. Read it with skein's `kv.md`. What is still open is listed in
section 16.

## 1. In one page

- **skein-kv, in the engine's process.** The engine's protocol layer owns
  `jig-protocol-store`, a component that owns two skein-kv stores (skein's
  `kv.md`): the store, with every record jig and the application keep,
  and the secrets' store. Keys are ordered bytes, values are bytes, both
  in memory, made durable by an append-only log and snapshots. There is
  no store process, no wire and no SQL: the engine is the store's only
  client.
- **One component, two owners of records.** jig's records (the core's
  and its children's) reach the component typed, in the core's
  vocabulary, and the component encodes them. The application's records
  (its connectors' and its own children's) reach it as key and value
  bytes the application encodes, under the application's tag. Both go
  into one commit (section 3).
- **Keys sort as the domain's keys do,** written with skein-kv's key
  writer, with a tag per owner, then per family. Every range the domain
  asks for maps to a short, fixed list of intervals (section 4).
- **Records from schemas,** with skein's generator: a codec per family,
  versioned, golden bytes and fuzzing. A format row names the versions
  and the deployment; a store of another version or deployment refuses
  to open, and nothing is translated (section 5; upgrades are open,
  section 16).
- **One decision, one commit, the same number.** Each of the root's
  commits is one skein-kv commit with the same number. Group commit folds
  bursts into one sync. A failed or refused commit stops the engine
  (section 6).
- **Loads are answered in the step that asks,** from memory, in bounded
  pages: the restart's live ranges, parties' reads with their joins, a
  child's pages, single records (section 7). Only a turn's bytes, read
  from its transcript, take io.
- **Ended tasks are found through indexes,** the tasks child's: by their
  requester party, and by their project for those flagged listed, each
  by label and newest ended first. Live tasks are the tasks child's
  memory, never the store's to list (7.3, 7.4).
- **Transcripts are files, one per task,** each turn a frame written and
  synced before the commit that accepts it. The map keeps where each
  task's current window starts, so a resumed run reads only its window
  (section 8).
- **Secrets in a second store,** in a directory only the engine's user
  may read, written by components alone: minted credentials' digests,
  and LLM accounts' records and keys, each kept before it is used
  (section 9).
- **Growth is bounded by retention and fill:** the domain forgets an
  ended task once read, or shrinks it past a horizon on its timers, by
  the task's retention class, and refuses new work above a high-water
  mark of the map (section 11).
- **The real store in every world:** the domain worlds run the component
  over skein-kv's in-memory mode, with an in-memory stand-in for
  transcripts' files, and the store's own world crashes it on skein's
  simulated disk (section 14).

## 2. Where it sits

```
the application's root          commits (jig's writes typed, its own wrapped), loads, restart steps
        │ the root's store vocabulary, through the protocol layer's routing
        │   (the application's records encoded by its own codecs on the way down, decoded on the way up)
jig-protocol-store              keys and records to bytes; ranges to intervals; parties' reads and joins;
        │                       transcripts' files; secrets
        │ skein-kv's vocabulary (skein's kv.md, section 2)      │ io's file requests (skein's io.md, section 5)
skein-kv ×2                     the store; the secrets' store
        │ io's file requests
skein-io                        files: the ring, or the simulator's disk
```

- **A component of the engine's protocol layer,** step code under the
  subset (programming-model.md, section 10), with its limits, worst case,
  deadlines and most outputs per entry point. It depends on skein-lib,
  skein-kv, io's file vocabulary, skein-codec and its generated codecs,
  and the core's vocabulary (its keys, records, ranges and reads). It
  depends on no application's crate, and links no smith crate: turns are
  bytes to it.
- **On disk,** in the deployment's directory (`configuration.md`,
  section 3), which nothing else writes:

```
<deployment>/
├── store/          skein-kv: snapshot, log segments
├── transcripts/    one file per task, in directories by the task's number
└── secrets/        the secrets' store: the directory 0o700, its files 0o600
```

- **Opened before the root starts** (section 10; `service.md`,
  section 3).

## 3. The boundary

What the component takes and answers, as the engine's protocol layer
routes it. The core's side is `../domain/engine.md`, sections 5 and 6;
the root wraps it, and the protocol layer's routing unwraps it, encoding
the application's part with the application's codecs.

| Down, to the component | Up, from it |
|---|---|
| **commit** n: jig's writes, typed (the core's saves and erases of its records and its children's, and a task's transcript dropped whole); the application's writes, each a key's bytes and a value's bytes, or an erase | **committed** n, with the store's fill (11.2); **failed** n |
| **restore** a restart step's range: jig's (the core's live families, 7.2), or the application's, as byte intervals under its tag | jig's records, typed, then done; the application's rows as bytes, then done |
| **load** a page of a child's range: jig's, or the application's intervals | the page's rows, typed or as bytes, and a cursor if more remain |
| **read** a party's read: its scope, position, direction, rows and bytes (`../domain/engine.md`, 5.3) | the page, each row with its joins, and its cursor; or busy, or gone |
| **get** a record by key: jig's or the application's | the record, or none |
| **transcript**: a task's current window, from the turn the core names as its start; a turn by its task, attempt and number; a page of a task's turns from either end | the turns, with their fields and bytes, and a cursor; or unread |
| **secrets**: keep, replace, erase, get and load, by owner and number; carry a value under a drawn number; adopt a carried value into its source's row (section 9) | kept, stale, erased, the value or none, a page, or the carried number and digest |

- **Turns are records of the core's** (`../domain/engine.md`, 5.4): the
  core saves each accepted turn, keyed by its task, attempt and number,
  with its spend, the last message it read, its time, whether it opens a
  window, and its bytes. The component keeps the bytes and those fields
  in the task's transcript file, and the transcript's places in the map
  (section 8). The core never erases a single turn: a task's transcript
  goes whole, by retention (11.1).
- **Reads see what is durable.** A load, a read, a get or a transcript
  sees every commit answered before it was asked, and nothing that is
  not durable (skein's `kv.md`, section 2). Each load of the core's is
  held until every commit made before it is answered
  (`../domain/engine.md`, 4.2 and 5.3), so it sees what the decisions
  before it wrote.
- **One terminal per request,** tagged with the asker's token
  (programming-model.md, 4.4).

## 4. Keys

### 4.1 The encoding

- **Bytes sort as the domain's keys do.** Every key is written with
  skein-kv's key writer (skein's `kv.md`, 2.1) so that byte order equals
  the order Rust derives for it: a variant's index as a tag, then its
  fields in order; numbers big-endian at fixed width, narrower ones
  widened; an absent option before a present one; a byte string escaped
  and terminated, so a shorter one sorts before its extensions; a list of
  byte strings as each string marked, then an end mark.
- **Keys never hold payload:** numbers, small fixed identities, and
  names within their limits.
- **Written out, not generated.** skein's generator writes values, whose
  length-prefixed bytes do not sort (skein's `codec.md`, section 3), so
  each family's key has a small encoder and decoder of its own, beside
  its codec. A property test holds each encoder to the derived order on
  generated keys, and each decoder to the encoder.

### 4.2 Tags

The first byte of every key names its owner:

| Tag | Owner | Then |
|---|---|---|
| 0 | the component's own rows | its family: the format row (5.3), transcripts' places (8.1) |
| 1 | jig | the child's tag, in the order of the core's key: the core's own, then each child's; then the family's key; and one row of the component's, the installation's identifier, at its reserved key (4.4) |
| 2 | the application | its own key bytes, which it encodes by the same rule, so that bytes sort as its keys do |

- **The application's bytes stay under its tag.** The component prefixes
  them, and refuses an application's key or interval it would have to
  write outside it, which is a bug, as a contract break.
- **A child's keys are one interval,** and each family one interval
  within it, since the encoding follows the keys' nesting.

### 4.3 Ranges to intervals

The domain asks for ranges as predicates over its keys; the store reads
intervals. So:

- **each range maps to a short, fixed, ordered list of intervals,** in the
  component, and a page that exhausts one interval with room left goes on
  into the next in the same step, since skein-kv answers loads at once;
- **families loaded together are adjacent:** within each child, the
  families a restart loads come first, and history after them
  (`../domain/engine.md`, 5.5), so each child's live part is one interval;
- **a test checks each mapping** against the range's predicate over
  generated keys: every key the predicate admits lies in an interval, and
  every key in an interval is admitted.

### 4.4 jig's families

What the store keeps of jig, by owner. "At start" families are read by
the restart (section 10), the others by a load, a read or a get when
wanted. The families' keys are the children's; the order within each
child puts live first.

| Owner | Family | Key | Loaded |
|---|---|---|---|
| component | the format row | none: one row | at open |
| component | the installation's identifier, at its reserved key in the accounts' family (01 04 03): drawn by the shell at each start (`configuration.md`, section 8) and kept by the store's first commit, beside the format row (5.3), so that each later start's draw is ignored | none: one row | at open, handed up as the store opens, never by the restart's ranges (7.2) |
| component | transcripts' places: each task's committed length and its current window's start; each attempt's first turn | task; task, attempt | at open, the run proofs' tasks' lengths (8.4); the rest when read |
| core | the deployment: its id, its numbers (the next project's among them, past every project seeded or made), the last commit | none: one row | at start |
| core | settings (`../domain/engine.md`, 17.7): the defaults the newest revision was made with, a record per setting; the changes over them, a record per setting changed (its value whole, or its removal, with the revision that made it); the held revisions' records; the charter versions held revisions read and the newest does not; pending proposals' sets | kind, then scope (0 for the deployment, a project's number), then number, for defaults and changes; revision; charter, then the revision that made it; proposal | at start, after the deployment's row, before every other family of the core's |
| core | settings' history: the records of revisions no longer held, each with its number, the revision it was made against, when, who decided it, the proposal it came from, whether it moved running work, and its items with the values they set; within the history limit, the oldest erased by the decision that makes a newer one | revision | on demand |
| core | goals' projections (feed, plan rows, pending milestones), run proofs, calls committed since a live task's last turn, live agent tasks' conversations (their first attempt, their current window's start) | a goal's, then its own; task; task, attempt, completion, position; task | at start |
| core | turns (in transcripts' files, section 8), terminals, decisions on proposals and escalations | task, attempt, turn; task, attempt; proposer, proposal; task, revision | on demand |
| tasks | live tasks, each with its revision and its claim's (`../domain/engine.md`, 17.3), stubs naming ended results, the funders' live periods, writer slots, pools' slots, pending proposals | task; task; funder; slot; proposal | at start |
| tasks | ended tasks, past periods, decided proposals, history, milestones, activity | task, then a position or revision; funder, period; proposal | on demand |
| people | parties, sign-ins (no secret), roles, projects' names and whole policies (their ceilings, escalation roles and overrides of templates among them), seeded and made alike, read positions over results and over live conversations, answered keys within the retention, a party's or, for the deployment's acts, a connector's (`../domain/people.md`, 5.3), including program declarations and checks, their declared names, and sign-out/removal answers naming unconfirmed configured workers with their original failure reasons | as the people child keys them; a live conversation's position by party, then task | at start |
| people | ended conversations' read positions (7.4) | task, then party | at start, by the census, every row (7.2); otherwise on demand |
| accounts | issued program names, including retired ones, with their source number and original holder and desired sign-out (key tag 4, after the installation's reserved tag); the next number the accounts child gives, below which it gives none again; LLM accounts: each one's holder, LLM provider, subject and display name, and its uses; their sources, each at its place in its holder's order, with its kind, its name for a program's, and its hold: limited, with its reset as a wall time where it has one, refused, or moving (a signed-in source whose record is out with a move, or back and not yet refreshed: `../domain/accounts.md`, section 6). The rest of a source's state is heard again at each start, and none is kept per host (`../domain/accounts.md`, 3.5) | source for an issued name; none: one row; account; account, source | at start, issued names before live account/source records |
| notes | entries, and an index line per entry per scope | name; scope, name | when a scope is in use; entries on demand |
| tasks | the results' index, the inbox's census's (7.2): a row per result of a task a party requested, at its position among that party's results, past the tasks child's live families | party, position | at start, by the census, past each party's read position over results |
| tasks, people | indexes for parties' reads (7.4): each party's inbox by time; ended tasks by requester party and by project; ended delegates; a subtree's delegates | the index's own, newest first where it reads so: an ended index's by party or project, then label, ended at reversed and task | on demand |

Children that keep nothing durable (the views, the fleet, authority,
briefs) have no family. What a run reports as it goes (its text, its
activity) is not stored: each committed turn holds what the run said
and did. Nor is a program's login on a host, which its host reports
(`llm.md`, 5.5), or a sign-in under way, which a restart ends
(`../domain/accounts.md`, section 5). An account's values are in the
secrets' store (section 9).

## 5. Records

The current tasks family is version 9: its durable `MessageKind` adds
`FinishRequested { revision }` for a cancel grace. Tasks version 8 stores
refuse to open under the existing format check; no migration is added.
Component2/core5/people8/notes1 remain unchanged. The accounts family
is version 2: a source's hold adds *moving* (`../domain/accounts.md`,
section 6); accounts version 1 stores refuse to open likewise.

Settings (`../domain/engine.md`, section 17) add the core's settings
family's records (4.4) and a revision on each task's record and claim,
each a new version of its family, taken from main's constants when its
milestone lands. A store of an earlier version refuses to open under the
format check (5.3), as every change of format before the baseline does;
nothing is translated. A deployment made again from the same document,
with no change stored, behaves as it did. Each record of a setting
holds its value whole, a charter's instructions as text; their sizes
follow from the limits that bound each kind (5.4).

### 5.1 Codecs

- **One schema per family,** generated by skein's generator (skein's
  `codec.md`), used by the component only: the core's records, each
  child's, the component's own. The domain's records are translated to
  and from the generated ones with small total functions, exhaustive both
  ways, so a domain change to a durable field breaks a translation's
  build the day it happens.
Open's ending authority is carried in tasks v7 and people v7. Core's
schema remains v4. These are the versions the format row requires.

- **Each record carries its family's version,** as every generated record
  does (skein's `codec.md`, section 5), which is what an upgrade will
  need.
- **A collection the domain bounds by its bytes in all is one piece**
  (skein's `codec.md`, 3), its limit the domain's sum: a verdict's
  fields by the result's bytes, an amendment's lists, an authority's and
  a policy's by the amendment's and the policy's bytes. Only a
  collection the domain bounds by a count of its own is a counted list.
  So a family's limits, made from the domain's (5.4), are never past
  its ceilings for a count the domain does not have, and a record's
  worst case is the sum, not a count times its largest item.
- **Account preferences are separate from claims.** People's retained asks
  keep a choice's optional account number, including through amendments and
  proposed batches. Tasks keep that preference and their last claimed account
  separately; the run proof keeps every provider/account/source binding.
- **Authority models retain their selector** (`../domain/authority.md`,
  section 3). Literal ALL is distinct from a named set, including an
  explicit empty set and a set of all 64 configured models. The people
  family (version 5) keeps one bounded name context per policy row or retained
  ask. Tagged authority masks reference that context, including nested Create members, Amend, compound operations,
  proposals and ChangePolicy roles. A retained ask's names are unique and sorted by their
  bytes (policy snapshot contexts follow configuration order), at most 64, each at most 4,096 UTF-8 bytes; these retained names
  are priced separately from an authority's other bounded payload.
  Omission is distinct too: Create defaults to the creator's models,
  Amend keeps the target's, an edited role keeps its models and a new
  role takes the project's ceiling. Explicit selectors replace models
  under that ceiling; literal ALL never stands for a frozen named set.
- **Retained request identity survives removal.** A retained ask keeps
  its original canonical selector, never remapped for equality. An exact
  retry returns its retained answer before current model-availability
  admission; another selector on that key is KeyConflict. A new key
  naming an unknown model is refused. Restoring effective policy maps
  names to today's configuration, dropping removed names without an
  error; only literal ALL gains a newly configured model. Answers and
  pending proposals retain their identity and outcome. A policy row
  holds no ceiling: each start restores it under configuration's.
  The tasks family's version 5 rows retain only names their own named
  selectors reference, in canonical byte order, with masks reindexed to
  that context. Literal ALL and unrelated variants retain no names.
  Live/ended authorities, pending or decided proposals, recurring members
  and amendment history remap effective masks on load. These are effective
  snapshots; the original keyed ask remains people's.
- **Added dependencies keep when and by whom.** The tasks family's
  version 6 rows keep each live task's dependency with whether it was
  named when the task was made or added by an amendment, and for an
  added one its wall and its maker, a party or a task
  (`../domain/tasks.md`, section 7); a hold for a circle of waits, with
  the circle's tasks, cut at their limit, and when the held task began
  to wait for holds (`../domain/tasks.md`, 6.7); and, for a task that left
  its queues and came back, when its wait bound counts from
  (`../domain/tasks.md`, 6.2). The tasks family's activity also keeps a
  parked run's `Kept` line, its attempt and pending dependencies
  (`../domain/tasks.md`, 5.7; `../domain/engine.md`, section 7), and the
  `Met` notice, the dependency and whether it was removed
  (`../domain/tasks.md`, 8.1). The index of holders,
  waiters and dependents is not stored: the restart rebuilds it from
  the live tasks, and judges their waits whole (`../domain/engine.md`,
  section 6). People's retained answers keep the refusals for added
  dependencies as the client's contract has them
  (`../client/domain/contract.md`, section 4).
- **Golden bytes** per family and version; a fuzz target per family's
  decoder; bounds at their limit and one past it.

### 5.2 The application's records

- **Its own codecs,** generated the same way, in its protocol layer: it
  encodes each record and key before the component sees it, and decodes
  each row the component hands back, before its root sees it.
- **Its store version,** one number the application gives the component
  when it makes it, which moves whenever an encoding of its records
  changes, as its payload version does for the channel
  (`channel.md`, 4.1).

### 5.3 The format row

- **What it records:** the version of the component's format (its keys'
  encoding, its frames, its own rows), the version of each of jig's
  families (the accounts' among them), the application's store version,
  and the deployment's id.
- **Written in the first commit,** with the deployment's header, by the
  component, and the installation's identifier with it (4.4), so that a
  crash keeps both rows or neither, and the identifier is durable before
  any sign-in sends it.
- **Checked at open:** a store whose row names another version of
  anything, or another deployment than its header, refuses to open,
  saying which; so does one whose installation's identifier is missing
  or unreadable. Pre-release, nothing is translated (`README.md`,
  section 7).

### 5.4 Sizes

The component computes, from the domain's limits and the application's,
the largest encoded key, value and commit, and refuses to start when
skein-kv's limits are smaller (its key, value, operations and commit;
skein's `kv.md`, section 7), so a commit the root admitted is never
refused as too large.

- **The largest commit is what it can leave in the map,** not its
  writes times the largest row: each key a commit puts is in the map
  once it is applied, at most skein-kv's budget in all, since a commit
  writes each key once (section 6) and the fill keeps the map within
  its budget (11.2). So the largest commit is the map's budget, plus,
  for each operation the journal's writes allow, a key and its framing.
  A deployment's commit then fits skein-kv's `u32` whenever its map's
  budget does.

## 6. Commits

- **Numbers are shared.** The journal numbers the root's commits, and
  skein-kv numbers its own from one, across restarts. Every root commit
  is one skein-kv commit, and nothing else writes the store, so the
  numbers are equal. The component checks this as it submits, and stops
  on a mismatch. At start, skein-kv's recovered last commit must equal
  the commit the deployment's header names, which every writing decision
  saves; a store that disagrees refuses the start.
- **Writes become operations** in order: a save a put of its encoded key
  and record, an erase an erase, the application's bytes under its tag.
  A key written more than once in one commit becomes one operation, its
  last write, which leaves the map as the writes in order would:
  skein-kv applies a commit whole (skein's `kv.md`, section 7), so
  nothing sees the writes between.
  A turn's save becomes a frame in its task's file, written first
  (8.2), and puts of its place in the map.
- **In order, whatever waits.** A commit with turns goes to skein-kv once
  their files are synced, and the commits after it queue behind it, so
  numbers reach skein-kv in order. The wait is one sync, and only for the
  commits behind a turn.
- **Group commit.** While a sync is in flight, commits queue, and the next
  write carries them all (skein's `kv.md`, section 4). The root's bound
  on commits in flight is checked at start against skein-kv's queue and
  queued bytes, and the component's queue behind transcripts' syncs, so
  the store never answers a commit busy.
- **Committed n** goes up when skein-kv answers that commit, which makes
  every earlier one durable too, with the store's fill (11.2).
- **A failure stops the engine.** skein-kv answers failed for every commit
  in flight when a write or a sync fails or stalls, and a failed write or
  sync of a transcript fails its commit the same way. The component
  answers failed, the root stops (`../domain/engine.md`, 5.1), and the
  process ends; the next opens the store, which recovers. The engine
  never carries on over a store that recovered beneath it.
- **A refusal is a failure.** Past the checks at start, a commit skein-kv
  refuses (too large, the map full) would be a decision already made and
  not kept, so it stops the engine as a failure does. Section 11 keeps
  the map from filling.
- **A commit's operations are bounded,** as skein-kv applies each commit
  in one step (skein's `kv.md`, section 7). Where removing a record would
  erase an unbounded number of rows (a transcript's attempts' places,
  8.5), its commit erases the row that names the rest, so nothing reads
  them from then on, and the rest, orphans, go in the commits that
  follow: the component adds a bounded number of their erases to each
  commit it makes for the root, within that commit's operations, until
  none is left. At open it finds any a restart left by the rows they
  name being gone. Every commit stays one decision's, numbered as the
  root numbers it.

## 7. Loads

### 7.1 Pages

- **Same step.** skein-kv serves a page from memory in the step that asks
  (skein's `kv.md`, section 2), and the component answers in that step
  too, through its outputs, never by recursion. Only transcripts' bytes
  take io (section 8).
- **The cursor is exclusive.** A page's cursor is the last key it
  returned; the next page starts just past it, and an absent cursor at
  the interval's beginning. skein-kv's ranges start inclusively, and its
  next key is the first it did not return, so the component asks from
  the cursor followed by a zero byte, the least key above it.
- **Two bounds.** skein-kv bounds a page by encoded bytes, the asker by
  rows and decoded bytes. The component keeps the longest prefix that
  fits both.
- **A row that does not decode** stops the engine, as a failed commit
  does: the store holds bytes it did not write, or that broke since.

### 7.2 The restart's ranges

The core's restart script (`../domain/engine.md`, section 6) asks the
root for one step at a time, and the root asks the store:

- **the core's live ranges,** in the order the core restores them: the
  deployment's header first, then the settings' live ranges (defaults,
  changes, held revisions, charter versions, pending proposals' sets),
  which every later record is judged against, then the accounts' family (its counter,
  accounts and sources: the interval stops before the installation's
  identifier, which the component keeps), then people's families (live
  conversations' read positions among them, ended ones never), then the
  tasks child's live families, then the core's own live rows (goals'
  projections, run proofs, calls, agents' conversations). It is a fixed
  list of intervals, in that order, paged and handed up typed. The root
  has the accounts child want its sources only once these are restored
  (`../domain/accounts.md`, 3.5);
- **the inbox's census,** a step of its own once people's read positions
  are restored: the family of ended read positions, one row per ended
  conversation not yet erased (7.4), then the results' index from each
  party's read position over results (4.4), parties in the census's
  order; each row joined to its ended task's record by a point read;
  never every ended task (section 15). It then reads each unanswered
  question's activity line by the position its live task keeps (15.7),
  to rebuild the core's bounded, volatile question facts: the original
  time and words, never another durable record. The census:
  - hands up an ended read position, then its task's record, and a
    result's row only as its task's record;
  - never parts a position from its record: a page counts both against
    the asker's rows and bytes, and ends before a pair that does not
    fit, so a root asks the census for at least two rows;
  - goes on from the last index row's key. The census places each key
    it admits in the walk's order (`Census::interval`), so that a root
    that keeps its own cursors finds the interval to go on from (4.3);
  - restores each ended task once per party, whichever part names it:
    neither part's progress skips a row of the other;
- **each connector's live ranges,** as the application's byte intervals,
  handed up as bytes for its codecs;
- **what the core never restores** (turns, terminals, history, ended
  tasks, notes' entries) is never read at start.

### 7.3 Parties' reads

A party's read (`../client/domain/contract.md`, section 6) names a
scope, a position, a direction and its bounds. The component serves each
scope as intervals over jig's families, and joins each row in the same
step by point reads, those its scope's rows carry: its owner task, live
or ended; its requester ancestry, within the domain's depth limit; its
links; the record it relates to. A task's activity joins its owner
alone, once a page, and each turn's line its turn (below).

| Scope | Served from |
|---|---|
| a task, a person task, a result | the task's record, by key, with its last attempt's terminal |
| a proposal | a party's: its record, by key, with its decision; a task's: the row of its proposer's history that keeps it |
| an escalation | its decision, by the revision its task's record names, with the hold's milestone |
| a question, a reply | the task's activity line, by its position, with the line that answers a question, where its answer was heard (below), or the turn a reply was (from the transcript, 8.3) |
| a turn | the task's transcript (8.3) |
| a task's activity, newest first | the activity family, read backwards (7.4); each turn's line with its turn, from the transcript (8.3); its owner task joined once for the page, and no row's ancestry or links, since an activity line names its task alone |
| a task's history, or a subtree's | the history and milestone families; a subtree's by the index of its delegates, task by task: the task's own rows, then each delegate's in that index's order, each task's rows in its own history's order |
| a task's ended delegates | the index of ended delegates by requester |
| an inbox group | the index of the party's inbox, by time: what needs them oldest first, updates newest first; a group, and the filter, select within it |
| a task query's live tasks | not the store's: the tasks child's live tasks, scanned in memory |
| a task query's ended tasks, naming a requester party | the index of ended tasks by requester, from the party, and the label where the query names one |
| a task query's ended tasks, naming a requester task or a parent | the index of ended delegates by requester |
| a task query's ended tasks, naming a project and none of the above | the index of ended tasks by project, from the project, and the label where the query names one: listed tasks only |
| a project's notes, a scope's notes | the notes' lines of the project's scopes, or of the one scope, filtered by description, each with its entry: by scope, then by name within it, as the lines are keyed |
| a project's spend | the funders' periods of the project |

- **A task query's ended part** (`../client/domain/contract.md`, 6.1)
  is served from one index, chosen by the query's keys in the table's
  order: a requester party; a requester task or a parent; a project. A
  query naming none of these has no ended part, and its page says so.
  The keys beyond the index's prefix (the label, where it is not in it;
  the root; the phase class; the executor kind; tracked) filter its
  rows. Its rows come in the index's order: within a label, newest
  ended first from the two ended indexes; by task from the ended
  delegates', whatever order the query asks of its live part.
- **A task query's row joins** its task's record, by key: the ended
  record, or its summary once retention has shrunk it (11.1); and, for
  a task flagged conversation, its requester's ended read position, by
  a point read (7.4).
- **A page goes on from the last row it examined,** in its scope's
  order, not from the last it returned. A page that skips rows (an index
  row whose owner is gone, 7.4) or leaves them out (an inbox group, a
  description's filter, a task query's filter beyond its index's
  prefix) goes on past them; a page that reached its bound
  and returned none is cut, not ended, whichever its scope and whichever
  bound it reached (rows, bytes or joins), and carries the position it
  reached, so the client reads on from the row it did not take. A
  subtree's history is in its scope's order, task by task,
  its position the task and the row.
- **Joins are point reads** in the same step, priced within the read's
  bounds: a page's rows times the joins each may need. An owner lookup
  is charged one join, whether the owner is found live, found ended
  after its live record is missed, or not found: a miss decodes
  nothing, so the second probe needs no join of its own.
  A turn's is read from its transcript, and answers once its frame is read.
  An activity page's joins are its owner's, once, and one for each turn, so its
  first row needs at most the owner's and a turn's.
- **A page of a task's activity carries its turns.** Each turn's line
  comes with its turn: its spend, the last message it read, its time,
  and smith's bytes. A turn and its line commit together, so a page's
  turn lines are the transcript's frames in the same order, one each,
  and the component reads them in one walk (8.3). A turn counts its
  bytes with its line's against the read's byte bound, and one join:
  - a turn whose line and bytes together pass the read's whole byte
    bound comes with its size in place of its bytes, and counts without
    them, so no page comes back empty for a turn;
  - one that fits the bound but not what is left of the page ends the
    page before its line, and the next page starts with it;
  - a frame that does not read whole (8.3) comes as unread, and ends
    the page.
- **Single reads go by key.** Where a task's record does not name the
  key a single read needs (a question's line and its answer's, an
  escalation's hold, a task's proposal in its proposer's history), the
  domain names it there, or keeps an index (section 15).
- **An answer is read where it was heard** (`../domain/engine.md`, 7.2;
  who may read it, `../client/domain/contract.md`, section 6): the
  question's record names the answer by its task and position, and the
  read joins it by a point read, as the record it relates to.
- **A read past its bounds** is cut and says so, as every load is
  (`../domain/engine.md`, 5.3).

### 7.4 Indexes

skein-kv has no secondary index (skein's `kv.md`, section 1). Where a
scope's order is not a family's key order, the domain keeps an index: a
family of small rows its owner saves and erases in the same decision as
the records they point at, with keys reversed in time where a page reads
newest first. Each index is its owner's, tested in its world. Section
15 lists those the domain owes.

**The ended indexes,** the tasks child's, each saved in the decision
that ends a task (`../domain/tasks.md`, section 3):

- *ended by requester:* key `(party, label, ended at reversed, task)`, a
  row for every ended task whose requester is a party;
- *ended by project:* key `(project, label, ended at reversed, task)`, a
  row only for an ended task flagged *listed*. The flag is the task's,
  set from its template when it was made, so the engine reads no
  template to index.

Each serves a task query's ended part from its prefix, the party or the
project, then the label (7.3). A filter beyond the prefix scans the
index's pages, which may come back cut. A task moved to a party
(`../domain/tasks.md`, section 7) is indexed by the requester it has
when it ends.

**Read positions over conversations,** the people child's: one per task
flagged *conversation*, its requester's, while that requester is a
party (`../domain/people.md`, section 6):

- *a live conversation's,* in a family keyed `(party, task)`, loaded at
  start, so their bound in memory stays the live conversations, one
  each;
- *an ended one's:* the decision that ends the task moves its position
  to the family of ended read positions, keyed `(task, party)`, read on
  demand. A read of the conversation and a task query's row join it by
  a point read (7.3); the inbox's census reads the family through
  (7.2);
- *erased with the task:* by retention, as it shrinks a kept task
  (11.1); or in the decision that forgets a task forgotten once read
  (below).

**An owner erased first.** When a task's record is erased before the
records its index rows name, as a task forgotten once read is
(`../domain/people.md`, section 6; 11.1), its decision erases the rows
its record names whole: its result's update, its rows in the ended
indexes, and its ended read position. Its replies' updates stay, since a
conversation's replies are unbounded, a decision is bounded, and the
task's record names only its newest; they go when retention erases the
reply lines they name (11.1). Until then a page skips a row whose owner
is gone, counting it within its bounds, so nothing of a forgotten task
shows and no reply read comes back unread.

**Reading backwards:** a family whose positions are dense (activity)
reads newest first by computing forward intervals from its last
position, which its owner task's record names; a family read newest
first by time is keyed reversed. skein-kv's load runs forwards only
(skein's `kv.md`, section 12).

## 8. Transcripts

### 8.1 A file per task

- **One file per task,** named by the task's number, in directories by
  the number's high bits, so each directory holds a bounded number of
  files. The name is derived: nothing lists the files.
- **A turn per frame,** appended in commit order: the turn's attempt and
  number, its spend, the last message it read, its time, whether it opens
  a window, then smith's bytes, with a checksum, and the frame's length
  at both ends, so the file can be walked from either end.
- **The map keeps each transcript's places,** in the component's own
  family: its committed length and the offset of its current window's
  first turn, one row per task with a transcript; and each attempt's
  first turn's offset, one row per attempt. A turn is found from its
  attempt's first turn, by walking forwards.
- **Windows.** A frame whose turn opens a window moves the task's
  current window to it, in the same commit. A run that starts fresh opens
  a new conversation, whose first turn opens a window (smith's
  `protocol/transcript.md`, section 3).

### 8.2 Written before its commit

The core commits a turn with its spend, its run proof and the messages it
read, in one decision, and acknowledges the host once that commit is
durable (`../domain/engine.md`, 7.2). For a commit with turns, the
component:

1. writes each turn's frame at its file's end, opening the file
   (`OpenWrite`), or creating it where it is missing. The end is the
   committed length, plus the frames queued ahead and not yet durable,
   since a task's next turn may be decided before its last one commits;
2. syncs each file it wrote, and, for a file it created, the directory
   that holds it, and that directory's own, for a directory it made;
3. submits the commit to skein-kv, with each transcript's new places among
   its operations;
4. answers committed once skein-kv has.

A crash after step 2 and before the commit is durable leaves a frame past
the committed length. The next open cuts the file to its committed length
(8.4), and the host, never acknowledged, sends the turn again.

### 8.3 Read back

- **Resuming:** a run that resumes is given its current window's turns,
  from the window's first turn to the end, within the resume limit
  (`../domain/engine.md`, 7.2), read forwards a page at a time while the
  task prepares. The core asks only when its conversation has a window,
  and names the turn that opens it, as it keeps it (`../domain/engine.md`,
  5.4); a window that starts at another turn fails the read as unread.
- **A turn,** for a party's read: from its attempt's first turn, walking
  frames forwards to its number. A turn read alone is bounded by the
  read's byte bound (7.3): past it, the read answers the turn's size in
  place of its bytes (`../client/domain/contract.md`, section 6).
- **A page's turns,** for a party's page of a task's activity (7.3):
  backwards, one frame for each turn line, from the frame of the page's
  newest turn: the transcript's last frame when the page reads from the
  activity's end, otherwise found as a turn is. A turn past the page's
  bound has its fields and length read, not its bytes.
- **A page of a task's turns,** forwards or backwards, from a cursor that
  is an offset the component gives and takes back.
- **A tail,** for a fresh run's brief: backwards from the end, within the
  section's budget.
- **Never past what is committed.**
- **A frame that fails its checksum** fails the read as unread. The core
  treats the transcript as one the agent cannot use: the run fails as
  transient and the next starts fresh (`../domain/engine.md`, 7.2).
- **Open files are bounded:** the component keeps a bounded set open,
  live tasks' first, and opens others to read them.

### 8.4 After a crash

At open, before the root starts, the component reads the core's run
proofs, one for each live task ever claimed (4.4), and cuts each such
task's file to its committed length, syncing it: zero for a task with no
places yet, whose first turn's commit was lost; a missing file is left
missing. Only a crash leaves bytes past a committed length, since a
failed commit stops the engine, and only a claimed task has frames, its
claim durable before its run was placed (`../domain/engine.md`, 5.2).
After the cut every file holds exactly its committed turns. An ended
task's file was cut while its task lived, and is not touched again.

### 8.5 Dropped

Retention drops a task's transcript whole (11.1): the commit erases the
task's row of places, and the component removes its file and syncs the
directory before it submits the commit. The task's attempts' places,
as many as its attempts, are orphans from that commit on, read by
nothing, and go in the commits that follow, a bounded number each
(section 6, "A commit's operations are bounded"). A crash
before the commit leaves places whose file is gone: a read of it answers
as for a dropped transcript, and retention's next pass drops it again.
Removing a missing file is not a failure.

## 9. Secrets

- **A second skein-kv store,** in `secrets/`, opened as a private
  directory (skein's `io.md`, 5.3): the directory `0o700`, its files
  `0o600`, owned by the engine's user. The store, its snapshots and its
  logs hold no secret.
- **What it keeps,** each under its owner's tag and the number the domain
  gives it:
  - minted credentials' digests, by their sign-in's number, with their
    expiry: each credential the clients' component mints, for a printed
    link's sign-in, a service's, or an identity the application's server
    verified (`client.md`, section 8);
  - services' credentials' digests, by the service's number;
  - each signed-in source's record, by the source's number: its tokens,
    the client id the provider issued, the scopes granted and the ID
    token, as skein's accounts driver hands it to keep (skein's
    `oauth.md`, 6.4; `llm.md`, 5.1), or as a moved record brought it
    (`llm.md`, 5.6);
  - each LLM key, by its source's number, adopted from the value its
    request carried (`client.md`, section 5);
  - an application's own records, under its tag: its connectors'
    credentials, which it decodes itself, by numbers below 2^63;
  - each value a request carried (`client.md`, section 5), an LLM key
    or an application's value, under the application's tag, by a number
    jig draws itself from 2^63 up, never one the request names: the next
    number is a row of its own beside the values (key `[2, 1]`), written
    in the commit that keeps the value, so a number is never drawn
    twice, an erased one included, and a number already holding a value
    is never written over: the carry is refused and nothing is kept.

  A program source's login is not here: it is the program's own, on
  each host (`llm.md`, 5.5).
- **Written by components alone,** through the protocol layer's routing.
  The domain names a secret by number, never holds one, and never sees
  the secrets' store's commits, which have numbers of their own.
- **Kept before used.** A sign-in is committed first, which gives it its
  number; its credential's digest is then kept, and durable; only then
  does the clients' component answer the credential, to the client that
  signed in or to the application's server that asked it to mint
  (`README.md`, section 8). A crash between leaves a sign-in no client
  holds a credential for, which expires with its time.
- **A record kept before lent.** skein's driver takes this store as its
  keeper (skein's `oauth.md`, 6.4): each new record goes out as a keep,
  which the routing writes here under its source's number, stating the
  digest of the record it replaces, and answers `kept` once it is
  durable; the driver lends the record's token only then. A crash
  between leaves at worst a record its driver never lent, which the
  next start lends from.
- **A carried value kept before the core sees it.** The clients'
  component takes the value out of the request carrying it as it
  decodes it, and keeps it here before the core gets the request, which
  names it by its number and digest only (`client.md`, section 5).
  Where the core leaves the request unused (refused, a key sent again
  under a key answered or in flight, or the engine stopping) it says
  so, naming the number, and the component erases that value, if its
  own decode kept it; replacing or erasing a kept value goes through
  the core's decisions alone.
- **An LLM key adopted before granted.** The accounts component checks
  the carried value against its byte bound, then asks the store to
  adopt it into the source's row, naming the digest it replaces, or
  none for a new source. Adoption writes the source's row and erases
  the carried row in one secrets commit; a missing carried value or a
  changed destination digest answers stale and changes neither row. The
  component grants the key only once adoption is durable, a replaced
  key as its next generation (`llm.md`, 5.1). A later erase of the
  carried number is a no-op. A source's record or LLM key occupies one
  secret row under a fixed-size store key.
- **A moved record adopted the same way,** in one secrets commit
  (`AdoptValue`, beside the key's `Adopt`) that writes the source's row
  and erases the carried row: the row holds
  the record the carried file holds, in skein's form, not the file's
  bytes as they came (`llm.md`, 5.6), and the adoption states the
  carried value's digest, answering stale when it changed. The record
  is added to the driver only once adoption is durable, and lent only
  after its first refresh is kept.
- **A moved-out record is read, never copied in the store:** the
  routing reads the source's row once the driver has removed it, and
  the file's bytes made from it live only in the protocol layer until
  the client takes them (`llm.md`, 5.6). The row is erased when the
  move is written.
- **A presented credential names its sign-in:** a credential is the
  sign-in's number and its random bytes, opaque to the transport that
  carries it. The component gets the digest by number, and the clients'
  component compares in constant time.
- **Erased** when the core says a sign-in ended (signed out or revoked)
  or expired, or a service was removed; and when the accounts child
  says a source was signed out or removed, a key replaced, a record
  revoked, or a move written. A crash between that commit and the erase leaves a value
  nobody names, which the restart sweeps: once people's families are
  restored, the component erases each of jig's digests whose sign-in or
  service it did not hand up. For sources, the accounts child names
  each party source to restore before wanting it, then says the list
  is complete. The accounts component re-adds signed-in sources and
  loads keys by number; it then pages the source records once and
  erases each neither configured nor named, including orphaned keys.
  Configured sources are spared even without an accounts-family row.
  Every application row is spared: a carried value an application used
  may still live at its drawn number. A value orphaned by a crash
  before the core says used or unused remains there, including an LLM
  key not yet adopted; collecting those values is later work.
- **A failure stops the engine,** as the store's does (section 6): what a
  failed commit of the secrets' store was keeping is never handed out.
- **Read at start,** within its own limits and worst case: named sources
  by number, the sources' sweep in bounded pages.

## 10. Opening and restart

1. **The shell opens the deployment's directory,** and the component opens
   both stores; skein-kv recovers each: the snapshot, then the log's
   longest valid prefix (skein's `kv.md`, section 3).
2. **The component reads its own rows,** and the core's that it checks:
   the format row, refusing another version or deployment (5.3); the
   deployment's header, refusing a last commit other than skein-kv's
   recovered one (section 6); the run proofs' tasks' committed lengths,
   cutting each file to its length (8.4); and the installation's
   identifier, which replaces the one the shell drew (4.4). An empty
   store gets its format row and the installation's identifier in the
   root's first commit, with the deployment's header.
3. **The root starts** (`service.md`, section 3): the core's restart
   script asks for one step at a time, and the store serves each step's
   ranges (7.2). The accounts child's want saves its never-reused
   counter when it advances, and the ordinary account/source rows it
   makes for providers whose source is none. A first start with no
   accounts needs no such commit. Restarts retain those same numbers.
4. **Secrets are served** to the protocol layer's other components once
   the secrets' store is open: each signed-in source's record to make
   skein's driver with, and each key to the accounts' table, when the
   accounts child names the restored sources. After all are named, the
   component sweeps unnamed source secrets (`llm.md`, 5.1).

## 11. Growth

### 11.1 Retention

Retention is the domain's: mechanisms on its timers, tested in its
worlds, with horizons the deployment's configuration gives. What becomes
of an ended task is its retention class, set from its template when it
was made (`../domain/tasks.md`, section 3):

- **A task kept** stays to the deployment's retention horizon, and whole
  while none is set.
- **Ended tasks shrink** past a horizon: one commit erases a task's full
  records (its record, history, activity, terminals, its ended read
  position) and saves a summary, so what parties read of old work comes
  from summaries. Its rows in the ended indexes stay, and a task query
  joins its summary (7.3).
- **Transcripts go** past a longer horizon, whole, by their task (8.5).
  A live task keeps its transcript, whatever its age.
- **Answered keys expire** past the people child's retention.
- **A task forgotten once read** (`../domain/people.md`, section 6) goes
  in one decision: its record, its rows in the ended indexes, its
  result's update and its ended read position (7.4).
- **A forgotten task's remains go** at retention's next pass, whatever
  their age: its activity, history, terminals and transcript, and its
  replies' rows in its requester's updates (7.4). Nothing reads them,
  and the requester's pages of updates skip those rows until then.

### 11.2 Fill

- **The store reports its fill** with each commit's answer: the map's
  bytes against its budget (skein's `kv.md`, section 7).
- **Above a high-water mark** of that budget (both are limits,
  `configuration.md`, section 6), the domain refuses new work at its
  entrances: new tasks, new sign-ins. Work under way goes on,
  and a refusal is never a failed commit. The deployment's document
  shows the fill.
- **Transcripts' disk** is the deployment's: retention bounds it, and its
  directory's free space is the operator's.

## 12. The application's part

- **Its families,** under its tag (4.2), keyed by its own encoder by the
  same rule, encoded by its own codecs, with its store version (5.2).
- **Its restart ranges** as byte intervals under its tag, each live
  interval of each connector, served in the restart's step for that
  connector (`../domain/engine.md`, section 6).
- **Its loads and gets** by byte intervals and keys, answered with bytes.
- **Its indexes,** as records of its own, by the same rule (7.4).
- **Its secrets,** under its tag in the secrets' store (section 9).
- **What it writes is in the same commits** as jig's: one decision, one
  commit, whoever's records it holds.

## 13. Limits and the worst case

- **The component's limits:** both stores' (skein-kv's, their maps'
  budgets among them), the fill's high-water mark, the open files,
  the frames' largest bytes (a turn's, from the domain's limits), the
  queue of commits behind transcripts' syncs, the reads in flight and the
  joins a page may make.
- **Three file drivers,** one for each store (skein's `kv.md`, section
  12) and one for transcripts, so a turn's sync never queues behind a
  store's write, nor a store's behind a turn's.
- **The worst case** is both stores' (skein's `kv.md`, section 7: about
  three times each map's budget), the component's buffers and tables, and
  the three drivers' (skein's `io.md`, section 5).
- **Checked whole at startup** with the root's and every component's
  (`configuration.md`, section 6).

## 14. Testing

Each within jig's suites' budgets (`../../development/workflow.md`,
section 2).

- **The real store in the domain worlds.** The worlds run the component
  over skein-kv's in-memory mode, in place of a fake of the map, so every
  story runs the real keys, codecs, intervals, paging and joins. No
  world fakes the store's component: the real one runs in each
  (testing-strategy.md, section 4).
- **Faults come from the world.** The world holds commits between the
  root and the component, in order: it delays them to make the store
  slow, and fails one to stop the engine. A crash keeps what reached the
  store and loses what the world held.
- **Transcripts' files have a stand-in** in the domain worlds: an
  in-memory file system answering the component's file requests, with
  the same rule after a crash. A crash keeps both in-memory stores and
  the stand-in, and the next engine's component is made over them and
  opens as section 10 says.
- **The store's world,** `tests/store`: the component over skein-kv and
  files on skein's simulated, crashing disk, a scripted root committing,
  loading, reading and appending turns, crashed at every file operation
  and sync. Its referee: after recovery, the last acknowledged commit, or
  that and one whole uncertain one, never part of one; no turn readable
  past its commit and none missing before it; numbers that match the
  header; pages equal to a model's.
- **Step tests:** each key encoder against the derived order; each
  range's intervals against its predicate; each codec both ways, with
  golden bytes; refusals at open (format, deployment, sizes, numbers).
- **Fuzz targets** for each family's decoder and a frame's.
- **On the real ring,** the reference binary (`service.md`, section 7)
  killed around each phase of a turn's commit and started again. A killed
  process keeps the page cache, so this checks integration, not power
  loss (skein's `kv.md`, section 9).

## 15. What it owes and is owed

- **The domain owes:**
  1. **live before history** in each child's keys: the tasks child's
     history and ended tasks after its live families (stubs among
     these), decided proposals apart from pending ones, and past funding
     periods apart from live ones (4.3);
  2. **the restart reads live work only:** the inbox's census reads
     unread results from the results' index and ended conversations from
     the family of ended read positions, not every ended task, joining
     each to its record by a point read; the census hands each ended read
     position up with its record, and the people child keeps none (7.2;
     `../domain/people.md`, section 6);
  3. **indexes for parties' reads:** each party's inbox by time; ended
     tasks by requester party, and by project for those flagged listed,
     each keyed by label and ended at reversed, the flag read from the
     task, never from a template; each task's ended delegates; a
     subtree's delegates for its history (7.3, 7.4);
  4. **turns as the store keeps them:** saved with whether each opens a
     window, never erased one by one, a task's transcript dropped whole;
     a resume reads the current window, from the turn the core keeps as
     its start (8.3; `../domain/engine.md`, 5.4 and 7.2);
  5. **fill:** with each commit's answer, and new work refused above the
     high-water mark (11.2);
  6. **retention:** by each task's retention class, summaries of ended
     tasks kept, transcripts dropped, answered keys expired, and the
     remains of tasks forgotten once read collected, each on the
     domain's timer (11.1); a task forgotten once read erased in one
     decision (11.1; `../domain/people.md`, section 6);
  7. **keys for single reads:** a question's line and its answer's, an
     escalation's hold, and a task's proposal's row in its proposer's
     history, each named by its position in its task's record, as a
     reply's is, an answer's with the task whose activity holds it, so a
     read finds it by key, never by a scan (7.3);
  8. **read positions over conversations:** a live conversation's in the
     people child's family, loaded at start; moved, by the decision that
     ends its task, to the family of ended read positions, read on
     demand; erased with the task (7.4, 11.1).
  9. **the accounts' family:** the next number; each account with its
     holder, LLM provider, subject, display name and uses; each source
     with its place in its holder's order, kind, name for a program's,
     and its hold, limited with its reset or refused; restored at start,
     a record breaking the child's rules counted and left in the store;
     a source's record or key named by the source's number, and
     erased when the source goes (4.4, section 9;
     `../domain/accounts.md`, 3.5). The installation's identifier is the
     component's (4.4, 5.3).
- **skein owes:** io's `OpenWrite` and `Cut` (skein's `io.md`, section
  5), for 8.2 and 8.4. skein-kv serves as it is: its in-memory
  mode, its key writer, its fill. Its accounts driver takes the owner's
  store as keeper, made with the records it holds (skein's `oauth.md`,
  6.4).
- **The application owes:** its keys' encoders, its records' codecs, its
  store version, its live intervals, and its indexes, by the rules of
  section 12.

## 16. Open questions

- **Upgrades:** reading old shapes as they load, or a pass at opening
  that rewrites them, once a deployment's data must outlive a release
  (`README.md`, sections 7 and 14).
- **Backups:** copying the directory while the engine is stopped works;
  an online backup would copy the snapshot, the segments after it and the
  transcripts, and rest on recovery's rule.
- **Finding a turn faster:** a row per turn in the map, if walking an
  attempt's frames proves slow for parties' reads.
- **Values on disk,** with only keys in memory, if a deployment outgrows
  its map's budget after retention (skein's `kv.md`, section 12).
- **Search** over tasks, notes and transcripts: an index of the domain's,
  or a file-based one beside the store.
- **Several engines:** the store is one process's; the log of commits is
  what would be replicated.
