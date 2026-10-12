# The store

Provisional, 2026-10-12. What lies beneath the engine's store vocabulary:
skein's key-value store, embedded in the engine's process, holds every
record jig's core and the application keep, transcripts are files beside
it, and secrets are in a second store. jig ships the component that does
this, `jig-protocol-store`; the application hands it its own records as
bytes it encodes. The domain's half (commits, loads, what is kept) is
`../domain/engine.md`, section 5, and what the rest of the protocol layer
relies on is `README.md`, section 9. Read it with skein's `kv.md`. What is
still open is listed in section 16.

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
  to open, and nothing is translated until upgrades are designed
  (section 5).
- **One decision, one commit, the same number.** Each of the root's
  commits is one skein-kv commit with the same number. Group commit folds
  bursts into one sync. A failed or refused commit stops the engine
  (section 6).
- **Loads are answered in the step that asks,** from memory, in bounded
  pages: the restart's live ranges, parties' reads with their joins, a
  child's pages, single records (section 7). Only a turn's bytes, read
  from its transcript, take io.
- **Transcripts are files, one per task,** each turn a frame written and
  synced before the commit that accepts it. The map keeps where each
  task's current window starts, so a resumed run reads only its window
  (section 8).
- **Secrets in a second store,** in a directory only the engine's user
  may read, written by components alone (section 9).
- **Growth is bounded by retention and fill:** the domain shrinks ended
  work past horizons on its timers, and refuses new work above a
  high-water mark of the map (section 11).
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
| **secrets**: keep, erase, get, by owner and number (section 9) | kept, erased, the value or none |

- **Turns are records of the core's** (`../domain/engine.md`, 5.4): the
  core saves each accepted turn, keyed by its task, attempt and number,
  with its spend, the last message it read, its time, whether it opens a
  window, and its bytes. The component keeps the bytes and those fields
  in the task's transcript file, and the transcript's places in the map
  (section 8). The core never erases a single turn: a task's transcript
  goes whole, by retention (11.1).
- **Reads see what is durable.** A load, a read, a get or a transcript
  sees every commit answered before it was asked, and nothing that is
  not durable (skein's `kv.md`, section 2).
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
| 1 | jig | the child's tag, in the order of the core's key: the core's own, then each child's; then the family's key |
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
| component | transcripts' places: each task's committed length and its current window's start; each attempt's first turn | task; task, attempt | at open, the run proofs' tasks' lengths (8.4); the rest when read |
| core | the deployment: its id, its numbers, the last commit | none: one row | at start |
| core | goals' projections (feed, plan rows, pending milestones), run proofs, calls committed since a live task's last turn, conversations | a goal's, then its own; task; task, attempt, completion, position; task | at start |
| core | turns (in transcripts' files, section 8), terminals, decisions on proposals and escalations | task, attempt, turn; task, attempt; proposer, proposal; task, revision | on demand |
| tasks | live tasks, stubs naming ended results, the funders' live periods, writer slots, pools' slots, pending proposals | task; task; funder; slot; proposal | at start |
| tasks | ended tasks, past periods, decided proposals, history, milestones, activity | task, then a position or revision; funder, period; proposal | on demand |
| people | parties, sign-ins (no secret), roles, projects' policies, read positions, answered keys within the retention | as the people child keys them | at start |
| notes | entries, and an index line per entry per scope | name; scope, name | when a scope is in use; entries on demand |
| tasks, people | indexes for parties' reads (7.4) | the index's own, newest first where it reads so | on demand |

Children that keep nothing durable (the views, the fleet, authority,
briefs, accounts) have no family. What a run reports as it goes (its
text, its activity) is not stored: each committed turn holds what the
run said and did.

## 5. Records

### 5.1 Codecs

- **One schema per family,** generated by skein's generator (skein's
  `codec.md`), used by the component only: the core's records, each
  child's, the component's own. The domain's records are translated to
  and from the generated ones with small total functions, exhaustive both
  ways, so a domain change to a durable field breaks a translation's
  build the day it happens.
- **Each record carries its family's version,** as every generated record
  does (skein's `codec.md`, section 5), which is what an upgrade will
  need.
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
  families, the application's store version, and the deployment's id.
- **Written in the first commit,** with the deployment's header, by the
  component.
- **Checked at open:** a store whose row names another version of
  anything, or another deployment than its header, refuses to open,
  saying which. Pre-release, nothing is translated (`README.md`,
  section 7).

### 5.4 Sizes

The component computes, from the domain's limits and the application's,
the largest encoded key, value and commit, and refuses to start when
skein-kv's limits are smaller (its key, value, operations and commit;
skein's `kv.md`, section 7), so a commit the root admitted is never
refused as too large.

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
  deployment's header first, then people's families, then the tasks
  child's live families, then the core's own live rows (goals'
  projections, run proofs, calls, conversations). It is a fixed list of
  intervals, in that order, paged and handed up typed;
- **the inbox's census,** a step of its own once people's read positions
  are restored: the index of what parties' inboxes show (7.4), never
  every ended task (section 15);
- **each connector's live ranges,** as the application's byte intervals,
  handed up as bytes for its codecs;
- **what the core never restores** (turns, terminals, history, ended
  tasks, notes' entries) is never read at start.

### 7.3 Parties' reads

A party's read (`../client/domain/contract.md`, section 6) names a
scope, a position, a direction and its bounds. The component serves each
scope as intervals over jig's families, and joins each row in the same
step by point reads: its owner task, live or ended; its requester
ancestry, within the domain's depth limit; its links; the record it
relates to.

| Scope | Served from |
|---|---|
| a task, a person task, a result | the task's record, by key, with its last attempt's terminal |
| a proposal | a party's: its record, by key, with its decision; a task's: the row of its proposer's history that keeps it |
| an escalation | its decision, by the revision its task's record names, with the hold's milestone |
| a question, a reply | the task's activity line, by its position, with the line that answers a question, or the turn a reply was (from the transcript, 8.3) |
| a turn | the task's transcript (8.3) |
| a task's activity, newest first | the activity family, read backwards (7.4) |
| a task's history, or a subtree's | the history and milestone families; a subtree's by the index of its delegates |
| a task's ended delegates | the index of ended delegates by requester |
| an inbox group | the index of the party's inbox, by time: what needs them oldest first, updates newest first; a group, and the filter, select within it |
| conversations | the index of the party's conversations, newest first |
| a project's goals | the index of the project's goals, live then ended newest first |
| a project's notes, a scope's notes | the notes' lines of the project's scopes, or of the one scope, filtered by description, each with its entry, by name |
| a project's spend | the funders' periods of the project |

- **Joins are point reads** in the same step, priced within the read's
  bounds: a page's rows times the joins each may need. A turn's is read
  from its transcript, and answers once its frame is read.
- **Single reads go by key.** Where a task's record does not name the
  key a single read needs (a question's line and its answer's, an
  escalation's hold, a task's proposal in its proposer's history), the
  domain names it there, or keeps an index (section 15).
- **A read past its bounds** is cut and says so, as every load is
  (`../domain/engine.md`, 5.3).

### 7.4 Indexes

skein-kv has no secondary index (skein's `kv.md`, section 1). Where a
scope's order is not a family's key order, the domain keeps an index: a
family of small rows its owner saves and erases in the same decision as
the records they point at, with keys reversed in time where a page reads
newest first. Each index is its owner's, tested in its world. Section
15 lists those the domain owes.

**An owner erased first.** When a task's record is erased before the
records its index rows name, as a forgotten chat's is
(`../domain/people.md`, section 6), its decision erases the index rows
its record names whole: the chat's result's update and its conversation
rows. Its replies' updates stay, since a chat's replies are unbounded, a
decision is bounded, and the chat's record names only its newest; they
go when retention erases the reply lines they name (11.1). Until then a
page skips a row whose owner is gone, counting it within its bounds, so
nothing of a forgotten chat shows and no reply read comes back unread.

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
  frames forwards to its number.
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

Retention drops a task's transcript whole (11.1): the commit erases its
places (the task's row and its attempts'), and the component removes its
file and syncs the directory before it submits the commit. A crash
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
  - sign-ins' digests, by the sign-in's number, with their expiry
    (`client.md`, section 8);
  - services' credentials' digests, by the service's number;
  - an application's own records, under its tag: its connectors'
    credentials, which it decodes itself.

  An LLM account's record is not here: skein's accounts driver keeps it
  in its private files (`llm.md`, section 5).
- **Written by components alone,** through the protocol layer's routing.
  The domain names a secret by number, never holds one, and never sees
  the secrets' store's commits, which have numbers of their own.
- **Kept before used.** A sign-in is committed first, which gives it its
  number; its digest is then kept, and durable; only then does the
  client's component hand its secret out (`README.md`, section 8). A
  crash between leaves a sign-in no client holds a secret for, which
  expires with its time.
- **A presented secret names its sign-in:** the secret a client holds is
  the sign-in's number and its random bytes. The component gets the
  digest by number, and the client's component compares in constant time.
- **Erased** when the core says a sign-in ended or expired, or a service
  was removed. A crash between that commit and the erase leaves a digest
  nobody names, which the restart sweeps: once people's families are
  restored, the component erases each of jig's digests whose sign-in or
  service it did not hand up.
- **A failure stops the engine,** as the store's does (section 6): what a
  failed commit of the secrets' store was keeping is never handed out.
- **Read at start,** whole: small, with its own limits and worst case.

## 10. Opening and restart

1. **The shell opens the deployment's directory,** and the component opens
   both stores; skein-kv recovers each: the snapshot, then the log's
   longest valid prefix (skein's `kv.md`, section 3).
2. **The component reads its own rows,** and the core's that it checks:
   the format row, refusing another version or deployment (5.3); the
   deployment's header, refusing a last commit other than skein-kv's
   recovered one (section 6); and the run proofs' tasks' committed
   lengths, cutting each file to its length (8.4). An empty store gets
   its format row in the root's first commit, with the deployment's
   header.
3. **The root starts** (`service.md`, section 3): the core's restart
   script asks for one step at a time, and the store serves each step's
   ranges (7.2).
4. **Secrets are served** to the protocol layer's other components once
   the secrets' store is open.

## 11. Growth

### 11.1 Retention

Retention is the domain's: mechanisms on its timers, tested in its
worlds, with horizons the deployment's configuration gives:

- **Ended tasks shrink** past a horizon: one commit erases a task's full
  records (its record, history, activity, terminals) and saves a summary,
  so what parties read of old work comes from summaries.
- **Transcripts go** past a longer horizon, whole, by their task (8.5).
  A live task keeps its transcript, whatever its age.
- **Answered keys expire** past the people child's retention; **ended
  chats are forgotten** once read (`../domain/people.md`, section 6).
- **A forgotten chat's remains go** at retention's next pass, whatever
  their age: its activity, history, terminals and transcript, and its
  replies' rows in its person's updates (7.4). Nothing reads them, and
  the person's pages of updates skip those rows until then.

### 11.2 Fill

- **The store reports its fill** with each commit's answer: the map's
  bytes against its budget (skein's `kv.md`, section 7).
- **Above a high-water mark** of that budget (both are limits,
  `configuration.md`, section 6), the domain refuses new work at its
  entrances: new tasks and chats, new sign-ins. Work under way goes on,
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

Each within jig's suites' budgets (`../../development/workflow.md`).

- **The real store in the domain worlds.** The worlds run the component
  over skein-kv's in-memory mode, in place of a fake of the map, so every
  story runs the real keys, codecs, intervals, paging and joins. A fake
  of the store's own component retires once the component exists
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
  2. **the restart reads live work only:** the inbox's restore reads what
     the inbox shows from an index, not every ended task (7.2);
  3. **indexes for parties' reads:** each party's inbox by time,
     each party's conversations, each project's goals, each task's ended
     delegates, a subtree's delegates for its history (7.3, 7.4);
  4. **turns as the store keeps them:** saved with whether each opens a
     window, never erased one by one, a task's transcript dropped whole;
     a resume reads the current window, from the turn the core keeps as
     its start (8.3; `../domain/engine.md`, 5.4 and 7.2);
  5. **fill:** with each commit's answer, and new work refused above the
     high-water mark (11.2);
  6. **retention:** summaries of ended tasks, transcripts dropped, answered
     keys expired, chats forgotten and their remains collected, each on
     the domain's timer (11.1);
  7. **keys for single reads:** a question's line and its answer's, an
     escalation's hold, and a task's proposal's row in its proposer's
     history, each named by its position in its task's record, as a
     reply's is, so a read finds it by key, never by a scan (7.3).
- **skein owes:** io's `OpenWrite` and `Cut` (skein's `io.md`, section
  5), for 8.2 and 8.4. skein-kv serves as it is: its in-memory
  mode, its key writer, its fill.
- **The application owes:** its keys' encoders, its records' codecs, its
  store version, its live intervals, and its indexes, by the rules of
  section 12.

## 16. Open questions

- **Upgrades:** reading old shapes as they load, or a pass at opening
  that rewrites them, once a deployment's data must outlive a release
  (`README.md`, section 7).
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
