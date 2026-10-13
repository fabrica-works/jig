# Session 18: the store in the worlds

jig's own worlds move onto the real store, and the store gets a world of
its own. The test application ships its part of the store (its
connector's keys and codecs, its store version, its live intervals) and
the routing between its root's store vocabulary and the component; an
in-memory stand-in answers transcripts' files with the crashing disk's
rule; a world adapter runs `jig-protocol-store` over skein-kv's memory
mode and that stand-in behind the face the worlds already use, with the
faults held between the root and the component; the core world (and the
client's joint world riding on it) and the conformance world run on it,
`jig-fake-store` staying for `ops`'s worlds alone (decision 8); and
`tests/store` crashes the component on skein's simulated disk at each
file operation and sync, under the referee store.md 14 states.

## Read first

- jig's `protocol/store.md`, sections 3, 7.2, 8.4, 12, 14 and 15 (the
  application's part).
- jig's `domain/testing.md`, sections 4 to 7 (the worlds and their
  store); jig's `docs/development/workflow.md`, section 2 (the suites and
  their costs).
- skein's `kv.md`, section 9 (the machine's world and its referee);
  skein's `codec.md`, sections 3 to 6.
- `decisions.md`, 8 (`ops`'s worlds stay on `jig-fake-store`), 9 (the
  test application ships real encoders and codecs; the conformance world
  replays durable commits for snapshots and crashes; a story reads a
  historical escalation and a seen chat through the real store), 10.
- `findings/store.md`, sections 1.2, 1.5, 4 (the fake store and each
  world's wiring, with counts) and 7.3 (W0 to W4); `findings/protocol.md`,
  section 2.1 (the test application's root).
- jig 13's boundary sketch (`jig/13-store-keys.md`); jig 11's root
  `Event::Committed { number, fill }`; jig 12's 12.3 (`Core::census`,
  `RestoreRange::Census`); jig 06's `Now::StartPreparation { from }`;
  jig 14's codec world (`tests/store-codecs`); jig 15's and jig 17's
  constructors, `historical`, `Root` and io face; jig 27's 27.3 (the
  service takes this session's routing).
- The code: `testing/jig-test-connector/src/boundary.rs:43-226, 355-380,
  598-657` (the records, keys and their parts); `testing/jig-test-domain/
  src/lib.rs:23-86, 133-161, 1003-1056` (its store vocabulary, requests,
  `restore_record`) and `routes.rs:355-380, 513-517`;
  `testing/jig-fake-store/src/lib.rs` (the face the worlds use);
  `testing/jig-conformance/src/harness.rs:1-533`; `tests/core/src/
  {lib.rs:15-27, effects.rs:480-830, 995-1190, world.rs:470-600,
  reads.rs:120-409, historical.rs}`; `tests/conformance/src/lib.rs:140-240,
  370-380, 440-610`; `tests/ops/src/joint/engine.rs:40-145`; skein's
  `tests/kv/tests/store.rs:33-170, 207-236` (the crash world to copy) and
  `tests/kv/tests/fuzzy_map.rs` (the model); skein's
  `testing/skein-fake-machine/src/fs.rs:359-545` (`open`, `write`,
  `sync`, `crash`, `reopen_root`) with skein 03's `How::Write` and
  `truncate`.

## What the design asks

- **The real store in the domain worlds** (store.md 14): "The worlds run
  the component over skein-kv's in-memory mode, in place of a fake of the
  map, so every story runs the real keys, codecs, intervals, paging and
  joins. A fake of the store's own component retires once the component
  exists."
- **Faults come from the world** (14): "The world holds commits between
  the root and the component, in order: it delays them to make the store
  slow, and fails one to stop the engine. A crash keeps what reached the
  store and loses what the world held."
- **The files' stand-in** (14): "an in-memory file system answering the
  component's file requests, with the same rule after a crash. A crash
  keeps both in-memory stores and the stand-in, and the next engine's
  component is made over them and opens as section 10 says."
- **The store's world** (14): "the component over skein-kv and files on
  skein's simulated, crashing disk, a scripted root committing, loading,
  reading and appending turns, crashed at every file operation and sync.
  Its referee: after recovery, the last acknowledged commit, or that and
  one whole uncertain one, never part of one; no turn readable past its
  commit and none missing before it; numbers that match the header; pages
  equal to a model's."
- **The application's part** (12, 15): "its keys' encoders, its records'
  codecs, its store version, its live intervals, and its indexes";
  encoded "by the same rule" under its tag (4.1, 4.2).
- **Decision 9:** "the test application ships real key encoders and
  codecs for its connector's records"; "the conformance world rebuilds an
  in-memory store by replaying its durable commits, for snapshots and
  crashes: skein-kv gains nothing"; "a story reads a historical
  escalation and a seen chat through the real store".

## Choices this plan makes

- **Two new kits beside the stand-in code-organization.md names:**
  `testing/jig-test-store` (step code, so `jig-test-service` can take it
  as it is, jig 25): the test application's part of the store and its
  routing; `testing/jig-test-store-files` (ordinary Rust, as
  code-organization.md section 2 lists it): the files' stand-in and the
  worlds' adapter.
- **The stand-in is skein's fake machine without its simulator:**
  `Files` answers each `file::Request` the component sends by calling
  `skein_fake_machine::Machine`'s own functions (`open`, `read`, `write`,
  `sync`, `truncate`, `remove`, `make_directory`, `crash`), so its crash
  rule is the crashing disk's, and no second file model is written.
- **The adapter keeps `jig-fake-store`'s face** (`applied`, `rows`,
  `pending`, `held`, `fault`, `submit`, `tick`, `release_held`, `crash`,
  `loading`), so the worlds' 178 reads of `store.rows` and 132 of
  `store.applied` (findings/store.md 4.1) keep their meaning: `rows` is a
  typed mirror of the durable commits' writes, turns included, updated
  when the component answers `Committed`. It sits beside `jig-fake-store`
  by decision 8, not as a second fake: its rows come from the component.
- **Faults map as findings/store.md 4.3 says:** `Hold` holds the
  component's `Committed` before the root; a delay holds the commit before
  the component; `Fail { commit }` answers the root `Failed` without the
  component; `SlowPages { by }` holds the component's restore and load
  answers that many ticks.
- **A crash keeps the stores** (store.md 14): the adapter takes the two
  memory stores back with `into_stores`, crashes the stand-in, and makes
  the next component over them, which opens (and cuts) as store.md 10
  says. **A snapshot replays** (decision 9): `fork` makes a fresh
  component over fresh memory stores and a fresh stand-in, and replays the
  durable commits through them. Decision 9's "for snapshots and crashes"
  is read as the harness's `from_snapshot`, which forks and then crashes
  (Questions).
- **Fixture rows reach the store through the root,** as
  `World::configured` sends `people::Event::Roles` before the restart
  (`tests/core/src/effects.rs:187`), or, where a test needs rows the
  root does not make (record surgery, `tests/core/tests/restart.rs:115,
  159`), through `Store::seed`: one commit of its own at the next number,
  carrying the deployment's header with `commits` moved on, so the header
  still matches the store's last commit.
- **The harness drives either store** through a trait in
  `testing/jig-conformance` (ordinary Rust), `Durable`, implemented for
  `jig_fake_store::Store` (for `ops`) and for the adapter.
- **The fake read transport moves to `ops`:** `tests/core/src/reads.rs`'s
  scan (`owner`, `row`, `candidate`, `page`) is used after W2 by
  `tests/ops/src/joint/engine.rs:48-142` alone, so it moves there, with
  the `jig_core_world::Store` it reads becoming
  `jig_fake_store::Store<root::Key, root::Record>`.
- **This session's share of the default suite:** at most 2.5 s of the
  gate's focused wall time, of the plan's 6 s (code-organization.md 5):
  `tests/store` at most 1 s, the domain worlds' move at most 1.5 s.

## Rules that bind it

- **"Worlds keep their layout: `tests/<world>/src/` for the world, its
  focused tests in `tests/<world>/tests/*.rs`, its fuzzy ones in
  `tests/<world>/tests/fuzzy_*.rs`"** and "A new world costs at most
  about 1 s focused and 5 s fuzzy, in all." "The budgets bind the gate's
  wall time: 15 s focused, 60 s fuzzy" (code-organization.md, section 5).
  A test above twice its ceiling is made cheaper before it lands; tests
  are never merged to fit (skein's development.md, 8.2).
- **"No increment of this plan ... adds a type beside an existing one
  for the same thing: it changes the type in place, and its users in the
  same increment; ... adds a lint exception in tests; ... computes an
  expected value with the code under test"** (code-organization.md,
  section 6). The adapter stands beside `jig-fake-store` by decision 8
  only.
- **"A durable record's codec follows its record"** (README, "Rules"):
  from 18.1 on, a change to a test connector's `Record` changes its
  schema, generated codec and translation in the same increment.

## Increments

### 18.1 The test application's keys and codecs (W0)

Scope: store.md 12 and 15, decision 9. `testing/jig-test-store`, step
code: the connector's `RecordKey` encoder and decoder by store.md 4.1's
rule (the connector's number as `u64`, then the variant's index as a tag,
then its fields; `Path` as a list of byte strings; the connector's `Key`
field by field, its `deployment` as bytes), each held to the derived
`Ord` by a property test; the 12 records' schema, `test-connector` family
version 1, generated, with total translations both ways (decoding is
fallible, never panicking); `STORE_VERSION: u32 = 1`; `live(connector:
u16) -> skein_kv::Range`, the connector's one live interval (each of
its records is restored at its step, `testing/jig-test-connector/src/
domain.rs:466-501`).
Needs: jig 13's key rule (`KeyWriter` widening, tags) and jig 14's codec
layout.
Files: `testing/jig-test-store/{Cargo.toml (new), schema/test-connector-v1.schema
(new), golden/v1/ (generated), src/{lib.rs, keys.rs, records.rs,
generated/v1.rs (generated), tests.rs, tests/{fixtures.rs, keys.rs,
records.rs}}}`; the workspace `Cargo.toml` (member and dependency rows:
shared hotspot); `tests/boundary/src/tests.rs` (the new crate's rules:
step code, links no smith, no `jig-protocol-*` but `jig-protocol-store`'s
boundary in 18.2; shared hotspot); `tests/store-codecs/tests/{drift.rs,
fuzzy_decode.rs}` (jig 14's codec world: the test connector's schema and
decoder beside jig's).
Shape:
- `keys::encode(number: u16, key: &connector::RecordKey) -> Box<[u8]>`;
  `keys::decode(bytes: &[u8], limits: &connector::Limits) ->
  Option<(u16, connector::RecordKey)>`.
- `records::encode(record: &connector::Record) -> Box<[u8]>`;
  `records::decode(bytes: &[u8], limits: &connector::Limits) ->
  Result<connector::Record, Undecodable>`.
- `live(connector: u16) -> skein_kv::Range`; `STORE_VERSION`;
  `application(limits: &connector::Limits) -> jig_protocol_store::sizes::Application`
  (its version, and its largest key and value from the generated codec's
  worst case, for 19.5's `sizes::check`).
Test shape: `testing/jig-test-store/src/tests/{keys.rs, records.rs}`;
fixtures in `src/tests/fixtures.rs` (new): one record of each of the 12
kinds, built from `jig_test_connector_world::path(1, 1)`
(`tests/test-connector`) where a `Path` is needed, `FIRST = 1`, `SECOND
= 2`, and a seeded generator of `RecordKey`s (`skein_lib::Rng`). The
reference test of the key order:

```rust
#[test]
fn encoded_keys_sort_as_the_derived_order() {
    let mut random = skein_lib::Rng::new(18);
    let mut previous = fixtures::record_key(&mut random);
    for _ in 0..500 {
        let next = fixtures::record_key(&mut random);
        let by_bytes = keys::encode(previous.0, &previous.1).cmp(&keys::encode(next.0, &next.1));
        assert_eq!(by_bytes, previous.cmp(&next), "{previous:?} against {next:?}");
        previous = next;
    }
}
```

(The crate is step code: its tests keep the subset, with no lint
exception, code-organization.md 6.)

Tests (`keys.rs`): `encoded_keys_sort_as_the_derived_order`,
`a_decoded_key_is_the_key_encoded`,
`a_shorter_path_sorts_before_its_extensions`,
`each_record_lies_within_its_connectors_live_interval`;
(`records.rs`): `each_record_kind_decodes_to_itself`,
`a_truncated_record_fails_to_decode`,
`a_path_past_its_limit_fails_to_decode`. Each under 20 ms; the order test
draws 500 keys.
Proof: unit, `keys::encoded_keys_sort_as_the_derived_order`, run once
with `heavy cargo nextest run -p jig-test-store -E
'test(encoded_keys_sort_as_the_derived_order)'`: 500 drawn keys of both
connectors sort the same as bytes and as values; under 20 ms. The joint
layer is 18.3's.
Don't: encode a connector record in `jig-test-domain` or the connector
(code-organization.md 6: no bytes in a domain crate).
Size: at most 500 lines (generated code excluded).
Removes: nothing.

### 18.2 The routing, the files' stand-in, and the worlds' adapter (W1)

Scope: store.md 14, decision 9.
- **The routing** (`jig_test_store::route`, step code): the test root's
  store requests to jig 13's boundary, and its answers to the root's
  events:
  - a commit: `root::Write::Save(Record::Core(r))` to `Write::Save(r)`,
    `Erase(Key::Core(k))` to `Write::Erase(k)`, a connector's record to
    `Write::Put` (18.1's key and value) and its erase to `Write::Remove`;
    `Committed { number, fill }` to the root's `Event::Committed { number,
    fill }` (jig 11, which sends `core::Event::Fill`); `Failed` to
    `Event::Failed`;
  - the restart: `RestartStep::LoadCore` to `Restore::Jig(RestoreRange::LIVE)`,
    `LoadInbox` to `Restore::Jig([RestoreRange::Census(census)])` with the
    census the root takes from `Core::census` (jig 12.3), `RestoreConnector
    { connector }` to `Restore::Application([live(connector)])`; each
    page's rows to records for `restore_record` (a connector's decoded by
    18.1 into `Record::Connector`), the next page asked with the cursor,
    then `people::Event::Restored` after `LoadCore` (as the worlds send it
    today, `tests/core/src/effects.rs:1116-1118`) and `RestartDone(step)`;
  - `Held::ReadLoad { load, range }` to `Read`, its answer to
    `Event::ReadLoaded { load, result }`; `Held::NotesLoad` to
    `Load::Notes`, its `Loaded` to `notes::Event::Loaded { rows, more:
    next.is_some() }`; the four historical `Now`s through jig 15's
    `historical`; `Now::StartPreparation { from: Some((attempt, turn)), .. }`
    (jig 06) to a `Window` read, its `Turns` to `Event::TranscriptLoaded`
    pages, `done` with the last;
  - `Corrupt` to the routing's stop, as `Failed` stops the root.
- **The files' stand-in** (`jig_test_store_files::Files`): Choices.
- **The adapter** (`jig_test_store_files::Store`): the component
  (`Store::over`, jig 15) over two memory stores, opened with `Files`'s
  root (jig 17's `Root`), with the face and faults of Choices; `crash`
  keeps the stores; `fork` replays; `seed` commits test rows with the
  header moved on.
Needs: nothing beyond 18.1.
Files: `testing/jig-test-store/src/{route.rs (new), lib.rs}`,
`src/tests/{fixtures.rs, routes.rs (new)}`;
`testing/jig-test-store-files/{Cargo.toml (new), clippy.toml (new, as
`testing/jig-test-system/clippy.toml`), src/{lib.rs, files.rs, store.rs,
tests.rs}}`; `tests/core/{Cargo.toml, tests/store_adapter.rs (new)}`
(the adapter's tests live with its first user, which has the root's
fixtures); the workspace `Cargo.toml`; `tests/boundary/src/tests.rs`.
Shared hotspots: the workspace `Cargo.toml`, `tests/boundary`.
Shape:
- `route::Routes::new(table: u32) -> Routes` (a bounded table of what
  each owner token asked); `route::down(routes: &mut Routes, out:
  StoreOut) -> Option<jig_protocol_store::Request>`; `route::up(routes:
  &mut Routes, event: jig_protocol_store::Event, into: &mut Queue<Up>)`;
  `StoreOut::{Commit { number, writes: Box<[root::Write]> },
  Restart { step: core::RestartStep, census: Option<core::Census> },
  ReadLoad { load, range }, NotesLoad { owner, range }, Now(core::Now)}`; `Up::{Restore(root::Record), Root(root::Event),
  Ask(jig_protocol_store::Request)}`.
- `Files::new() -> Files`; `Files::root(&mut self) -> Root`;
  `Files::answer(&mut self, request: file::Request) -> Arrived`;
  `Files::crash(&mut self, seed: u64)`.
- `Store` and `Fault` as sketched below; `Store::new(limits: &root::Limits)`
  derives the component's limits from the root's with jig 19's sizes (19.5),
  with test-sized budgets (a map of 4 MiB per store, 8 open files, a queue
  of 8 commits). `Store`'s `Debug` prints its face alone (`applied`,
  `rows`, `pending`, `held`): the conformance world compares a fork's
  outside state with a fresh run's by it
  (`tests/conformance/src/replay.rs:189-200`), and the component's own
  state differs between the two.
Test shape: three test files.
- `testing/jig-test-store/src/tests/routes.rs`: step tests of the
  routing over a component in memory; fixtures in `src/tests/fixtures.rs`
  (18.1's, plus the component's limits as a literal and a `component()`
  over two memory stores).
- `testing/jig-test-store-files/src/tests.rs`: the stand-in alone, over
  `file::Request` values written in the test.
- `tests/core/tests/store_adapter.rs`: the adapter, reusing
  `jig_core_world::world::limits()` (`tests/core/src/world.rs:16`);
  helpers in the file: `first_commit()` (the header and
  `people::Stored::Roles { project: 1, holdings: [] }`), `turn_commit(task,
  turn, bytes)` (the header, the turn, its run proof), `window_turns(store,
  task)` (the task's window read through `ask` and `answers`), `LIVE = 21`.
  The reference test:

```rust
#[test]
fn a_crash_keeps_a_durable_turn_and_loses_a_held_one() {
    let mut store = Store::new(&jig_core_world::world::limits());
    store.submit(1, first_commit(), 0);
    assert_eq!(store.tick(false), Ok(Some(1)));
    store.submit(2, turn_commit(LIVE, 1, b"kept"), 0);
    assert_eq!(store.tick(false), Ok(Some(2)));
    store.submit(3, turn_commit(LIVE, 2, b"held by the world"), 5);
    store.crash(7);
    assert_eq!(store.applied, 2);
    assert_eq!(window_turns(&mut store, LIVE), [b"kept".as_slice()]);
}
```

Tests (`routes.rs`): `a_commit_routes_connector_records_as_bytes`,
`load_core_pages_reach_the_root_then_people_restored`,
`a_connectors_step_restores_its_records`,
`a_seen_chat_is_answered_from_the_store`,
`a_window_read_reaches_the_root_as_transcript_pages`;
(`tests.rs`): `a_synced_write_survives_a_crash`,
`an_unsynced_cut_keeps_the_old_length_after_a_crash`,
`a_removed_missing_file_answers_not_found`;
(`store_adapter.rs`): `a_crash_keeps_a_durable_turn_and_loses_a_held_one`,
`a_held_commit_is_durable_but_unacknowledged`,
`a_failed_commit_never_reaches_the_component`,
`slow_pages_hold_a_restore_answer_for_their_ticks`,
`a_fork_replays_to_the_same_rows`,
`a_seeded_commit_keeps_the_header_in_step`. Unit tests under 20 ms; the
adapter's under 50 ms each.
Proof: world,
`store_adapter::a_crash_keeps_a_durable_turn_and_loses_a_held_one`, run
once with `heavy cargo nextest run -p jig-core-world --test
store_adapter -E
'test(a_crash_keeps_a_durable_turn_and_loses_a_held_one)'`: the real
component, routing and stand-in keep the durable turn and lose the one
the world held, across a crash; under 50 ms. The domain worlds run on it
in 18.3.
Don't: let the adapter write `rows` except from a `Committed`; give the
component a file answer the stand-in did not compute.
Size: at most 300 lines of production code (`route.rs`); the kits' code
is test code, at most 800 lines per file.
Removes: nothing.

### 18.3 The core world on the real store (W2)

Scope: store.md 14; decision 9. `tests/core`'s `effects::World` and
`world::World` run on the adapter: `crate::Store` names it
(`tests/core/src/lib.rs:15`), `store_writes` goes (the adapter takes the
root's writes); the answer paths move to the routing: the restart's
(`effects.rs:1003-1120`, with the rank sort), the transcripts'
(`effects.rs:507-526`), the historical reads' (`effects.rs:547-594`,
`world.rs:504-528`, `historical.rs` removed), the notes' loads
(`effects.rs:796-820`, `world.rs:569-595`) and the parties' reads
(`reads.rs:126-143` keeps its admissions; its scan moves to `ops`,
Choices); fixture rows go through the root or `seed`; the fault tests
(`tests/core/tests/store.rs`) run unchanged in meaning on the adapter
(the fake's scenarios against the real side, development.md 5.3). The
test root routes `Now::SeenChat` and `Now::HistoricalEscalation` out as
it routes the other two historical reads
(`testing/jig-test-domain/src/routes.rs:513-514`, today `unreachable!`),
and two stories read them through the store (decision 9). jig 03.2 routes
`Now::HistoricalEscalation` early, to the world's `historical.rs`
(ruling 107): 18.3 moves it to the store with the other historical
reads. The client's
joint world (`tests/client/src/joint/*`) rides `effects::World` and
moves with it, unchanged but for types.
Needs: nothing beyond 18.2.
Files: `tests/core/{Cargo.toml, src/{lib.rs, effects.rs, world.rs,
reads.rs, historical.rs (removed), observations.rs:534-540,
application_observations.rs, store_peer.rs, stories/reads.rs:153}}`;
`tests/core/tests/{application_documents.rs, client_peer.rs,
client_peer_routes.rs, faults.rs, fuzzy_faults.rs, fuzzy_reads.rs,
fuzzy_stories.rs, instructions.rs, peers.rs, reads.rs, restart.rs,
store.rs, store_peer.rs, stories.rs, stories/address.rs,
stories/procedure_goals.rs}` (the 25 files naming `jig_fake_store`,
findings/store.md 4.1); `tests/client/src/joint/{engine.rs, front.rs,
session.rs, truth.rs, random.rs}` where types change;
`tests/ops/src/joint/{engine.rs, reads.rs (new: the moved scan)}`;
`testing/jig-test-domain/src/routes.rs:513-514`. Shared hotspots:
`tests/core/src/effects.rs` (1 289 lines: this increment shrinks it;
split if it stays past 800), `testing/jig-test-domain/src/routes.rs`.
Shape: no new type; `crate::Store = jig_test_store_files::Store`;
`crate::Fault = jig_test_store_files::Fault`; the world's store paths call
`self.store.ask(StoreOut)` and take `self.store.answers()` each iteration.
Test shape: existing tests keep their names and expectations; two new
tests in `tests/core/tests/store.rs`, reusing `World::new`
(`effects.rs:161`), `turn` (`:1215`), `end` (`:1228`) and `restart`
(`:1150`); the escalation's story reaches `Now::HistoricalEscalation` as
`crates/jig-domain-core/src/routing/party_decisions.rs:438-477` does (a
decision on a hold its task no longer holds). The reference test:

```rust
#[test]
fn a_seen_chat_is_read_back_through_the_store_after_restart() {
    let mut world = World::new(127, false);
    let task = world.assigned.as_ref().expect("the world's chat").task;
    world.turn(1, 3, None, b"the chat's reply");
    world.end(
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"done".as_slice().into() },
            cancel_delegates: false,
        },
        3,
    );
    world.restart(127, false);
    let seen = people::Seen::Chat { task, through: 1 };
    jig_core_world::stories::reads::request(&mut world, 62, people::Ask::MarkSeen { seen });
    assert_eq!(world.people_answers.last(), Some(&people::Reply::Outcome(people::Outcome::MarkedSeen { seen })));
}
```

(An ended chat is not live, so the core asks `Now::SeenChat`,
`crates/jig-domain-core/src/routing/people.rs:261-279`; the helper
`request` is `tests/core/src/stories/reads.rs:43`. If the world's chat
ends otherwise, take the ending `tests/core/src/stories/closing_chat.rs`
uses.) Tests: `a_seen_chat_is_read_back_through_the_store_after_restart`,
`a_historical_escalation_is_read_through_the_store`. Each under 0.5 s.
Proof: joint,
`store::a_crash_after_durability_before_acknowledgement_restores_the_turn`,
run once with `heavy cargo nextest run -p jig-core-world --test store -E
'test(a_crash_after_durability_before_acknowledgement_restores_the_turn)'`
(`tests/core/tests/store.rs:116`), now on the real component: the turn
kept in its transcript's file survives the crash and is readable after
the restart; under 0.5 s.
Don't: add a test to reach a count; change an existing test's expected
values to fit the real store. A test that fails on the real store is a
finding: fix it in the component (in this increment, within 20 lines,
development.md 8.2) or raise it.
Size: no production code beyond `routes.rs:513-514`; test code: the
world shrinks.
Removes: `tests/core/src/historical.rs`; the fake scan in
`tests/core/src/reads.rs` (moved to `ops`); `store_writes`; the core
world's restart rank sort (`effects.rs:1075-1120`); `jig-fake-store` from
`tests/core/Cargo.toml`.


**18.3 takes what its gate needs of 18.4** (decisions.md, ruling 236).
`tests/conformance` imports the core world's `Store` and `store_writes`,
which 18.3 replaces and removes, and drives `Application` methods that
still take the fake store, so 18.3's gate cannot build. 18.3 also makes
18.4's `Durable` trait and `Application` changes, and the conformance
world's routing to the adapter, as 18.4's Scope, Shape and Files say,
named in its commit body. 18.4 keeps its new test
(`a_forked_snapshot_replays_to_the_durable_rows`), its joint proof and
the measure, and whatever of its Scope 18.3 did not need.

### 18.4 The conformance world on the real store; the budgets (W3)

Scope: decisions 8 and 9. `testing/jig-conformance`'s harness drives a
`Durable` store: the `Application` trait gains `type Store: Durable` and
`type Write`, its functions take `&Self::Store`, and `Output::Commit`
carries `Box<[Self::Write]>`; `cold_snapshot` keeps `Durable::fork`'s
store. `tests/conformance` runs on the adapter: its restart sort
(`lib.rs:146-233`), transcripts (`:461-475`), notes (`:554-576`) and pages
(`:600-610`) go to the routing; its fixture row (`:373-376`) goes through
the root. `tests/ops`'s two `Application` impls name
`jig_fake_store::Store` as their store and keep it. The suites are
measured once (Proof).
Needs: nothing beyond 18.3.
Files: `testing/jig-conformance/{Cargo.toml, src/{harness.rs, lib.rs}}`;
`testing/jig-test-store-files/src/{durable.rs (new), lib.rs}` (the
`Durable` impl, beside its type); `tests/conformance/{Cargo.toml, src/
{lib.rs, negative.rs, replay.rs, actions.rs}, tests/{faults.rs,
memory.rs, simulation.rs, referee.rs, trace.rs}}`;
`tests/ops/src/{conformance/mod.rs:140-200, joint/engine.rs:144-160}`
and the ops files that name `Output::Commit`.
Shape: the `Durable` trait and the `Application` changes, as sketched
below; `impl<K: Ord + Clone + Debug, R: Clone + Debug> Durable for
jig_fake_store::Store<K, R>` in `testing/jig-conformance`.
Test shape: existing tests keep their names and expectations; one new
test in `tests/conformance/tests/snapshot.rs`, reusing `replay::fresh`
(`tests/conformance/src/replay.rs:166`):

```rust
#[test]
fn a_forked_snapshot_replays_to_the_durable_rows() {
    let world = replay::fresh(Scenario::LateCopy, Cut::None, 81, false).expect("fresh story");
    let rows = world.store.rows.clone();
    let forked = jig_conformance::Harness::<Testing>::from_snapshot(world.cold_snapshot()).expect("fork restarts");
    assert_eq!(forked.store.rows, rows);
}
```

Ceilings: unchanged, each focused test under 0.5 s and fuzzy test about
5 s.
Proof: joint, `fuzzy_snapshot::late_copy_every_snapshot_matches_complete_fresh_state`
(`tests/conformance/tests/fuzzy_snapshot.rs:27`) on the real store, under
5 s; and the measure, run once after this increment's change:
`heavy --gate cargo nextest run -p jig-core-world -p
jig-conformance-world -p jig-client-world --profile measure -j 1` and the
same with `--ignore-default-filter -E 'binary(/^fuzzy_/)'`, their totals
and slowest tests written in the commit body beside findings/store.md
4.4's of 2026-10-11 (core 1.79 s / 1.65 s, conformance 1.56 s / 4.45 s).
Nothing else is measured. Guide: aim for the gate's focused wall time, as
`land`'s gate log prints it, at most 1.5 s above 9.0 s; if it is past,
profile the slowest and make them cheaper before landing (development.md
5.3).
Don't: move `ops`'s worlds off `jig-fake-store` (decision 8); raise a
suite's budget (workflow.md, section 2).
Size: no production code; test code within the file limits.
Removes: `jig-fake-store` from `tests/conformance/Cargo.toml`; the
conformance world's restart sort and its own page, transcript and notes
answers.

### 18.s1 A made store directory is synced into its parent

A step not in the plan (decisions.md, ruling 241): a durability defect
jig 18.5 found on the real store. `crates/jig-protocol-store/src/directory.rs`
makes `store/` and `transcripts/` under the deployment's directory
(`MakeDirectory`, :64 and :144) and never syncs that directory, so a crash
after the first acknowledged commit loses their entries and the commit
with them (18.5's crash tests at cut 0; `logs/jig-18/focused-18-5-3.log`).
store.md, 14 promises "the last acknowledged commit, or that and one
whole uncertain one".
Needs: nothing; before 18.5 lands.
Files: `crates/jig-protocol-store/src/directory.rs`,
`src/tests/directory.rs`; `tests/store` (the store's world), one row.
Shape: after it makes a child directory, and before it opens a store in
it, the component syncs the deployment's directory
(`file::Request::SyncDirectory`), as `transcripts/drop.rs:205` syncs
after a removal; a failed sync refuses the open, as a failed make does.
Test shape: a row in the directory unit tests that each `MakeDirectory`
is followed by the parent's `SyncDirectory` before the store opens; a
row in the store's world that starts from an empty deployment directory
and, crashed at every file operation after its first acknowledged commit,
keeps that commit.
Proof: 18.5's crash tests at cut 0 pass on the real store; the gate.
Don't: pre-create the directories in a fixture; change the crashing
machine's rules.
Size: at most 40 lines of production code.

### 18.s2 The private secrets directory is synced into its parent

A step not in the plan (decisions.md, ruling 245), the second of 18.5's
durability findings after 18.s1. The component opens the secrets' store
through skein's `OpenPrivate`, which makes `secrets/` when it is absent
(skein `io.md`, 5.3) and leaves its entry's durability to the caller
(`SyncDirectory`, io.md 5). `crates/jig-protocol-store/src/directory.rs`
opens the secrets' kv store without syncing the deployment's directory
after it, so a crash after `Kept` loses the secret (18.5's
`a_secret_kept_survives_a_crash`, seed 18; `logs/jig-18/focused-18-5-after-s1.log`).
Needs: 18.s1; before 18.5 lands.
Files: `crates/jig-protocol-store/src/directory.rs`,
`src/tests/directory.rs`.
Shape: after `OpenPrivate` answers `Opened`, and before the secrets' kv
store opens, the component syncs the deployment's directory, keeping the
private root it was given; a failed sync refuses the open, as 18.s1's
does.
Test shape: a row in 18.s1's directory table: `OpenPrivate` is followed
by the parent's `SyncDirectory` before the secrets' store opens. 18.5's
crash test is the world's proof.
Proof: 18.5's `a_secret_kept_survives_a_crash` passes on the real store;
the gate.
Don't: pre-create `secrets/` in a fixture; change skein.
Size: at most 25 lines of production code.

### 18.5 The store's world on the crashing disk (W4)

Scope: store.md 14 and 8.4. `tests/store` (package `jig-store-world`),
ordinary Rust: the component on a directory of skein's fake machine
through three `FileIo` drivers and `skein_sim::Sim`, as skein's kv world
does (`tests/kv/tests/store.rs:33-170`); a scripted root that commits
(the header with each commit, live tasks, run proofs, turns), restores,
loads, reads and reads turns back; crashes at each file operation and
sync of a commit with a turn, and of a commit without one; the referee:
after recovery, the store holds the last acknowledged commit, or that and
the one uncertain commit, never part of one; no turn readable past its
commit and none missing before it; the header's `commits` equal to the
store's last; each page equal to a `BTreeMap` model's (as
`tests/kv/tests/fuzzy_map.rs` models skein-kv). jig 14's codec world
stays `tests/store-codecs`; this is a world of its own.
Needs: nothing beyond the session's start: it uses none of 18.1 to
18.4 (skein 03's `How::Write` and `truncate` in the fake machine come with
jig 02).
Files: `tests/store/{Cargo.toml, clippy.toml, src/{lib.rs, world.rs,
root.rs, referee.rs, model.rs}, tests/{crashes.rs, pages.rs,
fuzzy_crashes.rs}}`; the workspace `Cargo.toml` (`skein-fake-machine`,
`skein-sim` rows: shared hotspot).
Shape: `World::new(seed) -> World`; `World::commit(&mut self, writes) ->
Option<u64>` (the number, if acknowledged before the cut); `World::cut_at(&mut
self, operation: u32)`; `World::crash(&mut self, seed: u64)`;
`World::reopen(&mut self) -> Restored` (open, then the core's restore
and each task's window read); `Referee::check(&Model, &Restored) ->
Result<(), Violation>`.
Test shape: `tests/store/tests/crashes.rs`, `pages.rs`,
`fuzzy_crashes.rs`; fixtures in `tests/store/src/root.rs` (the scripted
root's records and commits: `header(commits)`, `live_task(LIVE)`,
`run_proof(LIVE)`, `turn(LIVE, attempt, number, bytes)`, `first()` (commit
1: the header, the live task, its run proof), `turn_commit(number, task,
turn, bytes)` (the header and the turn), `LIVE = 21`). The reference
test:

```rust
#[test]
fn a_crash_at_each_file_operation_of_a_turn_keeps_a_whole_prefix() {
    for cut in 0..12_u32 {
        let mut world = World::new(u64::from(cut));
        world.commit(root::first());
        world.cut_at(cut);
        let acknowledged = world.commit(root::turn_commit(2, LIVE, 1, b"one turn"));
        world.crash(u64::from(cut));
        let restored = world.reopen();
        assert!(restored.last == 1 || restored.last == 2, "cut {cut}: a whole prefix");
        if acknowledged.is_some() {
            assert_eq!(restored.last, 2, "cut {cut}: an acknowledged commit survives");
        }
        assert_eq!(restored.turns(LIVE).len(), usize::try_from(restored.last - 1).expect("fits"), "cut {cut}");
    }
}
```

Tests (`crashes.rs`): `a_crash_at_each_file_operation_of_a_turn_keeps_a_whole_prefix`,
`a_crash_at_each_operation_of_a_plain_commit_keeps_a_whole_prefix`,
`a_frame_past_its_commit_is_cut_at_open`,
`the_header_matches_the_store_after_each_crash`,
`a_dropped_transcript_stays_dropped_after_a_crash`;
(`pages.rs`): `restored_pages_equal_the_models_rows`,
`a_secret_kept_survives_a_crash`; (`fuzzy_crashes.rs`):
`random_commits_and_crashes_keep_the_referees_promises` (64 seeds).
Ceilings: each focused test under 0.5 s, the world's focused tests at
most 1 s together, its fuzzy test about 5 s, unoptimized, read once from
nextest's times in the last focused run.
Proof: world,
`crashes::a_crash_at_each_file_operation_of_a_turn_keeps_a_whole_prefix`,
run once with `heavy cargo nextest run -p jig-store-world --test crashes
-E
'test(a_crash_at_each_file_operation_of_a_turn_keeps_a_whole_prefix)'`:
crashed at each of 12 file operations of a commit with a turn, the
recovered store is commit 1, or commit 2 and never part of it, and the
readable turns match it; under 0.5 s.
Don't: use the domain worlds' stand-in here (this world's disk is
skein's); give the scripted root jig's core (its records are fixtures).
Size: no production code; test code at most 800 lines per file.
Removes: nothing.

## Interface sketches

```rust
// jig_test_store (testing/jig-test-store, step code): the test application's part of the store.
use jig_test_connector as connector;
use jig_test_domain as root;
use jig_domain_core as core;
use skein_lib::{Queue, Token};

/// The application's store version (store.md 5.2).
pub const STORE_VERSION: u32 = 1;
/// A record that does not decode.
#[derive(Debug, PartialEq, Eq)]
pub struct Undecodable;
/// A connector's one live interval in the application's key space (store.md 12).
pub fn live(connector: u16) -> skein_kv::Range { todo!() }
/// The application's part of 19.5's sizes check: its version, its largest key and value.
pub fn application(limits: &connector::Limits) -> jig_protocol_store::sizes::Application { todo!() }
pub mod keys {
    use super::connector;
    pub fn encode(number: u16, key: &connector::RecordKey) -> Box<[u8]> { todo!() }
    pub fn decode(bytes: &[u8], limits: &connector::Limits) -> Option<(u16, connector::RecordKey)> { todo!() }
}
pub mod records {
    use super::{Undecodable, connector};
    pub fn encode(record: &connector::Record) -> Box<[u8]> { todo!() }
    pub fn decode(bytes: &[u8], limits: &connector::Limits) -> Result<connector::Record, Undecodable> { todo!() }
}
/// The root's requests the store serves, as the world or the service lifts them out of
/// `root::Request` and `root::Delivery`.
pub enum StoreOut {
    Commit { number: u64, writes: Box<[root::Write]> },
    /// A restart step; `LoadInbox` carries the census the root took from `Core::census` (jig 12.3).
    Restart { step: core::RestartStep, census: Option<core::Census> },
    ReadLoad { load: Token, range: Box<core::reads::Range> },
    NotesLoad { owner: Token, range: jig_domain_notes::Range },
    Now(core::Now),
}
/// What the routing hands on: a row for `restore_record`, an event for the root, or a request back to the store.
pub enum Up {
    Restore(root::Record),
    Root(root::Event),
    Ask(jig_protocol_store::Request),
}
/// What each owner token asked, to answer it in the root's words.
pub struct Routes;
impl Routes {
    /// At most `table` requests in flight to the store.
    pub fn new(table: u32) -> Routes { todo!() }
}
pub fn down(routes: &mut Routes, out: StoreOut) -> Option<jig_protocol_store::Request> { todo!() }
pub fn up(routes: &mut Routes, event: jig_protocol_store::Event, into: &mut Queue<Up>) { todo!() }
```

```rust
// jig_test_store_files (testing/jig-test-store-files, ordinary Rust): the stand-in and the adapter.
use jig_test_domain as root;
use std::collections::{BTreeMap, VecDeque};

/// Transcripts' files in memory, answered by skein's fake machine, with its crash rule (store.md 14).
pub struct Files;
impl Files {
    pub fn new() -> Files { todo!() }
    pub fn root(&mut self) -> jig_protocol_store::Root { todo!() }
    pub fn answer(&mut self, request: skein_io::file::Request) -> jig_protocol_store::Arrived { todo!() }
    pub fn crash(&mut self, seed: u64) { todo!() }
}
/// The worlds' faults between the root and the component (findings/store.md 4.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault { Hold { commits: u32 }, Fail { commit: u64 }, SlowPages { by: u32 } }
/// A commit the world holds before the component.
#[derive(Clone, Debug)]
pub struct Pending { pub number: u64, pub writes: Box<[root::Write]>, pub delay: u32 }
/// The domain worlds' store: the real component over memory and `Files`, with jig-fake-store's face.
pub struct Store {
    /// The last commit the component answered.
    pub applied: u64,
    /// The durable commits' writes, typed, turns included.
    pub rows: BTreeMap<root::Key, root::Record>,
    /// Commits held before the component.
    pub pending: VecDeque<Pending>,
    /// Commits answered but held from the root.
    pub held: VecDeque<u64>,
}
impl Store {
    pub fn new(limits: &root::Limits) -> Store { todo!() }
    pub fn fault(&mut self, fault: Fault) { todo!() }
    pub fn submit(&mut self, number: u64, writes: Box<[root::Write]>, delay: u32) { todo!() }
    /// Ok(Some(n)) committed and released, Ok(None) nothing or held, Err(n) failed.
    pub fn tick(&mut self, hold: bool) -> Result<Option<u64>, u64> { todo!() }
    pub fn release_held(&mut self) -> Option<u64> { todo!() }
    /// Keep both stores and the stand-in after its crash rule; drop what the world held.
    pub fn crash(&mut self, seed: u64) { todo!() }
    /// A fresh component, stores and stand-in, with the durable commits replayed (decision 9).
    pub fn fork(&self) -> Store { todo!() }
    /// One commit of test rows at the next number, the header moved on.
    pub fn seed(&mut self, rows: Box<[root::Record]>) { todo!() }
    pub fn ask(&mut self, out: jig_test_store::StoreOut) { todo!() }
    pub fn answers(&mut self) -> Vec<jig_test_store::Up> { todo!() }
    pub fn loading(&self) -> bool { todo!() }
}
```

```rust
// jig_conformance::harness (testing/jig-conformance, ordinary Rust): the harness over either store.
use std::fmt::Debug;

/// A store the harness drives: jig-fake-store for ops, the adapter for jig's application.
pub trait Durable: Sized {
    type Write: Debug;
    fn submit(&mut self, number: u64, writes: Box<[Self::Write]>, delay: u32);
    fn tick(&mut self, hold: bool) -> Result<Option<u64>, u64>;
    fn release_held(&mut self) -> Option<u64>;
    fn crash(&mut self, seed: u64);
    fn fork(&self) -> Self;
    fn applied(&self) -> u64;
    /// No commit held before the store and no page in flight.
    fn idle(&self) -> bool;
    /// An answer held from the root.
    fn holding(&self) -> bool;
}
// `Application` gains `type Write: Debug;` and `type Store: Durable<Write = Self::Write>;`, its
// functions take `&Self::Store` / `&mut Self::Store`, and `Output::Commit` carries
// `Box<[Self::Write]>`.
```

Who owns what: `jig-test-store` owns the test application's bytes and
its routing, and `jig-test-service` (jig 25) takes both as they are;
`jig-test-store-files` owns the worlds' faults and the stand-in; the
component owns the stores it is made over until the adapter takes them
back at a crash.

## Questions to raise when reached

- **Crashes by keeping or by replay** (18.2): store.md 14 says "A crash
  keeps both in-memory stores and the stand-in, and the next engine's
  component is made over them"; decision 9 says the conformance world
  "rebuilds an in-memory store by replaying its durable commits, for
  snapshots and crashes". This plan keeps the stores at a crash (so the
  stand-in's crash rule and the cut at open run in the domain worlds) and
  replays at a fork (a snapshot, then its crash). If the designer meant
  replay for each crash, the adapter's `crash` replays instead; the
  stories' outcomes do not change.
- **The fork's cost** (18.4): `fuzzy_snapshot` forks at each commit, so
  replaying makes its cost quadratic in commits. If a `fuzzy_snapshot`
  test passes about 5 s, profile before anything else; the fallback is
  forking only at the commits the test compares.
- **The census at the routing** (18.2): jig 12.3 has the root call
  `Core::census` when the core asks `LoadInbox`; the routing needs that
  census with the root's restart request. If the test root does not pass
  it out, raise where the routing takes it from.
- **`ops`'s worlds** stay on `jig-fake-store` (decision 8); if a change
  of the harness for the adapter cannot keep `tests/ops`'s scenarios'
  expectations unchanged, raise it rather than changing them.

## Done when

1. The five increments are on `origin/main`: `git -C ~/src/rust/jig log
   origin/main --format=%b | grep -c '^Step: jig-18\.'` prints 5.
2. jig's own worlds run on the real store: `git -C ~/src/rust/jig grep -n
   -E 'jig-fake-store|jig_fake_store' -- tests/core tests/conformance
   tests/client` prints nothing.
3. `jig-fake-store` serves `ops` alone: `git -C ~/src/rust/jig grep -l
   'jig-fake-store' -- '*.toml'` prints `Cargo.toml`,
   `testing/jig-conformance/Cargo.toml`, `testing/jig-fake-store/Cargo.toml`
   and `tests/ops/Cargo.toml`, and nothing else.
4. The store's world runs within its ceilings: `heavy cargo nextest run
   -p jig-store-world` passes, and the slowest focused test nextest prints
   is under 0.5 s.
5. jig's gate passed on the last of them: `git -C ~/src/rust/jig log -1
   --format='%(trailers:key=Gate-Result,valueonly)' <its commit>` names
   `scripts/check.sh passed`.
