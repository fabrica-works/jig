# Session 17: the store's files: transcripts and secrets

`jig-protocol-store` gets its files: the deployment's directory with both
skein-kv stores on it and the transcripts' root beside them; the secrets'
store, a second skein-kv store in a private directory, kept, erased, got
and loaded by owner and number, swept after people's families are
restored; transcripts as one file per task, each turn a frame written and
synced before the commit that accepts it, its places kept in the map;
turns read back by window, by turn and by page from either end, never
past what is committed; files cut to their committed length at open and
removed when a transcript is dropped; and the turns parties read. It
takes skein's `OpenWrite` and `Cut` and lib's `Secret` (jig 02) and the
core's windows (jig 06).

## Read first

- jig's `protocol/store.md`, sections 2, 3, 6, 8, 9, 10, 13 and 15 (item
  4); `protocol/README.md`, section 8 (secrets kept before used).
- jig's `domain/engine.md`, sections 5.4 and 7.2.
- skein's `io.md`, section 5 (`OpenWrite`, `Cut`, `OpenPrivate`), 5.3 and
  9 ("Roots by token everywhere"); skein's `kv.md`, sections 2, 4 and 8.
- `decisions.md`, 3, 13 (dropping a transcript is built; retention
  is later), 22 (wall times as `u64`), 24.
- `findings/store.md`, sections 1.6, 5 and 7.2 (S13, S14, S15);
  `findings/skein.md`, section 6 (skein 03's `OpenWrite`, `Cut`,
  `Event::Cut`).
- jig 13's file: `Request::{Open, Transcript, Secrets}`, `TranscriptRead`,
  `SecretOwner`, `Secrets`, `Write::DropTranscript`, `keys::Own`; jig
  14's 14.1 (`codec::component::{Transcript, Attempt}`); jig 19's 19.2
  (`codec::core::turn`, `codec::core::read_turn`); jig 06's 06.4
  (`Now::StartPreparation { from }`) and `TurnRecord.opens_window`; jig
  31's secrets' requests (`Request::Secrets`: load at start, keep, erase).
- The code: skein's `crates/skein-io/src/file.rs:31-200` (requests and
  events; `Create`, `OpenRead`, `SyncDirectory` and `Remove` name roots
  by `Fd`, `MakeDirectory` and `OpenPrivate` by `Token`),
  `file_layer.rs:338-351` (`adopt_root`, `descriptor`), `:587-607`
  (`Create` takes a path beneath its root), `:571-578` (`Remove` takes a
  plain name), and skein 03's `OpenWrite` and `Cut`;
  `crates/skein-kv/src/store.rs:25` (`IO_OWNER`), `:872-881` (`is_ready`,
  `fire`); jig's `crates/jig-domain-core/src/stored.rs:455-535`
  (`TurnRecord`, `RunProof`) as jig 05 and 06 changed them;
  `crates/jig-domain-people/src/boundary.rs:530-600` (`Key::SignIn`,
  `Key::Person`); jig 15's `open.rs`, `commits.rs`, `restore.rs`; jig 16's
  `reads/point.rs`.

## What the design asks

- **On disk** (store.md 2): `<deployment>/store/` (skein-kv),
  `transcripts/` ("one file per task, in directories by the task's
  number"), `secrets/` ("the directory 0o700, its files 0o600").
- **Three file drivers** (13): "one for each store ... and one for
  transcripts, so a turn's sync never queues behind a store's write, nor
  a store's behind a turn's."
- **Secrets** (9): a second skein-kv store opened as a private directory;
  sign-ins' digests, services' credentials' digests and the application's
  records, each "under its owner's tag and the number the domain gives
  it"; "Written by components alone"; "Kept before used"; erased when the
  core says, and "once people's families are restored, the component
  erases each of jig's digests whose sign-in or service it did not hand
  up"; "A failure stops the engine"; "Read at start, whole".
- **A frame per turn** (8.1): "the turn's attempt and number, its spend,
  the last message it read, its time, whether it opens a window, then
  smith's bytes, with a checksum, and the frame's length at both ends";
  the map keeps each task's committed length and its current window's
  first turn's offset, and each attempt's first turn's offset; "A frame
  whose turn opens a window moves the task's current window to it, in the
  same commit."
- **Written before its commit** (8.2): write each frame at the file's end
  ("the committed length, plus the frames queued ahead and not yet
  durable"), sync each file, and for a created file its directory, then
  submit the commit with the new places; "A commit with turns goes to
  skein-kv once their files are synced, and the commits after it queue
  behind it" (6).
- **Read back** (8.3): resuming from the window's first turn ("a window
  that starts at another turn fails the read as unread"), a turn by
  walking from its attempt's first turn, a page either way from an offset
  cursor, a tail backwards within a budget; "Never past what is
  committed"; a frame failing its checksum "fails the read as unread";
  "Open files are bounded".
- **After a crash** (8.4): at open, cut each run proof's task's file to
  its committed length and sync it; "a missing file is left missing".
- **Dropped** (8.5): the commit erases the places; the component removes
  the file and syncs the directory before it submits; "Removing a missing
  file is not a failure."
- **Parties' turns** (7.3): "A turn's is read from its transcript, and
  answers once its frame is read."

## Choices this plan makes

- **The deployment's directory** reaches the component in `Open` as a
  root the shell opened and the service adopted, its token and its
  descriptor (`Root { token, fd }`): `MakeDirectory` and `OpenPrivate`
  name a root by token, the others by descriptor. jig 13's `Open { root:
  Fd }` becomes `Open { root: Root }` in 17.1 (its users then: the crate's
  own tests). On a directory the component
  makes `store/` and `transcripts/` (`0o700`; an existing one is taken as
  it is) and opens `secrets/` with `OpenPrivate`. In memory mode, `root`
  is the transcripts' directory itself (jig 18's stand-in gives one).
- **Descriptors:** skein's `Create`, `OpenWrite`, `SyncDirectory`,
  `Remove` and skein-kv's `Open` take a root's `Fd`, which only the
  driver's owner reads (`FileIo::descriptor`; io.md 9). The routing hands
  each file event to the component with the descriptor of the file it
  opened (`Arrived { event, fd }`), so the component never reads io's
  table. This ends when io takes roots by token (io.md 9).
- **Owners on a driver:** each file request carries the component's own
  owner token or skein-kv's `IO_OWNER`, and each event goes back to the
  one that asked.
- **Transcripts' names:** `<high>/<task>` beneath the transcripts' root,
  `<high>` the task's number shifted right by 12 bits, both as 16
  lowercase hex digits; at most 4 096 files per directory.
- **A frame:** `length: u32`, then the turn as jig 19's
  `codec::core::turn` encodes it (its attempt, number, spend, last message
  read, time, whether it opens a window, and its bytes), then `crc32c:
  u32` over that encoding, then `length: u32` again; big-endian. The
  checksum is CRC-32C written in the component (skein-kv's is private),
  checked against `crc32c(b"123456789") == 0xE306_9283`.
- **A turn's save becomes a frame and places, never a row:** the
  component takes each `Write::Save(Record::Core(CoreRecord::Turn(..)))`
  of a commit out of its operations and writes its frame; this ends jig
  15.2's transitional turn rows.
- **The open files' set** keeps at most `Limits::files` transcripts open,
  closing the one written or read longest ago when full.
- **Secrets cross the boundary as `skein_lib::Secret`** (jig 13's choice:
  "jig 17 ... changes the field's type to `skein_lib::Secret`"), so a
  `Debug` of a request shows their lengths only. The store keeps a
  secret's value as its owner's bytes and never decodes it: jig 31 encodes
  a digest and its expiry.
- **The secrets' load** that jig 31 asks at start ("load at start")
  is `Secrets::Load { owner, after }`, answered a page at a time; jig 13's
  `Secrets` gains it in 17.1. It is how the store's "Read at start, whole"
  reaches the component that compares digests.
- **The sweep** walks jig's secrets in key order and gets each one's
  people key from the store (`People::SignIn(n)` for a sign-in's,
  `People::Person(n)` for a service's); an absent key's secret is erased,
  at most `skein_kv::Limits::ops` erases per secrets' commit. It runs
  where 15.3 named its point: the end of the `People` range of the core's
  restore.
- **Dropping is built and reached only through the store's boundary**
  (decision 13): `Write::DropTranscript { task }` (jig 13's); no domain
  write produces it until retention is planned (`later.md`).

## Rules that bind it

- **Shared interfaces:** "the store's boundary (commit, restore, load,
  read, get, transcript, secrets; fill) | `jig-protocol-store`'s
  `boundary.rs` | jig 13". "One session defines each; every other
  session uses it by name and extends it only as its increment says."
  (code-organization.md, section 4.) This session's extensions: `Open`'s
  root (17.1), `Secrets::Load` and its answer (17.1), the secrets' values
  as `Secret` (17.1).
- **"Transitional code has an end"** (skein's development.md, 5.3): the
  descriptor hand-over (`Arrived.fd`) ends with io's roots by token; 17.2
  removes jig 15.2's turn rows, 17.4 its `DropTranscript` arm, 17.5 jig
  16.2's `Turn` arms.
- **Step code** "follows skein's `programming-model.md`, section 10, and
  jig's `AGENTS.md`: no traits, no generic items, no `dyn`, no closures."
  (code-organization.md, section 2.) The tests too (jig 13's "Rules").

## Increments

### 17.1 The deployment's directory and the secrets' store

Scope: store.md 2, 9, 10 and 13.
- **On a directory:** `Store::on_directory(limits, application) ->
  Store` (two `skein_kv::Store::new`); its `Open { owner, root }` makes
  `store/` and `transcripts/`, opens each with `OpenDirectory`, opens
  `secrets/` with `OpenPrivate`, then sends skein-kv `Open { root }` for
  both stores, each on its own driver (`Below.store`, `Below.secrets`),
  and goes on to 15.1's checks once both answer `Opened`. skein-kv's file
  requests and events pass through by `IO_OWNER`; the component's `fire`
  calls skein-kv's `fire` for each store `is_ready`. On disk, skein-kv's
  `Failed { number }` answers that commit `Failed`; `Recovered` and
  `RecoveryFailed` stop the component as a failure does (store.md 6: "The
  engine never carries on over a store that recovered beneath it");
  `OpenFailed` answers `OpenRefused { why: OpenRefusal::Failed }`.
- **The secrets' requests:** `Keep { owner, number, value }` commits to
  the secrets' store and answers `Kept` once skein-kv answers; `Erase`
  likewise `Erased`; `Get` answers `Secret { value }` in the step; `Load
  { owner, after }` answers `SecretsLoaded { rows, next }`, a page of that
  owner's secrets after the number given. Keys: jig's under tag 1, the
  application's under tag 2, then the owner, then the number. A secrets'
  commit refused or failed stops the component, the request unanswered
  (Questions).
- **The sweep** (Choices), in memory and on a directory.
Needs: jig 02's skein (`Secret`, `OpenPrivate`'s checks).
Files: `crates/jig-protocol-store/src/{directory.rs (new), secrets.rs
(new), boundary.rs (jig 13's: `Open`'s root, `Secrets::Load`,
`SecretsLoaded`, `Secret` values, the io face), lib.rs, open.rs,
restore.rs}`; `src/tests/{fixtures.rs, directory.rs (new), secrets.rs
(new)}`.
Shape:
- `Root { token: Token, fd: Fd }`; `Below { store: Queue<file::Request>,
  secrets: Queue<file::Request>, transcripts: Queue<file::Request> }`;
  `Io::{Store(Arrived), Secrets(Arrived), Transcripts(Arrived)}`;
  `Arrived { event: file::Event, fd: Option<Fd> }`.
- The crate's entry points take the io face: `down(store, env, request,
  above, below: &mut Below)`, `up(store, env, io: Io, above, below)`,
  `fire(store, env, above, below)`; jig 15's memory path leaves `below`
  empty. Users of `down` by then: the crate's own tests.
- `secrets::secrets(store, owner, request: Secrets, above)`;
  `secrets::sweep(store, above)`; `SecretRow { owner: SecretOwner,
  number: u64, value: Secret }`.
Test shape: `src/tests/directory.rs` (scripted io: the test pops each
request the component put below and answers it by hand, as skein-kv's own
step tests do, `crates/skein-kv/src/tests.rs`) and `src/tests/secrets.rs`
(memory). Fixtures added: `ROOT: Root` (token 900, fd 3),
`directory() -> (Store, Below)` (a component on `ROOT`, `Open` sent),
`answer(store: &mut Store, io: Io) -> (Box<[Event]>, Below)`,
`sign_in_secret(number, byte) -> Secrets` (a 32-byte value of `byte`).
The reference test of scripted io:

```rust
#[test]
fn opening_on_a_directory_makes_the_stores_directory_first() {
    let (mut store, mut below) = fixtures::directory();
    let Some(file::Request::MakeDirectory { owner, root, path, mode }) = below.store.pop() else {
        panic!("the store's directory is made first");
    };
    assert_eq!((root, path.as_ref(), mode), (Token::new(900), b"store".as_slice(), 0o700));
    let (events, mut below) = fixtures::answer(
        &mut store,
        Io::Store(Arrived { event: file::Event::Made { owner }, fd: None }),
    );
    assert!(events.is_empty());
    let Some(file::Request::OpenDirectory { name, no_follow: true, .. }) = below.store.pop() else {
        panic!("the store's directory is opened next");
    };
    assert_eq!(name.as_ref(), b"store");
}
```

Tests (`directory.rs`): `opening_on_a_directory_makes_the_stores_directory_first`,
`the_secrets_directory_is_opened_as_private`,
`skein_requests_go_below_on_their_stores_driver`,
`a_failed_skein_commit_on_disk_answers_failed`,
`a_recovery_beneath_the_store_stops_it`;
(`secrets.rs`): `a_kept_secret_is_answered_once_committed`,
`a_secret_is_got_by_owner_and_number`,
`an_erased_secret_gets_none`,
`a_load_pages_an_owners_secrets_in_number_order`,
`the_sweep_erases_a_digest_whose_sign_in_is_gone`,
`the_sweep_keeps_a_digest_whose_service_remains`,
`a_secrets_request_debug_shows_lengths_not_bytes`. Each under 20 ms.
Proof: unit, `secrets::the_sweep_erases_a_digest_whose_sign_in_is_gone`,
run once with `heavy cargo nextest run -p jig-protocol-store -E
'test(the_sweep_erases_a_digest_whose_sign_in_is_gone)'`: with sign-ins
4 and 5 kept as secrets and only sign-in 4 in the store, the core's
restore past people's families leaves sign-in 4's value and erases 5's;
under 20 ms. The stores on a crashing disk run in jig 18's
`tests/store`.
Don't: write a secret to the main store, its log or its snapshot
(store.md 9); answer `Kept` before the secrets' commit is answered; read
`FileIo`'s table from the component.
Size: at most 550 lines.
Removes: nothing.

### 17.2 Frames, places, and the write before the commit

Scope: store.md 8.1, 8.2 and 6. A commit with turn saves: for each turn,
in order, its frame at its task's end (the committed length from its
`Own::Transcript` place, plus the frames queued ahead for that task); the
file opened with `OpenWrite`, or, when `OpenWrite` answers missing, made
with `CreateNoFollow` beneath the transcripts' root (its `<high>`
directory made, and the transcripts' root synced, first, if absent); each
file written synced, and each directory that got a new file synced
(`SyncDirectory` on its descriptor); then the skein-kv commit, its
operations jig 15's without the turns' saves, plus the places: the
task's `Transcript { length, window }` moved past its frames, its
`window` moved to a frame whose turn opens a window, and an attempt's
`Attempt { first }` for a new attempt. Commits behind a commit with turns
wait in order, at most `Limits::queue`; `Committed` once skein-kv
answers. A failed write or sync answers that commit and each waiting
behind it `Failed`, and stops the component.
Needs: jig 06's `TurnRecord.opens_window`; jig 19's `codec::core::turn`
(19.2); jig 02's `OpenWrite`.
Files: `crates/jig-protocol-store/src/{transcripts.rs (new),
transcripts/{frame.rs, write.rs} (new), commits.rs, lib.rs, limits.rs
(jig 13's: `queue`, `files`)}`; `src/tests/{fixtures.rs, frames.rs
(new), transcripts.rs (new)}`; `tests/store-codecs/tests/fuzzy_decode.rs`
(jig 14's: the frame's decoder).
Shape:
- `frame::encode(turn: &core::TurnRecord, limits: &codec::Limits) ->
  Option<Box<[u8]>>`; `frame::decode(bytes: &[u8], limits: &codec::Limits)
  -> Result<core::TurnRecord, Unread>`; `frame::crc32c(bytes: &[u8]) ->
  u32`; `Unread::{Short, Lengths, Checksum, Record}`.
- `write::begin(store, number, writes, above, below)` holds the commit in
  `Pending::{Writing, Syncing, Ready}` until its files are synced;
  `write::arrived(store, arrived, above, below)`.
Test shape: `src/tests/frames.rs` (pure: encode and decode) and
`src/tests/transcripts.rs` (scripted io, as 17.1). Fixtures added:
`codec_limits()` (jig 19's `codec::limits` of the fixture limits),
`turn(task, attempt, number, bytes) -> core::TurnRecord` (no window),
`window_turn(task, attempt, number, bytes)`, `opened_with_transcripts()
-> (Store, Below)` (memory stores, `Open` answered, `ROOT` as the
transcripts' directory), `script(store, below, answers: &[Answer]) ->
Box<[Event]>` where `Answer::{Opened { len }, Missing, Made, Written,
Synced}` answers the next request on the transcripts' driver. The
reference test of frames:

```rust
#[test]
fn a_frame_decodes_to_the_turn_it_encoded() {
    let limits = fixtures::codec_limits();
    let turn = fixtures::window_turn(LIVE, 2, 3, b"smith's bytes");
    let bytes = frame::encode(&turn, &limits).expect("within the turn's limits");
    assert_eq!(frame::decode(&bytes, &limits), Ok(turn));
}
```

Tests (`frames.rs`): `a_frame_decodes_to_the_turn_it_encoded`,
`the_trailing_length_matches_the_leading_one`,
`a_flipped_byte_fails_the_frames_checksum`,
`a_frame_cut_short_reads_as_short`,
`crc32c_matches_its_check_value`; (`transcripts.rs`):
`a_turn_is_written_and_synced_before_its_commit`,
`a_turns_save_puts_no_turn_row_in_the_map`,
`a_missing_file_is_created_and_its_directory_synced`,
`a_second_turn_writes_past_the_first_still_queued`,
`a_window_turn_moves_the_tasks_window_in_the_commit`,
`a_commit_behind_a_turn_waits_its_turn`,
`a_failed_sync_fails_the_commit_and_those_behind`. Each under 20 ms.
Proof: unit,
`transcripts::a_turn_is_written_and_synced_before_its_commit`, run once
with `heavy cargo nextest run -p jig-protocol-store -E
'test(a_turn_is_written_and_synced_before_its_commit)'`: skein-kv
beneath the component has no commit 2 until the test answers the frame's
write and sync, and then has it with the task's place moved by the
frame's length; under 20 ms.
Don't: put a turn's bytes in the map; submit a commit with turns before
its files are synced; reorder commits.
Size: at most 550 lines.
Removes: jig 15.2's transitional turn rows.

### 17.3 Read back

Scope: store.md 8.3. `Request::Transcript { owner, read, rows, bytes }`
(jig 13's):
- `TranscriptRead::Window { task, attempt, turn, after }`: from the task's
  window offset; the frame there must be `(attempt, turn)`, else
  `Unread`; frames read forwards a page at a time within `rows` and
  `bytes`;
- `Turn { task, attempt, turn }`: from the attempt's first turn's offset,
  walking frames forwards to that number;
- `Page { task, backwards, after }`: from an offset cursor the component
  gave, forwards or backwards (by the trailing lengths); `backwards: true,
  after: None` is a fresh run's tail, handed up oldest first.
Each answered `Turns { owner, turns, next }` once its frames are read,
`next` an offset; never past the committed length; a frame whose checksum
or lengths fail, a task with no place, or a missing file: `Unread`. Reads
waiting on frames are at most `Limits::reads`, checked at start against
the core's `load_slots` and the root's preparation slots, so a read past
it is a contract break (`assert!`; Questions). Files opened with
`OpenRead` (a written one is read through its open token), within the
open files' set.
Needs: nothing beyond 17.2.
Files: `crates/jig-protocol-store/src/{transcripts/read.rs (new),
transcripts.rs, lib.rs, limits.rs (jig 13's: `reads`)}`;
`src/tests/{fixtures.rs, transcripts.rs}`.
Shape: `read::transcript(store, owner, read: TranscriptRead, rows, bytes,
above, below)`; `read::arrived(store, arrived, above, below)`; a table of
reads in flight keyed by owner, at most `Limits::reads`.
Test shape: `src/tests/transcripts.rs`; fixtures as 17.2, plus
`written(store, below, turns: &[core::TurnRecord])` (commits them through
17.2's path, answering the io), and `Answer::Read { bytes }`. Tests:
`a_window_read_starts_at_the_windows_first_turn`,
`a_window_named_at_another_turn_reads_as_unread`,
`a_turn_is_found_from_its_attempts_first_turn`,
`a_backwards_page_uses_the_trailing_lengths`,
`a_tail_page_comes_back_oldest_first`,
`a_read_never_passes_the_committed_length`,
`a_frame_failing_its_checksum_reads_as_unread`. Each under 20 ms.
Proof: unit, `transcripts::a_read_never_passes_the_committed_length`,
run once with `heavy cargo nextest run -p jig-protocol-store -E
'test(a_read_never_passes_the_committed_length)'`: with a frame written
past the committed length (a commit that never reached skein-kv), a
window read hands up the committed turns alone; under 20 ms.
Don't: read frames before a turn's attempt's first to find it; hand up a
frame past the committed length.
Size: at most 450 lines.
Removes: nothing.

### 17.4 After a crash, and dropped

Scope: store.md 8.4, 8.5 and 10.
- **The cut at open:** after 15.1's checks, the component reads the
  core's run proofs (the `CoreKey::RunProof` family, by jig 13's key
  encoding), and for each task in turn: its committed length from its
  place (zero without one); `OpenWrite`, `Cut { length }`, `Sync`,
  `Close`; a missing file is left missing. `Opened` once the last is
  synced.
- **Dropped:** `Write::DropTranscript { task }` (jig 13's): the task's
  `Own::Transcript` place and its `Own::Attempt` rows erased in the
  commit; the file removed (`Remove` on its directory's descriptor;
  missing is success) and the directory synced before the skein-kv
  commit; places left without a file (a crash before the commit) read as
  `Unread`.
Needs: jig 02's `Cut`.
Files: `crates/jig-protocol-store/src/{transcripts/{cut.rs, drop.rs}
(new), open.rs, commits.rs}`; `src/tests/transcripts.rs`.
Shape: `cut::begin(store, above, below)`, `cut::arrived(..)`;
`drop::begin(store, number, tasks: &[u64], ..)`.
Test shape: `src/tests/transcripts.rs`; fixtures as above, plus
`Answer::Cut`, `Answer::Removed`, `Answer::NotFound`, and jig 15's
`run_proof(task)`. Tests:
`open_cuts_a_claimed_tasks_file_to_its_committed_length`,
`open_cuts_a_file_without_places_to_zero`,
`open_leaves_a_missing_file_missing`,
`a_dropped_transcript_is_removed_before_its_commit`,
`dropping_a_missing_file_is_not_a_failure`,
`places_without_a_file_read_as_unread`. Each under 20 ms.
Proof: unit,
`transcripts::open_cuts_a_claimed_tasks_file_to_its_committed_length`,
run once with `heavy cargo nextest run -p jig-protocol-store -E
'test(open_cuts_a_claimed_tasks_file_to_its_committed_length)'`: a run
proof's task with a committed length of 63 and a file of 120 gets `Cut {
length: 63 }` and a sync before `Opened`; under 20 ms.
Don't: cut a file of a task without a run proof (store.md 8.4: "An ended
task's file was cut while its task lived, and is not touched again");
treat a missing file as a failure.
Size: at most 300 lines.
Removes: jig 15.2's `Failed` arm for `DropTranscript`.

### 17.5 Turns in parties' reads

Scope: store.md 7.3. `Scope::Turn { task, attempt, turn }` answered from
the transcript (17.3's turn walk) as a row of `CoreRecord::Turn` with its
owner task joined; `Scope::Reply` whose activity line is a `Turn {
attempt, turn }` answered with that turn as `related`. Such a read waits
for its frame in 17.3's table of reads; `Loaded::Busy` when the table is
full; a frame that reads as unread gives the row without `related` (a
reply) or an empty page (a turn).
Needs: jig 16's point scopes (`reads/point.rs`).
Files: `crates/jig-protocol-store/src/{reads/point.rs, reads.rs,
transcripts/read.rs}`; `src/tests/{reads.rs, transcripts.rs}`.
Shape: `point::turn` and `point::reply` start a transcript read and finish
the page in `read::arrived`.
Test shape: `src/tests/reads.rs`; fixtures as above. Tests:
`a_turn_read_answers_once_its_frame_is_read`,
`a_reply_that_was_a_turn_joins_that_turn`,
`a_turn_read_past_the_waiting_limit_is_busy`,
`an_unread_frame_gives_a_reply_without_its_turn`. Each under 20 ms.
Proof: unit, `reads::a_reply_that_was_a_turn_joins_that_turn`, run once
with `heavy cargo nextest run -p jig-protocol-store -E
'test(a_reply_that_was_a_turn_joins_that_turn)'`: a reply's activity
line naming attempt 1, turn 2 comes back with that turn's record as
`related`, after the test answers the frame's read; under 20 ms.
Don't: answer a turn's read before its frame is read.
Size: at most 200 lines.
Removes: jig 16's transitional arms (16.2): `Scope::Turn`'s empty page,
and a `Turn` reply's missing `related`.

## Interface sketches

jig 13's `Request::Transcript`, `TranscriptRead`, `Event::{Turns,
Unread, Kept, Erased, Secret}`, `SecretOwner`, `Secrets` and
`Write::DropTranscript` are used as its file sketches them. What this
session adds or changes:

```rust
// jig_protocol_store::boundary: this session's changes to jig 13's boundary, and the io face.
use skein_io::file;
use skein_io::kernel::Fd;
use skein_lib::{Queue, Secret, Token};

/// A root the shell opened and the service adopted: its token and descriptor (store.md 10).
#[derive(Clone, Copy, Debug)]
pub struct Root {
    pub token: Token,
    pub fd: Fd,
}
/// The component's file requests, one queue per driver (store.md 13).
pub struct Below {
    pub store: Queue<file::Request>,
    pub secrets: Queue<file::Request>,
    pub transcripts: Queue<file::Request>,
}
/// One file event, with the descriptor the driver holds for the file it opened (io.md 9).
pub struct Arrived {
    pub event: file::Event,
    pub fd: Option<Fd>,
}
/// A file event, by the driver it came from.
pub enum Io {
    Store(Arrived),
    Secrets(Arrived),
    Transcripts(Arrived),
}
/// jig 13's `SecretOwner`.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum SecretOwner {
    SignIn,
    Service,
    Application,
}
/// jig 13's `Secrets`, its values as `Secret`, with the load jig 31 asks at start.
#[derive(Debug)]
pub enum Secrets {
    Keep { owner: SecretOwner, number: u64, value: Secret },
    Erase { owner: SecretOwner, number: u64 },
    Get { owner: SecretOwner, number: u64 },
    /// One owner's secrets after `after`, a page at a time: `SecretsLoaded`.
    Load { owner: SecretOwner, after: Option<u64> },
}
/// One loaded secret.
#[derive(Debug)]
pub struct SecretRow {
    pub owner: SecretOwner,
    pub number: u64,
    pub value: Secret,
}
/// The events this session adds or changes (the rest of jig 13's `Event` unchanged).
#[derive(Debug)]
pub enum Event {
    /// A secret, or none (jig 13's, its value a `Secret`).
    Secret { owner: Token, value: Option<Secret> },
    /// A load's page; `next` is the number to go on after.
    SecretsLoaded { owner: Token, rows: Box<[SecretRow]>, next: Option<u64> },
}
/// `Request::Open`'s root becomes a `Root`; the rest of jig 13's `Request` unchanged.
pub enum Request {
    Open { owner: Token, root: Root },
}
```

```rust
// jig_protocol_store::transcripts::frame (this session): store.md 8.1.
use jig_domain_core as core;
pub mod codec {
    pub struct Limits; // jig 19's
}
/// Why a frame cannot be read.
#[derive(Debug, PartialEq, Eq)]
pub enum Unread {
    Short,
    Lengths,
    Checksum,
    Record,
}
/// The turn framed: its length, jig 19's encoding, its checksum, its length again.
pub fn encode(turn: &core::TurnRecord, limits: &codec::Limits) -> Option<Box<[u8]>> {
    todo!()
}
pub fn decode(bytes: &[u8], limits: &codec::Limits) -> Result<core::TurnRecord, Unread> {
    todo!()
}
/// CRC-32C (Castagnoli), bitwise; crc32c(b"123456789") is 0xE306_9283.
pub fn crc32c(bytes: &[u8]) -> u32 {
    let mut crc = !0_u32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = if crc & 1 == 0 { crc >> 1 } else { (crc >> 1) ^ 0x82F6_3B78 };
        }
    }
    !crc
}
```

Who owns what: the component owns its files and both stores; the service
owns the drivers and hands each event back with its descriptor; the
routing translates the core's `StartPreparation { from }` (jig 06) into a
`Window` read and `Turns` into the root's transcript pages; the secrets'
requests come from jig 31's clients' component, never from the root.

## Questions to raise when reached

- **A secrets' commit that fails** (17.1): jig 13's `Failed { number }`
  names a root commit; a secrets' commit has its own numbers (store.md 9:
  "the secrets' store's commits ... have numbers of their own"). This
  plan stops the component and answers the secrets' request nothing; if
  jig 31's component needs a terminal for it, raise which.
- **The sweep twice** (17.1): store.md 9 gives the sweep to this
  component; jig 31's sketch also sweeps (`Event::Restored { sign_ins }`
  in its component). Both erase the same digests; raise which keeps it.
- **What the root hears when a window reads as unread** (17.3): store.md
  8.3, "The core treats the transcript as one the agent cannot use: the
  run fails as transient and the next starts fresh"; the routing needs the
  core event jig 06 lands for it. If jig 06 landed none, raise it.
- **Transcript reads past the table** (17.3): jig 13's boundary has no
  busy answer for a transcript read; this plan sizes the table at start
  so none passes it. If the test root's unbounded preparation slots
  (`PreparedAgent { busy: false }`, `testing/jig-test-domain/src/routes.rs:364-366`)
  can pass it, raise a busy answer.
- **Syncing a new transcript's directory** (17.2): io names a root by its
  descriptor for `SyncDirectory` and `Remove` (io.md 9), which this plan
  hands over in `Arrived.fd`. If skein moves those requests to tokens
  first, take its form and drop `Arrived.fd`.

## Done when

1. The five increments are on `origin/main`: `git -C ~/src/rust/jig log
   origin/main --format=%b | grep -c '^Step: jig-17\.'` prints 5.
2. The files' tests pass: `heavy cargo nextest run -p jig-protocol-store
   -E 'test(/tests::(directory|secrets|frames|transcripts)::/)'` passes,
   running at least 30 tests.
3. No turn's bytes go to the map: `heavy cargo nextest run -p
   jig-protocol-store -E 'test(=tests::transcripts::a_turns_save_puts_no_turn_row_in_the_map)'`
   passes.
4. jig 16's transitional arms are gone: the test
   `reads::a_reply_that_was_a_turn_joins_that_turn` passes in jig's gate,
   whose `Gate-Result` trailer on 17.5's commit names `scripts/check.sh
   passed`.
