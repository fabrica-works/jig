# jig's store: inputs for its design

Draft, 2026-10-10. Not a design: what jig's store design takes as input,
gathered while jig's protocol layer was designed (`../protocol/`). The
store is designed on its own, as jig's subset of the store design its
first application already has (that application's `docs/design/store/`),
which applications extend with their own families. This draft lists the
base and how it maps onto jig, the records jig keeps as built, what the
rest of the protocol layer relies on, the questions the design settles,
with a leaning for each, and what the domain may owe it. When the store
is designed, its content moves into `../protocol/store.md` and this draft
is deleted.

## 1. In one page

- **The base is a design on skein-kv** (skein's `kv.md`), embedded in the
  engine's process, with no store process and no wire: ordered keys and
  values in memory, an append-only log and snapshots; transcripts beside
  it; secrets in a second store; one decision, one commit; a thin protocol
  layer that encodes keys so that bytes sort as the domain's keys do, and
  records with a version per shape.
- **jig's part is the engine's own:** the core's and its children's
  families, their keys and encodings, commits and loads, transcripts,
  secrets, opening and restart, growth, retention and fill. The
  application's part is its connectors' families and its root's
  wrapping, which it adds beside jig's in the same commits.
- **What the other components rely on** is stated in
  `../protocol/README.md`, section 9, and holds whatever the store's
  design settles.
- **Six questions** (section 5) are the design's to settle: who writes
  the store's component, transcripts, the real store in the worlds,
  reads and their indexes, retention and fill, and versions.

## 2. The base, mapped onto jig

| The base's section | In jig's store |
|---|---|
| skein-kv in the engine's process; the data directory | jig's, as is; `jig-*` crates |
| the boundary: commits, loads; transcripts' appends, drops and reads | against `../domain/engine.md`, 5.1 to 5.6 and 7.2, and the root's store vocabulary as built (section 3) |
| key encoding: byte order equal to the derived order; ranges to intervals | jig's, as is |
| families | the core's are jig's; the application's are its own, under one rule: a tag per root child, then the child's own keys, as the reference root's key already nests them |
| records: a codec per shape, versions, golden bytes | jig's; codecs from skein's generator (skein's `codec.md`), as the channel's are, or written out (section 5.6) |
| commits: numbered as the journal numbers them; group commit; a failure stops the engine; loads in pages | jig's, with skein-lib's journal numbering |
| transcripts: a file per session, written before their commit, read from either end, cut after a crash | jig's (section 5.2) |
| secrets in a second store | jig's: sign-ins' digests, services' credentials' digests; an application's connectors' credentials as its own records there. LLM accounts' records are skein's accounts driver's private files, not the store's |
| opening and restart | jig's, before the core's restart script (`../domain/engine.md`, section 6; `../protocol/service.md`, section 3) |
| growth, retention, fill | jig's mechanisms; the base's numbers become examples (section 5.5) |
| the real store in the domain worlds | section 5.3 |

## 3. jig's records as built

At the level of families and keys, on `main` as of this draft. Names move;
the store's design checks them against the code when it is written.

| Owner | Family | Key | Loaded |
|---|---|---|---|
| core | the deployment: its id and numbers, and the last commit | none: one row | at start |
| core | goals' projections: feed, plan rows, milestones pending | a goal's, then its own | at start |
| core | calls committed since a live task's last turn, with the core's part of their answers | task, attempt, completion, position | at start |
| core | run proofs: each live claim, its attempt, the first attempt of its conversation, its host's kind, the last turn and terminal accepted | task | at start |
| core | turns: one accepted turn, with its spend, the message it read, its time and its bytes | task, attempt, turn | to resume; on demand |
| core | terminals: an attempt's accepted answer | task, attempt | on demand |
| core | decisions of proposal and escalation races | proposal; task, revision | on demand |
| tasks | live tasks, stubs naming ended results, the funders' ledger (with a period's beginning and the next's), writer slots, pools' slots | task; funder; slot | at start |
| tasks | ended tasks, history, milestones, activity lines, parties' proposals | task, then a position or revision; proposal | on demand |
| people | parties, sign-ins (no secret), roles per project, projects' policies, read positions (results; per chat) | party; sign-in; project; party, task | at start |
| people | answered keys: jig's requests with their answers, and an application's with its digest and its connector's token, for the retention | party, key | at start, within the retention |
| notes | entries, and an index line per entry per scope | name; scope, name | per scope in use; entries on demand |
| an application's connector | its outbox, procedures' states, what it made by key, its projections' states, its outcomes kept for parties' requests, its own state (for `ops`, incidents with their leads) | its own | live ones at start; the rest on demand |

- **The root's vocabulary today:** commits of its wrapped writes; restart
  loads of a coarse range per child (the core's live rows, the inbox's,
  each connector's); and the core's own loads, each its own request: a
  party's read (a scope, a position, a direction, rows and bytes, with
  each row's owner task, its requester ancestry, related and linked
  records), notes' pages, a transcript from an attempt, historical rows
  by key.
- **Turns are records of the map today,** keyed by attempt, and a resumed
  run's conversation is read from the attempt its run proof names as its
  first.

## 4. What the other components rely on

`../protocol/README.md`, section 9: commits numbered, ordered and answered
once durable, a failure stopping the engine; loads by range in pages,
seeing every commit answered before them; transcripts appended with their
commit and read from either end; secrets' records written by components
alone, durable on request, read at start; opening before the root,
refusing another format, or records that disagree with their own header.
The client's protocol keeps a sign-in's digest under the number its commit
gave it, before handing the client its secret (`../protocol/README.md`,
section 8).

## 5. Questions to settle, with a leaning

### 5.1 Who writes the store's component

No traits or generics in step code, so jig cannot own a store machine that
holds an application's records typed. Either jig ships the component, and
the application's protocol layer hands it connector records already
encoded, with their keys' bytes; or jig ships codecs and key encodings,
and the application writes its store component from `ops`'s.

**Leaning:** jig ships it. The core's reads are queries (a scope joined
with each row's owner, ancestry and links; indexes walked), transcripts
need files and ordering behind syncs, and secrets need keeping before
use: these are jig's, and written once. The application's families
cross as encoded keys and values under its root's tag.

### 5.2 Transcripts

- **By session, with no domain change:** a run proof already names the
  first attempt of its task's conversation, so a session is a task and
  that attempt. A resumed run appends to it; a fresh one opens the next.
- **Map records, files, or skein-kv's payload logs** (skein's `kv.md`,
  section 6, designed and not built): records keep the map's memory
  growing with every turn; files per session are the base's choice, its
  protocol layer writing, syncing, cutting after a crash and reading from
  either end; payload logs move that into skein-kv, generic, with the
  extent committed in the map. **Leaning:** skein-kv's payload logs,
  asked of skein, with a frame readable from either end; files of jig's
  own if skein's are not built in time.
- **The domain's vocabulary** then gains appends and drops in a commit and
  reads from either end, as the base's boundary has them; a domain
  change (section 6).

### 5.3 The real store in the domain worlds

The base runs the real protocol layer over skein-kv's in-memory mode in
every world, with no fake of the map; jig's worlds use a domain fake
(`testing/jig-fake-store`) whose reads scan every row and join in each
world's own code. **Leaning:** move the worlds to the real store once it
exists, retiring the fake and the worlds' join code, which the store's
component replaces, so every story runs the real keys, codecs, paging
and joins. Transcripts' files, if files, have an in-memory stand-in.

### 5.4 Reads and their indexes

- **A party's reads page by scope:** updates for a party, their
  conversations, a project's ended goals, a task's ended delegates, a
  task's or a subtree's history, a task's activity from its end. The
  fake answers them by scanning; skein-kv has no secondary index.
- **Indexes are records,** saved and erased by their owners in the same
  decisions as what they point at, with keys reversed in time where a
  page reads newest first (skein's `kv.md`, 2.1). Which indexes, and who
  writes them, is a domain change (section 6).
- **Joins are point reads** in the same step: a row's owner task, its
  requester ancestry within the depth limit, its links.
- **Reading backwards:** a page from the end of a family (activity, a
  transcript's tail) needs skein-kv's `Load` in reverse, which it lists as
  later work (skein's `kv.md`, section 12), or keys reversed in time.

### 5.5 Retention and fill

The domain has neither yet: ended tasks shrinking to summaries past a
horizon, transcripts dropped past a longer one, answered keys expiring
past the people child's retention, ended chats forgotten (their record and
read position erased together, `../domain/people.md`, section 6), and
new work refused above a high-water mark of the store's fill. Each is a
mechanism the domain runs on a timer, tested in its worlds; who decides
when a chat is forgotten is the domain's (section 6).

### 5.6 Versions and their codecs

- **Codecs from skein's generator,** as the channel's: schemas per family,
  golden bytes, bounds, fuzzing; the domain's records translated to and
  from the generated ones with small total functions, so a domain change
  that touches a durable field breaks a translation's build. **Leaning:**
  generated.
- **A store written by another version.** Pre-release, a reader refuses
  another version and nothing is translated (skein's `codec.md`,
  section 5); a store of another format refuses to open. A deployment that
  must keep its data across a release needs an upgrade: old shapes read
  and upgraded as they load, or a pass at opening that rewrites them.
  Upgrades come later (`../protocol/README.md`, section 7): every record
  carries its version, and until then a store of another version refuses
  to open.

## 6. What the domain may owe

Each waits for the store's design, which decides it:

1. **Transcripts' vocabulary,** if transcripts leave the map: appends and
   drops in a commit, reads from either end (5.2).
2. **Indexes for paged reads,** as records their owners write (5.4).
3. **Retention:** summaries of ended tasks, transcripts dropped, answered
   keys expired, chats forgotten, each on the domain's timer (5.5).
4. **Fill:** the store's fill with each commit's answer, and new work
   refused above a high-water mark (5.5).
5. **Families ordered live first,** within each child, so a restart reads
   one interval per child (the tasks child's keys today put history
   first).
