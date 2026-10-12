# Session 15: the store's commits and loads

`jig-protocol-store` starts answering its boundary (jig 13's): it opens
over two skein-kv stores in memory, checking its format row and the
deployment's header against the store's last commit; it turns each of the
root's commits into one skein-kv commit of the same number, answered with
the store's fill, and fails on a refusal or a number out of step; it
serves the restart's steps from jig 13's interval lists in the core's
restore order, the inbox's census as the ended tasks its index rows name,
and each connector's live intervals as bytes; and it serves a child's
pages and single records under both bounds, answering `Corrupt` for a row
that does not decode. Everything here runs over skein-kv's memory mode
(`Store::memory`), which the domain worlds will use (jig 18); the stores
on a directory and transcripts' files are jig 17's.

## Read first

- jig's `protocol/store.md`, sections 1 to 3, 5.3, 6, 7.1, 7.2, 10, 12
  and 13.
- jig's `domain/engine.md`, sections 5.1, 5.3, 5.5 and 6.
- skein's `kv.md`, sections 2, 4 and 7.
- `decisions.md`, 7, 9 (fill's route; delegates' historical inputs), 13,
  20 and 24 (the boundary's `Open`, `OpenRefused`, `Corrupt`).
- `findings/store.md`, sections 1.1, 1.2, 1.4, 1.5, 2.6, 5 and 7.2 (S8,
  S9, S10).
- jig 13's file (`jig/13-store-keys.md`): its boundary sketch (the types
  this session answers), `keys` and `intervals`; jig 19's 19.5
  (`codec::encode`, `codec::decode`, `format::current`, `format::check`,
  `sizes::check`); jig 11's `RestoreRange` and `Event::Fill`; jig 12's
  12.3 (`RestoreRange::Census`).
- The code: skein's `crates/skein-kv/src/boundary.rs` (`Request`, `Op`,
  `Range`, `Page`, `Event`), `store.rs:162-186` (`memory`, `last`,
  `used`) and `:460-625` (`down`, `commit_memory`), `limits.rs`; jig's
  `crates/jig-domain-core/src/{restart.rs, stored.rs:155-240,
  numbers.rs:1-34, routing/vocabulary.rs}`;
  `crates/jig-domain-notes/src/boundary.rs:78-103` (`Key`, `Record`,
  `Range`); and what jig 13, 14 and 19 landed in
  `crates/jig-protocol-store/src/`, with its `src/tests/fixtures.rs`.

## What the design asks

- **Numbers are shared** (store.md 6): "Every root commit is one skein-kv
  commit, and nothing else writes the store, so the numbers are equal.
  The component checks this as it submits, and stops on a mismatch. At
  start, skein-kv's recovered last commit must equal the commit the
  deployment's header names".
- **Writes become operations in order** (6): a save a put of its encoded
  key and record, an erase an erase, the application's bytes under its
  tag (4.2).
- **Committed n** with the store's fill (3, 11.2); "A failure stops the
  engine" and "A refusal is a failure" (6).
- **The root's bound on commits in flight** "is checked at start against
  skein-kv's queue and queued bytes, and the component's queue behind
  transcripts' syncs, so the store never answers a commit busy" (6).
- **The format row** (5.3): written in the first commit with the
  header; at open, "a store whose row names another version of anything,
  or another deployment than its header, refuses to open, saying which".
- **Open** (10, step 2): the format row, then the header against
  skein-kv's last commit; an empty store gets its format row in the
  root's first commit.
- **The restart's ranges** (7.2): the core's live ranges in the order it
  restores them, "a fixed list of intervals, in that order, paged and
  handed up typed"; the census from the inbox's index; each connector's
  live ranges as the application's byte intervals; "what the core never
  restores ... is never read at start".
- **Pages** (7.1): answered in the step that asks; the cursor is
  exclusive; "skein-kv bounds a page by encoded bytes, the asker by rows
  and decoded bytes. The component keeps the longest prefix that fits
  both"; "A row that does not decode stops the engine, as a failed
  commit does".
- **The application's loads and gets** by byte intervals and keys,
  answered with bytes (12); jig's single records by key (3).

## Choices this plan makes

- **The component is made over two skein-kv stores its caller made**
  (`Stores`), and hands them back with `Store::into_stores`; jig 18's
  world keeps them across a crash (store.md 14). In this session the
  caller makes them with `skein_kv::Store::memory`.
- **`Open` is sent in memory mode too,** so that a world's restart runs
  store.md 10's checks (and jig 17's cut). jig 13's sketch says "the
  worlds' component needs none"; this plan sends it, its `root` unused
  until jig 17 gives it a meaning in memory mode (the transcripts' root).
- **The deployment's id comes from the store,** never from the
  component's maker: at a first start the shell draws it and the core
  commits it in the header (`configuration.md`, section 8), so the first
  commit's format row takes the id of the header that commit saves.
- **The component counts the next number itself:** the store's `last()`
  plus one at open, one more per commit it submits. A root commit with
  another number is answered `Failed { number }`.
- **After `Failed` or `Corrupt`, nothing more is answered** (jig 13's
  sketch, "What ends what"): the component keeps a stopped phase, never
  touches its stores again, and the routing stops the engine.
- **Fill** is skein-kv's `Store::used()` of the store (not the secrets'
  store), in each `Committed { number, fill }`; the routing hands it to the
  root, which sends jig 11's `core::Event::Fill { bytes }`.
- **The in-flight bound** joins 19.5's `sizes::check`: the journal's
  `commits` at most skein-kv's `queued`; `commits` times the largest
  commit at most `queued_bytes`; `commits` at most the component's queue
  behind transcripts' syncs (`Limits::queue`, jig 17). It is stricter than
  skein-kv's own ceiling (about twice `queued`, findings/store.md 5).
- **Restore and load pages** keep the longest prefix within the
  request's `rows` and `bytes` (decoded bytes) and skein-kv's
  `Limits::page` (encoded bytes); the routing asks for the next page with
  the cursor the component gave.
- **The census's rows are the ended tasks** its index rows name (jig 12's
  `Result` and conversation rows): each index row is joined by a get of
  `tasks::Key::Ended(task)`, and that record is handed up; an index row
  whose ended task is absent is `Corrupt`.
- **The four historical reads** the core asks as `Now` (findings/store.md
  1.5) are answered by the component's `Get`; a small pure module,
  `historical`, gives any application's routing the key to get and the
  core event to send back, so no application writes that translation.
- **Delegates' historical inputs are not built** (decision 9; `later.md`).

## Rules that bind it

- **Shared interfaces:** "the store's boundary (commit, restore, load,
  read, get, transcript, secrets; fill) | `jig-protocol-store`'s
  `boundary.rs` | jig 13". "One session defines each; every other
  session uses it by name and extends it only as its increment says."
  (code-organization.md, section 4.) This session answers jig 13's types
  as landed and adds no field to them.
- **Step code** "follows skein's `programming-model.md`, section 10, and
  jig's `AGENTS.md`: no traits, no generic items, no `dyn`, no closures."
  (code-organization.md, section 2.) The tests too: `Box`, arrays and
  `skein_lib::List`, never `Vec`, and no lint exception (jig 13's
  "Rules").
- **A durable record's codec follows its record:** "Once jig 14 or jig 19 has
  landed a family's codec, an increment of any session that changes a
  durable record of that family changes its schema, its generated codec
  and its translation in the same increment" (README, "Rules"). This
  session changes no record and encodes none outside jig 14's and jig
  19's codecs.

## Increments

### 15.1 Open over memory: the format row and the header

Scope: the component made over two memory stores, and its answer to
`Request::Open` (store.md 10, step 2; 5.3; 6), in the step that asks: it
gets the format row (`keys::Own::Format`) and the deployment's header
(`CoreKey::Deployment`) from the store; `format::check` against
`format::current(application.version, header.id)` refuses another
version of any family or another deployment, as `OpenRefused`; a header
whose `commits` differs from skein-kv's `last()` refuses as `Numbers`; an
empty store (no format row, no header, `last()` zero) answers `Opened {
last: 0 }` with its format row due in the first commit; a header without
a format row refuses as `Format { family: Family::Component, found: 0,
wanted }`, a format row without a header as `Numbers { store: last,
header: 0 }`.
Needs: jig 19's 19.5 (`format`, `codec::decode`).
Files: `crates/jig-protocol-store/src/{lib.rs, open.rs (new)}`;
`src/tests/{fixtures.rs, open.rs (new)}`.
Shape:
- `Stores { store: skein_kv::Store, secrets: skein_kv::Store }`;
  `Store::over(limits: &Limits, application: sizes::Application, stores:
  Stores) -> Store`; `Store::into_stores(self) -> Stores`.
- `open::open(store: &mut Store, owner: Token, above: &mut Queue<Event>)`,
  called by the crate's `down` for `Request::Open`.
- The component's phase: `Phase::{Closed, Ready { next: u64, format_due:
  bool }, Stopped}`.
Test shape: `src/tests/open.rs`. Fixtures (added to jig 13's
`src/tests/fixtures.rs`, reusing jig 13's `KEY_LIMIT`, 19.5's
`APPLICATION` and `DEPLOYMENT`, jig 14's and jig 19's record fixtures):
`OTHER_DEPLOYMENT: [u8; 16] = [9; 16]`; `ROOT_FD: Fd` (a descriptor the
memory mode never uses); `stores() -> Stores` (two
`skein_kv::Store::memory`); `plant(stores: &mut Stores, ops:
&[skein_kv::Op])` (one skein-kv commit beneath the component, for rows it
never wrote); `format_put(deployment) -> Op`, `header_put(deployment,
commits) -> Op` (each encoded with jig 13's keys and jig 19's codecs);
`over(stores) -> Store`; `run(store: &mut Store, request: Request) ->
Box<[Event]>` (one `down` step, the events above in order); `opened() ->
Store` (an empty store, `Open` answered). The reference test:

```rust
#[test]
fn a_store_of_another_deployment_refuses_to_open() {
    let mut stores = fixtures::stores();
    fixtures::plant(
        &mut stores,
        &[fixtures::format_put(fixtures::OTHER_DEPLOYMENT), fixtures::header_put(fixtures::DEPLOYMENT, 1)],
    );
    let mut store = fixtures::over(stores);
    let events = fixtures::run(&mut store, Request::Open { owner: Token::new(1), root: fixtures::ROOT_FD });
    let [Event::OpenRefused { owner, why: OpenRefusal::Deployment }] = &*events else {
        panic!("one refusal naming the deployment: {events:?}");
    };
    assert_eq!(*owner, Token::new(1));
}
```

Tests: `an_empty_store_opens_with_its_format_row_due`,
`a_store_of_another_deployment_refuses_to_open`,
`a_store_of_another_tasks_version_refuses_naming_tasks`,
`a_header_behind_the_last_commit_refuses_naming_both_numbers`,
`a_header_without_its_format_row_refuses_as_format`,
`a_header_equal_to_the_last_commit_opens_with_it`. Each under 20 ms.
Proof: unit, `open::a_store_of_another_deployment_refuses_to_open`, run
once with `heavy cargo nextest run -p jig-protocol-store -E
'test(a_store_of_another_deployment_refuses_to_open)'`: a store whose
format row names deployment `[9; 16]` beside a header of 19.5's
`DEPLOYMENT` is refused as `Deployment`; under 20 ms. No layer above
runs the component before jig 18's adapter.
Don't: read beyond the format row and the header at open here (the run
proofs and the cut are jig 17's); translate a row of another version
(store.md 5.3).
Size: at most 250 lines.
Removes: nothing.

### 15.2 Commits

Scope: store.md 6 and 11.2. Each `Request::Commit { number, writes }`
becomes one skein-kv `Commit` in the step that asks, its operations in
the writes' order: `Write::Save(record)` a put of
`keys::encode(&record.key())` and `codec::encode(&record)`; `Erase(key)`
an erase of the encoded key; `Put { key, value }` and `Remove { key }`
the application's, through `keys::application` (tag 2); the first commit
after an empty open also puts the format row,
`format::current(application.version, id)` with the id of the header
that commit saves (a first commit without the header is an `assert!`:
each writing decision saves it, `numbers.rs:3-5`). A number other than
the component's next is answered `Failed { number }`, as is a commit
skein-kv refuses (any `Refusal`). `Committed { number, fill }` once
skein-kv answers `Committed` with the same number. The in-flight bound
joins `sizes::check` (Choices).
Needs: nothing beyond 15.1.
Files: `crates/jig-protocol-store/src/{commits.rs (new), lib.rs,
sizes.rs (19.5's: three refusals)}`; `src/tests/{fixtures.rs, commits.rs
(new), format.rs (19.5's: the bound's tests)}`.
Shape:
- `commits::commit(store: &mut Store, number: u64, writes: Box<[Write]>,
  above: &mut Queue<Event>)`.
- `sizes::SizeRefusal` gains `Queue { commits: u32, queued: u32 }`,
  `QueuedBytes { needs: u64, has: u64 }`, `Transcripts { commits: u32,
  queue: u32 }`; `sizes::check` returns them after 19.5's own.
- Transitional, ended by jig 17: `Save(CoreRecord::Turn(..))` is put as a
  row under its `CoreKey::Turn` (jig 17's "frames, places, and the write
  before the commit" moves turns to files and removes this), and a commit
  holding `Write::DropTranscript` is answered `Failed` (jig 17's "after a
  crash, and dropped" replaces the arm; nothing produces one before
  retention).
Test shape: `src/tests/commits.rs`; fixtures: 15.1's; this increment
adds `commit(number, writes: Box<[Write]>) -> Request`, `header(commits)
-> Write` (the header for `DEPLOYMENT` saved), `save(record) -> Write`,
`get_kv(store: &Store, key: &[u8]) -> Option<Box<[u8]>>` (a skein-kv
`Get` beneath the component, for what reached the store). The reference
test:

```rust
#[test]
fn a_commit_numbered_past_the_next_fails() {
    let mut store = fixtures::opened();
    let events = fixtures::run(&mut store, fixtures::commit(2, Box::new([fixtures::header(2)])));
    let [Event::Failed { number: 2 }] = &*events else {
        panic!("commit 2 failed: {events:?}");
    };
}
```

Tests: `a_first_commit_puts_the_format_row_with_its_header`,
`a_save_puts_the_encoded_record_under_its_key`,
`an_erase_removes_the_key_from_the_store`,
`application_writes_land_under_the_application_tag`,
`a_committed_answer_carries_the_stores_used_bytes`,
`a_commit_numbered_past_the_next_fails`,
`a_refused_skein_commit_is_answered_failed`,
`nothing_is_answered_after_a_failure`,
`a_journal_bound_past_the_queue_is_refused_at_start`. Each under 20 ms.
Proof: unit,
`commits::a_first_commit_puts_the_format_row_with_its_header`, run once
with `heavy cargo nextest run -p jig-protocol-store -E
'test(a_first_commit_puts_the_format_row_with_its_header)'`: after
commit 1, skein-kv beneath the component holds the format row with the
header's id, and the component answered `Committed { number: 1, .. }`;
under 20 ms.
Don't: answer a commit busy or retry one; carry on after `Failed`.
Size: at most 400 lines.
Removes: nothing.

### 15.3 Restart loads

Scope: store.md 7.2 and 4.3. `Request::Restore { owner, range, after,
rows, bytes }` answered in the same step with one page, `Restored {
owner, rows, next }`:
- `Restore::Jig(ranges)`: for `RestoreRange::LIVE` (the header, people,
  the tasks child's live families, the core's live rows), the
  concatenation of `intervals::restore(&range)` in the list's order, paged
  and handed up typed (`Row::Jig`); a page that exhausts one interval with
  room left goes on into the next in the same step (4.3);
- `RestoreRange::Census(census)`: its intervals' index rows, each joined
  by a get of the ended task it names, handed up as that ended task
  (Choices);
- `Restore::Application(intervals)`: the application's intervals, each
  under tag 2, handed up as `Row::Application` with the tag taken off.
The cursor is jig 13's `Cursor { interval, after }` (exclusive: the next
page asks skein-kv from `after` followed by a zero byte, store.md 7.1);
`next: None` ends the step. A row that does not decode answers `Corrupt
{ owner }`.
Needs: jig 13's restart and census intervals (13.4, 13.5).
Files: `crates/jig-protocol-store/src/{restore.rs (new), pages.rs (new:
the interval walk 15.3 and 15.4 share), lib.rs}`; `src/tests/
{fixtures.rs, restore.rs (new)}`.
Shape:
- `pages::walk(store: &skein_kv::Store, intervals: &[skein_kv::Range],
  after: Option<&Cursor>, rows: u32, encoded: u32) -> Walked`; `Walked {
  rows: Box<[skein_kv::Row]>, next: Option<Cursor> }`; the decoded bound
  is applied by the caller, which decodes.
- `restore::restore(store: &mut Store, owner: Token, range: Restore,
  after: Option<Cursor>, rows: u32, bytes: u64, above: &mut
  Queue<Event>)`.
- jig 17's secrets sweep runs where the `People` range of
  `RestoreRange::LIVE` is exhausted; this increment names that point in a
  comment and nothing else.
Test shape: `src/tests/restore.rs`; fixtures: 15.1's and 15.2's; this
increment adds `committed(store: &mut Store, writes: Box<[Write]>)` (one
commit at the next number, asserting `Committed`), and the record
fixtures it needs where jig 14 and 19 did not add them:
`header_record(commits)`, `person(number)`, `live_task(number)`,
`ended_task(number)`, `history(task, revision)`, `run_proof(task)`,
`call(task, completion)`, `notes_entry(name)`, `result_index(party,
position, task)`, with named numbers `ALICE = 11`, `LIVE = 21`, `ENDED =
22`. The reference test:

```rust
#[test]
fn the_core_restore_hands_up_header_people_then_live_tasks() {
    let mut store = fixtures::opened();
    fixtures::committed(
        &mut store,
        Box::new([
            fixtures::header(1),
            fixtures::save(fixtures::live_task(LIVE)),
            fixtures::save(fixtures::person(ALICE)),
            fixtures::save(fixtures::ended_task(ENDED)),
        ]),
    );
    let range = Restore::Jig(Box::new(RestoreRange::LIVE));
    let request = Request::Restore { owner: Token::new(5), range, after: None, rows: 8, bytes: 65_536 };
    let events = fixtures::run(&mut store, request);
    let [Event::Restored { rows, next: None, .. }] = &*events else {
        panic!("one page: {events:?}");
    };
    let [Row::Jig(header), Row::Jig(person), Row::Jig(task)] = rows.as_ref() else {
        panic!("three live rows: {rows:?}");
    };
    assert_eq!(
        [header, person, task],
        [&fixtures::header_record(1), &fixtures::person(ALICE), &fixtures::live_task(LIVE)]
    );
}
```

Tests: `the_core_restore_hands_up_header_people_then_live_tasks`,
`run_proofs_follow_live_tasks_and_calls_follow_run_proofs`,
`ended_tasks_history_and_notes_are_never_read_at_start`,
`a_restore_page_goes_on_into_the_next_interval`,
`the_cursor_resumes_just_past_the_last_row`,
`the_census_hands_up_the_ended_task_its_row_names`,
`a_census_row_without_its_ended_task_is_corrupt`,
`a_connectors_interval_hands_up_rows_without_the_tag`,
`an_undecodable_restored_row_is_corrupt`. Each under 20 ms; the page
tests ask 2 rows.
Proof: unit,
`restore::ended_tasks_history_and_notes_are_never_read_at_start`, run
once with `heavy cargo nextest run -p jig-protocol-store -E
'test(ended_tasks_history_and_notes_are_never_read_at_start)'`: with an
ended task, a history row and a notes entry committed beside live rows,
the core's restore hands up the live rows only; under 20 ms.
Don't: sort rows in the component (the order is the interval list's and
the key order within each); call the core (the routing feeds the rows to
the root).
Size: at most 400 lines.
Removes: nothing.

### 15.4 Pages, child loads and gets

Scope: store.md 7.1, 3 and 12.
- `Request::Load { owner, range, after, rows, bytes }`:
  `Load::Notes(range)` over `intervals::notes(&range)` (an entry by key, or
  a scope's lines after the name given), answered `Loaded` with
  `Row::Jig(Record::Notes(..))` rows; `Load::Application(intervals)`,
  answered with `Row::Application` rows. Both bounds (Choices); the
  decoded bytes of a record are jig 19's measure of it, of the
  application's row its length.
- `Request::Get { owner, key }`: `Get::Jig(key)` answered `Got { row:
  Some(Row::Jig(..)) }` or none; `Get::Application(key)` answered with
  `Row::Application`.
- A row or value that does not decode: `Corrupt { owner }`.
- `historical::key(now: &core::Now) -> Option<core::Key>` and
  `historical::answer(now: core::Now, row: Option<core::Record>) ->
  Option<core::Event>` for `SeenChat` (`Tasks::Ended(task)`),
  `HistoricalPersonTask` (`Tasks::Ended(task)`), `HistoricalProposal`
  (`CoreKey::ProposalDecision`) and `HistoricalEscalation`
  (`CoreKey::EscalationDecision`), each answered with `busy: false`
  (gets are same-step); `None` for another `Now`, and a row of another
  family is answered as none.
Needs: jig 13's notes intervals (13.4).
Files: `crates/jig-protocol-store/src/{loads.rs (new), gets.rs (new),
historical.rs (new), pages.rs, lib.rs}`; `src/tests/{fixtures.rs,
pages.rs (new), gets.rs (new)}`.
Shape: `loads::load(store, owner, range: Load, after, rows, bytes,
above)`; `gets::get(store, owner, key: Get, above)`; the two `historical`
functions above, total over the four `Now` variants
(`crates/jig-domain-core/src/routing/vocabulary.rs`: `Now::SeenChat`,
`HistoricalProposal`, `HistoricalPersonTask`, `HistoricalEscalation`, and
their `Event` answers).
Test shape: `src/tests/pages.rs` and `src/tests/gets.rs`; fixtures as
above, plus `notes_line(scope, name) -> core::Record` and `PROJECT = 3`.
The reference test of the bounds (`pages.rs`):

```rust
#[test]
fn a_notes_page_stops_at_the_askers_rows_with_a_cursor() {
    let mut store = fixtures::opened();
    let scope = notes::Scope::Project { project: PROJECT };
    fixtures::committed(
        &mut store,
        Box::new([
            fixtures::header(1),
            fixtures::save(fixtures::notes_line(&scope, 1)),
            fixtures::save(fixtures::notes_line(&scope, 2)),
            fixtures::save(fixtures::notes_line(&scope, 3)),
        ]),
    );
    let range = Load::Notes(notes::Range::Lines { scope: scope.clone(), after: None });
    let request = Request::Load { owner: Token::new(6), range, after: None, rows: 2, bytes: 4096 };
    let events = fixtures::run(&mut store, request);
    let [Event::Loaded { rows, next: Some(_), .. }] = &*events else {
        panic!("one page with more: {events:?}");
    };
    let [Row::Jig(first), Row::Jig(second)] = rows.as_ref() else {
        panic!("two rows: {rows:?}");
    };
    assert_eq!([first, second], [&fixtures::notes_line(&scope, 1), &fixtures::notes_line(&scope, 2)]);
}
```

Tests (`pages.rs`): `a_notes_page_stops_at_the_askers_rows_with_a_cursor`,
`a_page_keeps_the_prefix_within_the_decoded_bytes`,
`a_page_keeps_the_prefix_within_the_encoded_bytes`,
`lines_after_a_name_start_just_past_it`,
`an_application_page_carries_its_cursor_and_no_tag`,
`an_undecodable_page_row_is_corrupt`; (`gets.rs`):
`a_get_answers_the_typed_record_by_its_key`,
`a_get_of_a_missing_key_answers_none`,
`an_application_get_answers_its_bytes`,
`a_seen_chat_is_answered_with_its_ended_task`,
`a_historical_escalation_is_answered_with_its_decision`. Each under
20 ms.
Proof: unit, `pages::a_page_keeps_the_prefix_within_the_decoded_bytes`,
run once with `heavy cargo nextest run -p jig-protocol-store -E
'test(a_page_keeps_the_prefix_within_the_decoded_bytes)'`: three rows
whose second passes the asker's decoded bound come back as the first
alone with a cursor naming it; under 20 ms.
Don't: add a load or a get for delegates' historical inputs
(`DelegateValidated.stubs`; decision 9); answer a get busy.
Size: at most 450 lines.
Removes: nothing.

## Interface sketches

jig 13's boundary (`Request`, `Write`, `Restore`, `Load`, `Get`,
`Cursor`, `Row`, `OpenRefusal`, `Family`, `Event`) is used as its file
sketches it, unchanged. What this session adds:

```rust
// jig_protocol_store (lib.rs): construction (this session).
pub struct Limits; // jig 13's
pub struct Store; // jig 13's component state
pub mod sizes {
    pub struct Application; // jig 19's (19.5)
}
/// The two skein-kv stores the component runs over, made by its caller.
pub struct Stores {
    pub store: skein_kv::Store,
    pub secrets: skein_kv::Store,
}
impl Store {
    /// The component over stores its caller made (memory here; a directory in jig 17).
    pub fn over(limits: &Limits, application: sizes::Application, stores: Stores) -> Store {
        todo!()
    }
    /// The stores back, for a world that keeps them across a crash (store.md 14).
    pub fn into_stores(self) -> Stores {
        todo!()
    }
}
```

```rust
// jig_protocol_store::historical: the core's four historical reads as gets (findings/store.md 1.5).
use jig_domain_core as core;
/// The key the core's historical read names; none for another `Now`.
pub fn key(now: &core::Now) -> Option<core::Key> {
    todo!()
}
/// The core event answering that read with the got row (busy: false); none for another `Now`.
pub fn answer(now: core::Now, row: Option<core::Record>) -> Option<core::Event> {
    todo!()
}
```

```rust
// jig_protocol_store::sizes: the in-flight bound joins 19.5's check (store.md 6).
/// 19.5's refusals, and the three this session adds.
pub enum SizeRefusal {
    // … 19.5's Key, Value, Ops, Commit, Page, Ceiling …
    /// The journal's commits in flight pass skein-kv's queue.
    Queue { commits: u32, queued: u32 },
    /// Those commits at their largest pass skein-kv's queued bytes.
    QueuedBytes { needs: u64, has: u64 },
    /// Those commits pass the component's queue behind transcripts' syncs.
    Transcripts { commits: u32, queue: u32 },
}
```

Who owns what: the component owns the stores it was made over until
`into_stores`; the routing (jig 18's for the test application, jig 27.3's
service) owns the translation of the root's requests, feeds restored rows
to the root, and stops the engine on `Failed` or `Corrupt`.

## Questions to raise when reached

- **The in-flight bound** (15.2): store.md 6 checks the journal's
  commits in flight against skein-kv's queue; this plan takes the
  journal's `commits` at most `queued`, though skein-kv can hold about
  twice that (one batch writing, one queued). If jig 20's profiles need
  more commits in flight than `queued` (at most 32,
  `skein_kv::Limits::is_usable`), ask before relaxing it.
- **A restored row the core refuses** (15.3): the component hands rows up
  and never judges them; the test root's `Now::RestoreRefused` is
  `unreachable!` today (`testing/jig-test-domain/src/routes.rs:513-517`).
  If neither jig 18's routing nor jig 27.3's service stops the start on it,
  raise it.

## Done when

1. The four increments are on `origin/main`: `git -C ~/src/rust/jig log
   origin/main --format=%b | grep -c '^Step: jig-15\.'` prints 4.
2. The component's tests of this session pass: `heavy cargo nextest run
   -p jig-protocol-store -E 'test(/tests::(open|commits|restore|pages|gets)::/)'`
   passes, running at least 30 tests.
3. No arm in the component is unreachable: `git -C ~/src/rust/jig grep
   -n 'unreachable!' -- crates/jig-protocol-store/src` prints nothing.
4. jig's gate passed on the last of them: `git -C ~/src/rust/jig log -1
   --format='%(trailers:key=Gate-Result,valueonly)' <its commit>` names
   `scripts/check.sh passed`.
