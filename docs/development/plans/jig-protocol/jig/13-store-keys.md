# Session 13: `jig-protocol-store`: the crate, its boundary, keys and intervals

The store's component starts here. This session makes the crate
`jig-protocol-store`, writes its boundary (`protocol/store.md`,
section 3, with what the audit found it lacks), owns the bytes of every
key: the tags of the three owners, an encoder and a decoder for each of
jig's keys on skein-kv's key writer, held to the domain's derived order by
property tests, and maps each range the domain asks for to a short, fixed,
ordered list of intervals, checked against the domain's own predicates.
Nothing here reads or writes a store yet: jig 14 adds the records' codecs,
jig 15 the commits and loads, jig 16 the parties' reads, jig 17 the
transcripts and secrets. The roots' duplicated `core_key` moves into the
domain, where the component needs it too.

## Read first

- jig's `protocol/store.md`, sections 2, 3, 4 (whole), 7.1, 7.2, 7.3 (the
  table), 8.3, 9, 10, 12 and 14 ("Step tests").
- skein's `kv.md`, sections 2 and 2.1 (requests, keys, the key writer).
- jig's `domain/engine.md`, 5.5 and 5.6.
- `findings/store.md`, sections 1.7 (what store.md 3's boundary lacks), 2
  (jig's families and keys), 3 (each scope against 7.3), 5 (skein-kv:
  `KeyWriter`, `Range`, `Loaded.next`) and 7.2 (S1, S2).
- jig 11's and jig 12's files: `tasks::Key`'s order and its index
  families, `Newest`, `RestoreRange` and `Census`.
- `code-organization.md`, sections 2 to 4 (this crate's kind, who links
  what, the boundary's owner).

## What the design asks

- **A component of the engine's protocol layer** (store.md 2): "step code
  under the subset …, with its limits, worst case, deadlines and most
  outputs per entry point. It depends on skein-lib, skein-kv, io's file
  vocabulary, skein-codec and its generated codecs, and the core's
  vocabulary … It depends on no application's crate, and links no smith
  crate: turns are bytes to it."
- **The boundary** (store.md 3): commit, restore, load, read, get,
  transcript and secrets, down; committed with the fill, failed, records
  typed or as bytes, pages with cursors, the record or none, turns or
  unread, kept, erased, a value or none, up. "One terminal per request,
  tagged with the asker's token."
- **Keys** (4.1): "Every key is written with skein-kv's key writer … so
  that byte order equals the order Rust derives for it: a variant's index
  as a tag, then its fields in order; numbers big-endian at fixed width,
  narrower ones widened; an absent option before a present one; a byte
  string escaped and terminated …; a list of byte strings as each string
  marked, then an end mark." "A property test holds each encoder to the
  derived order on generated keys, and each decoder to the encoder."
- **Tags** (4.2): 0 for the component's own rows, 1 for jig ("the child's
  tag, in the order of the core's key: the core's own, then each child's;
  then the family's key"), 2 for the application. "The component prefixes
  them, and refuses an application's key or interval it would have to
  write outside it."
- **Ranges to intervals** (4.3): "each range maps to a short, fixed,
  ordered list of intervals, in the component"; "a test checks each
  mapping against the range's predicate over generated keys: every key the
  predicate admits lies in an interval, and every key in an interval is
  admitted."

## Choices this plan makes

- **The crate depends on** `skein-lib`, `skein-kv`, `skein-io` (for
  `kernel::Fd`), `jig-domain-core`, `jig-domain-tasks`, `jig-domain-people`
  and `jig-domain-notes`; jig 14 adds `skein-codec`.
- **Intervals are `skein_kv::Range`** (start inclusive, end exclusive or
  open), the type the component hands skein-kv: no interval type of this
  crate's own.
- **Narrower numbers** (`u16`, `u32`) are written as `u64`; a fixed array
  (`people::RequestKey.key: [u8; 16]`) as its bytes, one `tag` each. No key
  holds an option today: the option rule is written when one does.
- **A cursor across a list of intervals** names the interval and the last
  key returned: `Cursor { interval: u32, after: Box<[u8]> }`.
- **The application's keys** are relative to its tag: the component adds
  the tag on the way down and strips it on the way up, so no application
  key or interval can reach outside tag 2.
- **The secrets' values are `Box<[u8]>`** in this session's boundary:
  skein-lib has no `Secret` at the plan's skein revision. jig 17, whose
  first secrets increment needs jig 02 (skein's `Secret`), changes the
  field's type to `skein_lib::Secret` in that increment.
- **Reads' scopes whose keys follow from the record** (an activity page
  from the owner's last position, a question's, an escalation's and a
  task proposal's single read, a subtree's history) get no fixed interval
  here: jig 16 serves them by point reads from the owner's record (jig
  11.3, jig 12.5).
- **`core_key` moves into the domain** as `jig_domain_core::Record::key`,
  with `CoreRecord::key` and `jig_domain_notes::Record::key`, which the two
  roots and this crate call.

## Rules that bind it

- **Step code** (`code-organization.md`, section 2): "Step code follows
  skein's `programming-model.md`, section 10, and jig's `AGENTS.md`: no
  traits, no generic items, no `dyn`, no closures." Its tests too: they
  use `Box`, arrays and `skein_lib::List`, never `Vec`, and add no lint
  exception ("adds a lint exception in tests" is on the plan's "Don't"
  list).
- **The boundary is this session's** (`code-organization.md`, section 4):
  "One session defines each; every other session uses it by name and
  extends it only as its increment says." The types sketched for
  `crates/jig-protocol-store/src/boundary.rs` land in 13.1, field by
  field.
- **"`jig-protocol-*` crates depend on no application's crate. Only
  `jig-protocol-charter` and `jig-protocol-transcript` link smith"**
  (`code-organization.md`, section 3), checked by `tests/boundary`.

## Increments

### 13.1 The crate, its boundary, and the domain's record keys

Scope: the crate `crates/jig-protocol-store` (step code), its
`boundary.rs` as sketched below, each type (store.md 3, 9, 10; the audit's
`findings/store.md` 1.7: fill on committed, a transcript dropped whole, the
window's and a turn's and a page's transcript reads, the application's
intervals as bytes, its loads and gets, the secrets' requests), the tags'
constants; `Record::key` in the domain, used by both roots.
Needs: nothing beyond the session's start (jig 11's keys).
Files:
- `crates/jig-protocol-store/{Cargo.toml, src/lib.rs, src/boundary.rs, src/keys.rs}` (new; `keys.rs` holds the tags only);
- the workspace `Cargo.toml` (`members`; `[workspace.dependencies]` rows `skein-kv`, `skein-io` from skein's git URL, as `skein-lib`'s row at :67, and `jig-protocol-store`; shared hotspot);
- `tests/boundary/src/tests.rs` (the crate among the protocol layer's, as jig 01's rules name them; shared hotspot);
- `crates/jig-domain-core/src/stored.rs` (`Record::key`, `CoreRecord::key`), `crates/jig-domain-notes/src/boundary.rs` (`Record::key`);
- `testing/jig-test-domain/src/lib.rs` (`core_key` :52-79 removed; `Record::key` :41-50 calls the domain's), `examples/ops/domain/src/store.rs` (`core_key` :60-85 removed; `Record::key` :41-52).
Shape:
- `jig_protocol_store::boundary`: `Request`, `Write`, `Restore`, `Load`,
  `Get`, `TranscriptRead`, `Secrets`, `SecretOwner`, `Event`, `Row`,
  `Cursor`, `OpenRefusal`, `Family`, as sketched; re-exported from
  `lib.rs`. No function reads them yet.
- `jig_protocol_store::keys::{COMPONENT, JIG, APPLICATION}`: `u8` 0, 1, 2.
- `jig_domain_core::Record::key(&self) -> Key`, `CoreRecord::key(&self) -> CoreKey`,
  `jig_domain_notes::Record::key(&self) -> Key`; the bodies are the roots'
  `core_key`, moved.
Test shape:
- `crates/jig-domain-core/src/tests/restore_admission.rs` (fixtures at
  `crates/jig-domain-core/src/tests.rs:29,193`):
  `a_records_key_is_the_key_its_row_is_saved_under` (one record per
  `CoreRecord` variant, keys written as literals).
- `tests/boundary`: the crate is checked by the protocol rule jig 01 wrote;
  no new test name.
- Ceiling: unit 20 ms.
Proof: unit. `heavy cargo nextest run -p jig-domain-core
restore_admission` and `heavy cargo nextest run -p jig-boundary` once: a
record's key comes from the domain, and the new crate links no smith or
application crate. No higher layer reads the boundary yet (jig 15 does).
Bound: 1 minute.
Don't: give any boundary type a behaviour; depend on skein-codec yet.
Size: at most 200 lines of production code.
Removes: `core_key` in `testing/jig-test-domain/src/lib.rs:52-79` and
`examples/ops/domain/src/store.rs:60-85`.

### 13.2 Keys: the writer's rules, the component's, the core's and people's

Scope: the encoder and decoder of the component's own keys (tag 0: the
format row, a task's transcript places, an attempt's first turn), of
`CoreKey` (with `ProjectionKey`) and of `people::Key`, under tag 1 and the
child's tag; `keys::encode` and `keys::decode` for `core::Key` dispatching
to each child; the application's prefix (store.md 4.1, 4.2).
Needs: nothing beyond 13.1.
Files: `crates/jig-protocol-store/src/keys.rs` (13.1's; and new `keys/own.rs`,
`keys/core.rs`, `keys/people.rs`), `src/lib.rs`; `src/tests.rs`,
`src/tests/fixtures.rs` and `src/tests/keys.rs` (new).
Shape:
- `keys::Own { Format, Transcript { task: u64 }, Attempt { task: u64, attempt: u64 } }`,
  deriving `Ord`; `keys::encode_own(&Own, limit: u32) -> Option<Box<[u8]>>`,
  `keys::decode_own(&[u8]) -> Option<Own>`.
- `keys::encode(&core::Key, limit: u32) -> Option<Box<[u8]>>` (`None` past
  `limit`), `keys::decode(&[u8]) -> Option<core::Key>` (`None` for an
  unknown tag, a short key or bytes left over); in this increment the
  `Tasks` and `Notes` arms answer `None` (13.3 fills them; nothing calls
  `encode` before jig 15).
- `keys::application(key: &[u8], limit: u32) -> Option<Box<[u8]>>` adds
  tag 2; `keys::of_application(bytes: &[u8]) -> Option<&[u8]>` strips it.
- Layout: tag, then the child's tag (the variant index of `core::Key`:
  Core 0, Tasks 1, People 2, Notes 3), then the family's (the variant index
  of the child's key), then its fields in order.
Test shape:
- `src/tests/fixtures.rs`: `KEY_LIMIT: u32 = 1024`; generators
  `own(rng)`, `core_key(rng)`, `people_key(rng)` and `key(rng)` drawing
  each number from `[0, 1, 2, u32::MAX as u64, u64::MAX]` and each family
  at random, so that keys share prefixes; `within(intervals:
  &[skein_kv::Range], key: &[u8]) -> bool` for 13.4.
- New `src/tests/keys.rs`. Reference test, written out in full:

  ```rust
  //! Keys' bytes against the order Rust derives (store.md 4.1, 14).
  use super::fixtures;
  use crate::keys;
  use skein_lib::Rng;

  #[test]
  fn core_and_people_keys_sort_as_bytes_as_they_derive() {
      let mut rng = Rng::new(0x5701_e001);
      for _ in 0..2_000 {
          let left = fixtures::key(&mut rng);
          let right = fixtures::key(&mut rng);
          let left_bytes = keys::encode(&left, fixtures::KEY_LIMIT).expect("within the key limit");
          let right_bytes = keys::encode(&right, fixtures::KEY_LIMIT).expect("within the key limit");
          assert_eq!(left.cmp(&right), left_bytes.cmp(&right_bytes), "{left:?} against {right:?}");
      }
  }
  ```
- Also: `every_generated_key_decodes_to_itself`,
  `own_keys_sort_as_bytes_as_they_derive`, `a_key_with_bytes_left_over_does_not_decode`,
  `a_key_past_its_limit_is_none`, `an_application_key_carries_tag_two_and_strips_back`.
- Ceiling: unit 20 ms each (2,000 pairs).
Proof: unit. `heavy cargo nextest run -p jig-protocol-store` once: on
generated keys, bytes sort as the domain's keys do and decode back to them.
Bound: 1 minute.
Don't: use skein's generator for keys (store.md 4.1: "Written out, not
generated"); write an option or list rule no key uses.
Size: at most 250 lines of production code.
Removes: nothing.

### 13.3 Keys: the tasks child's and the notes child's

Scope: the encoders and decoders of `tasks::Key` (live families, history
and, once jig 12 has landed them, the index families with `Newest` written
raw) and of `notes::Key` (`Scope` and `Pattern`: each segment marked, then
an end mark; `Last` as a tag and its bytes); `keys::encode` and `decode`
fill their two arms (store.md 4.1).
Needs: nothing beyond 13.2. The index families arrive with jig 12: an
increment of jig 12 that lands after this one extends `keys/tasks.rs`
itself (jig 12's "Rules").
Files: `crates/jig-protocol-store/src/keys/{tasks.rs, notes.rs}` (new),
`src/keys.rs`, `src/tests/fixtures.rs` (`tasks_key`, `notes_key`, and
`key` drawing from all four children), `src/tests/keys.rs`.
Shape:
- `Funder` as its variant index then its fields; `Newest` as `u64` of its
  `raw()`; `UpdateObject` as its variant index then its fields.
- `notes::Key::Line { scope, name }`: scope's variant index; `project`,
  `goal`, `connector` widened; each pattern segment as `tag(1)` then
  `bytes`, then `tag(0)`; `last` as its variant index then `bytes`.
Test shape:
- In `src/tests/keys.rs`: `tasks_keys_sort_as_bytes_as_they_derive`,
  `notes_keys_with_shared_pattern_prefixes_sort_as_they_derive` (patterns
  drawn from segments `[]`, `[0]`, `[0, 0]`, `[0, 255]`, `[1]`),
  `a_shorter_pattern_sorts_before_its_extension`. The reference test of
  13.2 now draws from every child.
- Ceiling: unit 20 ms each.
Proof: unit. `heavy cargo nextest run -p jig-protocol-store` once: on
generated keys of all four children, bytes sort as they derive and decode
back. Bound: 1 minute.
Don't: escape a pattern's segments differently from `KeyWriter::bytes`.
Size: at most 200 lines of production code.
Removes: the two `None` arms 13.2 left.

### 13.4 Ranges to intervals: the restart, the notes child, fixed scopes

Scope: `intervals::restore` for `RestoreRange::LIVE` (the deployment's
header, people, the tasks child's live families, the core's live rows),
`intervals::notes` for the notes child's loads, and `intervals::scope` for
the reads' scopes whose keys follow from the scope: a task, a person task,
a result (`Live` then `Ended`, by key), a reply, a party's proposal
(`PersonProposal`, `DecidedProposal`), a task's own history (two
intervals), spend (`Ledger` and `ClosedLedger` of the project's periods),
a project's notes (its scopes' lines) and one scope's notes; each checked
against the domain's predicate (store.md 4.3, 7.2, 7.3).
Needs: nothing beyond 13.3.
Files:
- `crates/jig-protocol-store/src/intervals.rs` (new), `src/tests/intervals.rs` (new);
- `crates/jig-domain-core/src/reads.rs` (exports `admits`, the predicate over a stored row's key, from `rows::position_allowed` at `reads/rows.rs:1427`).
Shape:
- `intervals::restore(range: &core::RestoreRange) -> Box<[skein_kv::Range]>`;
  `intervals::notes(range: &notes::Range) -> Box<[skein_kv::Range]>`;
  `intervals::scope(party: u64, scope: &reads::Scope) -> Option<Box<[skein_kv::Range]>>`,
  `None` for the scopes jig 16 serves from the owner's record or jig 12's
  indexes; each list ordered as its rows are handed up, at most 4
  intervals.
- `jig_domain_core::reads::admits(party: u64, scope: &Scope, key: &Key) -> bool`:
  `position_allowed(scope, &Position::Owner(key))` and the party's own
  keys, at most 20 lines.
Test shape:
- New `src/tests/intervals.rs`. Reference test, written out in full:

  ```rust
  //! Each range's intervals against its predicate (store.md 4.3).
  use super::fixtures;
  use crate::{intervals, keys};
  use jig_domain_core::RestoreRange;
  use skein_lib::Rng;

  #[test]
  fn each_restart_range_holds_the_keys_it_admits_and_no_others() {
      let mut rng = Rng::new(0x5701_e004);
      for _ in 0..2_000 {
          let key = fixtures::key(&mut rng);
          let bytes = keys::encode(&key, fixtures::KEY_LIMIT).expect("within the key limit");
          for range in RestoreRange::LIVE {
              let inside = fixtures::within(&intervals::restore(&range), &bytes);
              assert_eq!(range.admits(&key), inside, "{range:?} and {key:?}");
          }
      }
  }
  ```
- Also: `the_restart_reads_its_ranges_in_the_cores_order` (the four lists'
  first starts, as literal bytes), `a_notes_load_holds_exactly_its_scopes_lines`,
  `each_fixed_scope_holds_the_keys_it_admits_and_no_others` (parties 1 and 2,
  each scope this increment maps), `no_restart_interval_holds_a_turn_terminal_or_note`.
- Ceiling: unit 20 ms each.
Proof: unit. `heavy cargo nextest run -p jig-protocol-store intervals`
once: on 2,000 generated keys, each key a range admits lies in its
intervals and no key in them is refused by it. Bound: 1 minute.
Don't: map a scope whose keys need the owner's record; read the store.
Size: at most 220 lines of production code.
Removes: nothing.

### 13.5 Intervals of the indexed scopes and the census

Scope: `intervals::scope` for jig 12's indexed scopes (an inbox's updates,
results and replies; conversations newest first and oldest first; a
project's goals; a task's ended delegates) and `intervals::restore` for
`RestoreRange::Census` (the oldest-first conversations, then each listed
party's results after its read position); `intervals::delegates` for a
subtree's collection (store.md 7.2, 7.3).
Needs: jig 12's index families, `Census` and `index_key` (each scope here
needs only the increment of jig 12 that lands its family).
Files: `crates/jig-protocol-store/src/intervals.rs`, `src/tests/intervals.rs`,
`src/tests/fixtures.rs` (index keys in `tasks_key`).
Shape:
- An indexed scope's list starts at the scope's party, project or
  requester, and ends at the next one; `intervals::delegates(requester: u64) -> skein_kv::Range`.
- `intervals::restore(&RestoreRange::Census(census))` holds one interval for
  the conversations and one per `ReadResults`, in the census's order.
Test shape:
- In `src/tests/intervals.rs`: `each_indexed_scope_holds_the_keys_it_admits_and_no_others`,
  `the_census_holds_the_keys_it_admits_and_no_others`,
  `a_partys_updates_interval_holds_no_other_partys_rows`.
- Ceiling: unit 20 ms each.
Proof: unit. `heavy cargo nextest run -p jig-protocol-store intervals`
once: on generated keys, each indexed scope's and the census's intervals
hold the keys the domain's predicates admit and no others. Bound: 1 minute.
Don't: map needs groups (jig 12: never stored).
Size: at most 120 lines of production code.
Removes: nothing.

## Interface sketches

```rust
// jig_protocol_store::boundary (crates/jig-protocol-store/src/boundary.rs)
// store.md, sections 3, 7 to 10. Owned by jig 13; jig 15 to 17 implement it.
use alloc::boxed::Box;
use jig_domain_core::{self as core, RestoreRange, reads};
use jig_domain_notes as notes;
use skein_io::kernel::Fd;
use skein_lib::Token;

/// What the engine's protocol layer asks of the component. Each request but
/// `Commit` names its asker's token; each has one terminal event.
#[derive(Debug)]
pub enum Request {
    /// Open both stores in the deployment's directory (store.md 10); the
    /// component's own rows and the core's header are checked before
    /// `Opened`. In skein-kv's memory mode, the worlds' component needs none.
    Open { owner: Token, root: Fd },
    /// Commit `number` (store.md 6): its writes, in order, one skein-kv
    /// commit with the same number. Terminal: `Committed` or `Failed`.
    Commit { number: u64, writes: Box<[Write]> },
    /// A page of one restart step's ranges (store.md 7.2), from `after`.
    Restore { owner: Token, range: Restore, after: Option<Cursor>, rows: u32, bytes: u64 },
    /// A page of a child's range (store.md 7.1), from `after`.
    Load { owner: Token, range: Load, after: Option<Cursor>, rows: u32, bytes: u64 },
    /// A party's read (store.md 7.3): its scope, position, direction and bounds.
    Read { owner: Token, range: reads::Range },
    /// One record by key.
    Get { owner: Token, key: Get },
    /// A transcript's turns (store.md 8.3).
    Transcript { owner: Token, read: TranscriptRead, rows: u32, bytes: u64 },
    /// The secrets' store (store.md 9).
    Secrets { owner: Token, request: Secrets },
}

/// One write of a commit (store.md 6).
#[derive(Debug)]
pub enum Write {
    /// jig's save: a put of its encoded key and record; a turn's save
    /// becomes a frame in its task's transcript and puts of its places.
    Save(core::Record),
    /// jig's erase.
    Erase(core::Key),
    /// A task's transcript dropped whole (store.md 8.5): its places erased,
    /// its file removed before the commit is submitted.
    DropTranscript { task: u64 },
    /// The application's put: its key's bytes (without the tag) and its value's.
    Put { key: Box<[u8]>, value: Box<[u8]> },
    /// The application's erase.
    Remove { key: Box<[u8]> },
}

/// What a restart step restores.
#[derive(Debug)]
pub enum Restore {
    /// jig's ranges, in order: `RestoreRange::LIVE` for `LoadCore`, the
    /// census for `LoadInbox`. Rows are handed up typed; the census's are
    /// the ended tasks its index rows name.
    Jig(Box<[RestoreRange]>),
    /// A connector's live intervals, relative to the application's tag.
    Application(Box<[skein_kv::Range]>),
}

/// A child's range to load.
#[derive(Debug)]
pub enum Load {
    /// The notes child's entry or a scope's lines.
    Notes(notes::Range),
    /// The application's intervals, relative to its tag.
    Application(Box<[skein_kv::Range]>),
}

/// A record to get.
#[derive(Debug)]
pub enum Get {
    Jig(core::Key),
    /// The application's key, relative to its tag.
    Application(Box<[u8]>),
}

/// A transcript read (store.md 8.3). Offsets are the component's, given
/// and taken back; none reads past what is committed.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TranscriptRead {
    /// A resuming run's current window, from the turn the core names as its
    /// start; a window that starts at another turn is `Unread`.
    Window { task: u64, attempt: u64, turn: u32, after: Option<u64> },
    /// One turn, walked to from its attempt's first turn.
    Turn { task: u64, attempt: u64, turn: u32 },
    /// A page of a task's turns from either end; a fresh run's tail is
    /// `backwards: true, after: None`.
    Page { task: u64, backwards: bool, after: Option<u64> },
}

/// Whose secret (store.md 9).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum SecretOwner {
    /// A sign-in's digest, by the sign-in's number.
    SignIn,
    /// A service's credential's digest, by the service's number.
    Service,
    /// The application's own record, under its tag, by its number.
    Application,
}

/// A request to the secrets' store. jig 17 makes `value` a `skein_lib::Secret`.
#[derive(Debug)]
pub enum Secrets {
    Keep { owner: SecretOwner, number: u64, value: Box<[u8]> },
    Erase { owner: SecretOwner, number: u64 },
    Get { owner: SecretOwner, number: u64 },
}

/// Where a page stopped: the interval of the request's list, and the last
/// key it returned (store.md 7.1: the next page starts just past it).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Cursor {
    pub interval: u32,
    pub after: Box<[u8]>,
}

/// One restored or loaded row.
#[derive(Debug)]
pub enum Row {
    /// jig's record, decoded.
    Jig(core::Record),
    /// The application's row, its key relative to its tag.
    Application { key: Box<[u8]>, value: Box<[u8]> },
}

/// Whose format the format row disagrees on (store.md 5.3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Family {
    Component,
    Core,
    Tasks,
    People,
    Notes,
    Application,
}

/// Why a store refuses to open, saying which (store.md 5.3, 6, 10).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OpenRefusal {
    /// The format row names another version of `family`.
    Format { family: Family, found: u32, wanted: u32 },
    /// The format row names another deployment than the header.
    Deployment,
    /// skein-kv's recovered last commit is not the header's.
    Numbers { store: u64, header: u64 },
    /// A file failed, or a store did not recover.
    Failed,
}

/// What the component answers.
#[derive(Debug)]
pub enum Event {
    /// Both stores are open and checked; `last` is the last durable commit.
    Opened { owner: Token, last: u64 },
    OpenRefused { owner: Token, why: OpenRefusal },
    /// Commit `number` is durable, and every earlier one; `fill` is the
    /// map's bytes (skein-kv's `used`), for the core's high-water mark.
    Committed { number: u64, fill: u64 },
    /// Commit `number` failed or was refused; the engine stops (store.md 6).
    Failed { number: u64 },
    /// A restore's page; `next` is none once the step's ranges are done.
    Restored { owner: Token, rows: Box<[Row]>, next: Option<Cursor> },
    /// A load's page; `next` is none once the range is done.
    Loaded { owner: Token, rows: Box<[Row]>, next: Option<Cursor> },
    /// A read's page, busy, or gone.
    Read { owner: Token, result: reads::Loaded },
    Got { owner: Token, row: Option<Row> },
    /// A transcript's turns; `next` is the offset to continue from.
    Turns { owner: Token, turns: Box<[core::TurnRecord]>, next: Option<u64> },
    /// A transcript read failed: a frame's checksum, a window at another
    /// turn, a dropped transcript (store.md 8.3, 8.5).
    Unread { owner: Token },
    Kept { owner: Token },
    Erased { owner: Token },
    Secret { owner: Token, value: Option<Box<[u8]>> },
    /// A stored row did not decode: the request's terminal; the engine
    /// stops (store.md 7.1).
    Corrupt { owner: Token },
}
```

```rust
// jig_protocol_store::keys (crates/jig-protocol-store/src/keys.rs)
use alloc::boxed::Box;
use jig_domain_core as core;

/// The owners' tags (store.md 4.2).
pub const COMPONENT: u8 = 0;
pub const JIG: u8 = 1;
pub const APPLICATION: u8 = 2;

/// The component's own rows, under `COMPONENT` (store.md 5.3, 8.1).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Own {
    /// The format row: one row.
    Format,
    /// A task's transcript: its committed length and its window's start.
    Transcript { task: u64 },
    /// An attempt's first turn's offset.
    Attempt { task: u64, attempt: u64 },
}

pub fn encode_own(key: &Own, limit: u32) -> Option<Box<[u8]>> { todo!() }
pub fn decode_own(bytes: &[u8]) -> Option<Own> { todo!() }
/// jig's key under `JIG`, then the child's tag; none past `limit`.
pub fn encode(key: &core::Key, limit: u32) -> Option<Box<[u8]>> { todo!() }
/// None for an unknown tag, a short key, or bytes left over.
pub fn decode(bytes: &[u8]) -> Option<core::Key> { todo!() }
pub fn application(key: &[u8], limit: u32) -> Option<Box<[u8]>> { todo!() }
pub fn of_application(bytes: &[u8]) -> Option<&[u8]> { todo!() }
```

```rust
// jig_protocol_store::intervals (crates/jig-protocol-store/src/intervals.rs)
use alloc::boxed::Box;
use jig_domain_core::{RestoreRange, reads};
use jig_domain_notes as notes;

/// A restart range's intervals, in the order its rows are handed up.
pub fn restore(range: &RestoreRange) -> Box<[skein_kv::Range]> { todo!() }
/// The notes child's load: an entry's key, or a scope's lines.
pub fn notes(range: &notes::Range) -> Box<[skein_kv::Range]> { todo!() }
/// A party's read scope's intervals, or None where jig 16 serves it from
/// the owner's record.
pub fn scope(party: u64, scope: &reads::Scope) -> Option<Box<[skein_kv::Range]>> { todo!() }
/// The `Delegate` rows of one requester, for a subtree's collection.
pub fn delegates(requester: u64) -> skein_kv::Range { todo!() }
```

- **Owners.** This crate owns the bytes of every key and the boundary;
  the domain owns the keys' order and the predicates (`RestoreRange::admits`,
  `Census::admits`, `reads::admits`). The application owns its key bytes
  under tag 2.
- **What ends what.** Each request ends with its one terminal; a commit
  with `Committed` or `Failed`, and after `Failed` or `Corrupt` the root
  stops the engine and nothing more is answered.

## Questions to raise when reached

- **Notes' pages' positions** (13.4). store.md 7.3 serves "a project's
  notes, a scope's notes" from "the notes' lines …, each with its entry,
  by name", but the core pages notes by the entries' keys
  (`crates/jig-domain-core/src/reads/rows.rs:343`, `key_kind` :1540-1543
  maps lines to `Unsupported`). This session maps the scopes to the lines'
  intervals; the core's positions over lines are not in jig 11 or 12.

## Done when

- `git -C ~/src/rust/jig grep -n 'fn core_key' -- testing examples` prints
  nothing.
- `git -C ~/src/rust/jig grep -n -E 'smith|jig-ops|jig-test' --
  crates/jig-protocol-store/Cargo.toml` prints nothing.
- `heavy cargo nextest run -p jig-protocol-store -p jig-boundary` passes
  once on `main` after 13.5.
