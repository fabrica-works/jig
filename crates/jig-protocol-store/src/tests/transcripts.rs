//! Files synced before turns become durable (protocol/store.md, 8.1 and 8.2).
use super::fixtures::{self, Answer, LIVE};
use crate::{Arrived, Below, Event, Io, Phase, Store};
use alloc::boxed::Box;
use jig_domain_core::{CoreKey, CoreRecord, Key, Record, TurnRecord};
use skein_io::{file, kernel::Error};
use skein_lib::{Queue, Token};
fn submit(store: &mut Store, number: u64, turns: &[TurnRecord]) -> (Box<[Event]>, Below) {
    let mut writes =
        skein_lib::List::with_capacity(u32::try_from(turns.len()).expect("test turns").checked_add(1).expect("header"));
    writes.push(fixtures::header(number)).expect("header slot");
    for turn in turns {
        writes.push(fixtures::save(Record::Core(CoreRecord::Turn(turn.clone())))).expect("turn slot");
    }
    let mut above = Queue::with_capacity(crate::MAX_OUT_DOWN);
    let mut below = fixtures::below();
    crate::down(store, &fixtures::env(), fixtures::commit(number, writes.into_boxed()), &mut above, &mut below);
    (fixtures::events(&mut above), below)
}
fn writing() -> (Store, Below) {
    let (mut store, _) = fixtures::opened_with_transcripts();
    let (events, below) = submit(&mut store, 2, &[fixtures::turn(LIVE, 1, 1, b"one")]);
    assert!(events.is_empty());
    (store, below)
}
fn turn_bytes(store: &mut Store, mut below: Below) -> (Box<[u8]>, Below) {
    let Some(file::Request::OpenWriteNoFollow { owner, .. }) = below.transcripts.pop() else {
        panic!("open transcript");
    };
    let (_, below) = fixtures::answer(
        store,
        Io::Transcripts(Arrived { event: file::Event::Opened { owner, file: Token::new(700), len: 0 }, fd: None }),
    );
    let (bytes, offset, below) = written_frame(store, below);
    assert_eq!(offset, 0);
    (bytes, below)
}
fn written_frame(store: &mut Store, mut below: Below) -> (Box<[u8]>, u64, Below) {
    let Some(file::Request::WriteAt { owner, offset, bytes, .. }) = below.transcripts.pop() else {
        panic!("frame write");
    };
    let (events, below) =
        fixtures::answer(store, Io::Transcripts(Arrived { event: file::Event::Written { owner }, fd: None }));
    assert!(events.is_empty());
    (bytes, offset, below)
}
fn place(store: &mut Store) -> crate::codec::component::Transcript {
    let key =
        crate::keys::encode_own(&crate::keys::Own::Transcript { task: LIVE }, fixtures::KEY_LIMIT).expect("place key");
    let bytes = fixtures::get_kv(store, &key).expect("durable place");
    crate::codec::component::Transcript::decode(&bytes).expect("place decodes")
}
fn finish(store: &mut Store, below: Below) {
    let events = fixtures::script(store, below, &[Answer::Opened { len: 0 }, Answer::Written, Answer::Synced]);
    let [Event::Committed { number: 2, .. }] = &*events else {
        panic!("durable second commit: {events:?}");
    };
}
#[test]
fn a_turn_is_written_and_synced_before_its_commit() {
    let (mut store, below) = writing();
    assert_eq!(store.stores.store.last(), 1);
    let (bytes, below) = turn_bytes(&mut store, below);
    assert_eq!(store.stores.store.last(), 1);
    let events = fixtures::script(&mut store, below, &[Answer::Synced]);
    let [Event::Committed { number: 2, .. }] = &*events else {
        panic!("commit after frame sync");
    };
    assert_eq!(store.stores.store.last(), 2);
    assert_eq!(place(&mut store).length, u64::try_from(bytes.len()).expect("frame length"));
}
#[test]
fn a_turns_save_puts_no_turn_row_in_the_map() {
    let (mut store, below) = writing();
    finish(&mut store, below);
    let key = crate::keys::encode(&Key::Core(CoreKey::Turn { task: LIVE, attempt: 1, turn: 1 }), fixtures::KEY_LIMIT)
        .expect("turn key");
    assert_eq!(fixtures::get_kv(&mut store, &key), None);
}
#[test]
fn a_turn_is_saved_as_a_row_until_transcript_files_land() {
    let (mut store, below) = writing();
    let (bytes, below) = turn_bytes(&mut store, below);
    let expected = fixtures::turn(LIVE, 1, 1, b"one");
    assert_eq!(crate::transcripts::frame::decode(&bytes, &fixtures::codec_limits()), Ok(expected));
    let events = fixtures::script(&mut store, below, &[Answer::Synced]);
    let [Event::Committed { number: 2, .. }] = &*events else {
        panic!("framed turn durable");
    };
}
#[test]
fn a_missing_file_is_created_and_its_directory_synced() {
    let (mut store, below) = writing();
    let events = fixtures::script(
        &mut store,
        below,
        &[
            Answer::Missing,
            Answer::Made,
            Answer::Synced,
            Answer::Opened { len: 0 },
            Answer::Opened { len: 0 },
            Answer::Written,
            Answer::Synced,
            Answer::Synced,
            Answer::Closed,
        ],
    );
    let [Event::Committed { number: 2, .. }] = &*events else {
        panic!("created frame durable");
    };
    assert!(place(&mut store).length > 0);
}
#[test]
fn a_second_turn_writes_past_the_first_still_queued() {
    let (mut store, below) = writing();
    let (bytes, mut below) = turn_bytes(&mut store, below);
    assert!(submit(&mut store, 3, &[fixtures::turn(LIVE, 1, 2, b"two")]).0.is_empty());
    let Some(file::Request::Sync { owner, .. }) = below.transcripts.pop() else {
        panic!("first sync");
    };
    let (_, mut below) =
        fixtures::answer(&mut store, Io::Transcripts(Arrived { event: file::Event::Synced { owner }, fd: None }));
    let Some(file::Request::WriteAt { offset, .. }) = below.transcripts.pop() else {
        panic!("second frame");
    };
    assert_eq!(offset, u64::try_from(bytes.len()).expect("first frame length"));
}
#[test]
fn a_window_turn_moves_the_tasks_window_in_the_commit() {
    let (mut store, below) = writing();
    let (bytes, below) = turn_bytes(&mut store, below);
    fixtures::script(&mut store, below, &[Answer::Synced]);
    let (_, below) = submit(&mut store, 3, &[fixtures::window_turn(LIVE, 2, 1, b"window")]);
    let (_, _, below) = written_frame(&mut store, below);
    assert_eq!(place(&mut store).window, 0);
    fixtures::script(&mut store, below, &[Answer::Synced]);
    assert_eq!(place(&mut store).window, u64::try_from(bytes.len()).expect("first frame length"));
}
#[test]
fn a_commit_behind_a_turn_waits_its_turn() {
    let (mut store, below) = writing();
    assert!(submit(&mut store, 3, &[]).0.is_empty());
    assert_eq!(store.stores.store.last(), 1);
    let events = fixtures::script(&mut store, below, &[Answer::Opened { len: 0 }, Answer::Written, Answer::Synced]);
    let [Event::Committed { number: 2, .. }, Event::Committed { number: 3, .. }] = &*events else {
        panic!("commits retain their order: {events:?}");
    };
}
#[test]
fn a_failed_sync_fails_the_commit_and_those_behind() {
    let (mut store, below) = writing();
    assert!(submit(&mut store, 3, &[]).0.is_empty());
    let (_, mut below) = turn_bytes(&mut store, below);
    let Some(file::Request::Sync { owner, .. }) = below.transcripts.pop() else {
        panic!("sync frame");
    };
    let (events, _) = fixtures::answer(
        &mut store,
        Io::Transcripts(Arrived {
            event: file::Event::Failed { owner, error: Error::Permission, committed: false, residue: None },
            fd: None,
        }),
    );
    let [Event::Failed { number: 2 }, Event::Failed { number: 3 }] = &*events else {
        panic!("failed ordered commits: {events:?}");
    };
    assert_eq!(store.stores.store.last(), 1);
    assert_eq!(store.phase, Phase::Stopped);
}
fn two_open_transcripts() -> Store {
    let (mut store, _) = fixtures::opened_with_transcripts();
    let (_, below) = submit(&mut store, 2, &[fixtures::turn(LIVE, 1, 1, b"one"), fixtures::turn(22, 1, 1, b"two")]);
    fixtures::script(
        &mut store,
        below,
        &[
            Answer::Opened { len: 0 },
            Answer::Written,
            Answer::Synced,
            Answer::Opened { len: 0 },
            Answer::Written,
            Answer::Synced,
        ],
    );
    store
}
#[test]
fn a_full_file_set_closes_the_file_written_longest_ago() {
    let mut store = two_open_transcripts();
    let (_, below) = submit(&mut store, 3, &[fixtures::turn(LIVE, 1, 2, b"recent")]);
    fixtures::script(&mut store, below, &[Answer::Written, Answer::Synced]);
    let (_, mut below) = submit(&mut store, 4, &[fixtures::turn(23, 1, 1, b"third")]);
    let Some(file::Request::Close { file, .. }) = below.transcripts.pop() else {
        panic!("close before another open");
    };
    assert_eq!(file, Token::new(1_003));
    assert_eq!(store.transcripts.files.len(), 2);
}
#[test]
fn a_new_attempt_keeps_its_first_turns_offset() {
    let (mut store, below) = writing();
    let (bytes, below) = turn_bytes(&mut store, below);
    fixtures::script(&mut store, below, &[Answer::Synced]);
    let (_, below) = submit(&mut store, 3, &[fixtures::turn(LIVE, 2, 1, b"new"), fixtures::turn(LIVE, 2, 2, b"next")]);
    fixtures::script(&mut store, below, &[Answer::Written, Answer::Synced, Answer::Written, Answer::Synced]);
    let key = crate::keys::encode_own(&crate::keys::Own::Attempt { task: LIVE, attempt: 2 }, fixtures::KEY_LIMIT)
        .expect("attempt key");
    let value = fixtures::get_kv(&mut store, &key).expect("new attempt place");
    assert_eq!(
        crate::codec::component::Attempt::decode(&value).expect("attempt decodes").first,
        u64::try_from(bytes.len()).expect("first frame length")
    );
}
#[test]
fn the_transcript_queue_refuses_before_a_later_commit_is_submitted() {
    let (mut store, below) = writing();
    assert!(submit(&mut store, 3, &[]).0.is_empty());
    assert!(submit(&mut store, 4, &[]).0.is_empty());
    let (events, _) = submit(&mut store, 5, &[]);
    let [
        Event::Failed { number: 5 },
        Event::Failed { number: 2 },
        Event::Failed { number: 3 },
        Event::Failed { number: 4 },
    ] = &*events
    else {
        panic!("all unsubmitted commits fail: {events:?}");
    };
    assert!(!below.transcripts.is_empty());
    assert_eq!(store.stores.store.last(), 1);
    assert_eq!(store.phase, Phase::Stopped);
}

fn readable(turns: &[TurnRecord]) -> (Store, Box<[u8]>) {
    let (mut store, _) = fixtures::opened_with_transcripts();
    let (_, below) = submit(&mut store, 2, turns);
    let disk = fixtures::written(&mut store, below, turns);
    (store, disk)
}
fn ask(store: &mut Store, owner: u64, read: crate::TranscriptRead, rows: u32, bytes: u64) -> (Box<[Event]>, Below) {
    let mut above = Queue::with_capacity(crate::MAX_OUT_DOWN);
    let mut below = fixtures::below();
    crate::down(
        store,
        &fixtures::env(),
        crate::Request::Transcript { owner: Token::new(owner), read, rows, bytes },
        &mut above,
        &mut below,
    );
    (fixtures::events(&mut above), below)
}
fn read_answer(store: &mut Store, mut below: Below, disk: &[u8], chunk: u32) -> (Box<[Event]>, Below) {
    let request = below.transcripts.pop().expect("transcript io request");
    let event = match request {
        file::Request::OpenRead { owner, .. } => {
            file::Event::Opened { owner, file: Token::new(701), len: u64::try_from(disk.len()).expect("disk length") }
        }
        file::Request::ReadAt { owner, offset, max, .. } => {
            let start = usize::try_from(offset).expect("fixture offset");
            let count = usize::try_from(max.min(chunk)).expect("fixture count");
            let end = start.checked_add(count).expect("fixture read end").min(disk.len());
            let bytes = Box::from(disk.get(start..end).expect("read within fixture disk"));
            file::Event::Read { owner, bytes }
        }
        file::Request::MakeDirectory { .. }
        | file::Request::OpenPrivate { .. }
        | file::Request::Create { .. }
        | file::Request::CreateNoFollow { .. }
        | file::Request::OpenReadNoFollow { .. }
        | file::Request::OpenWrite { .. }
        | file::Request::OpenWriteNoFollow { .. }
        | file::Request::OpenDirectory { .. }
        | file::Request::Load { .. }
        | file::Request::Scan { .. }
        | file::Request::Store { .. }
        | file::Request::Stat { .. }
        | file::Request::WriteAt { .. }
        | file::Request::Cut { .. }
        | file::Request::Sync { .. }
        | file::Request::Close { .. }
        | file::Request::SyncDirectory { .. }
        | file::Request::Rename { .. }
        | file::Request::Remove { .. }
        | file::Request::List { .. } => panic!("expected transcript read"),
    };
    fixtures::answer(store, Io::Transcripts(Arrived { event, fd: None }))
}
fn answer_reads(store: &mut Store, mut below: Below, disk: &[u8], chunk: u32) -> Box<[Event]> {
    for _ in 0_u32..4096 {
        let (events, next) = read_answer(store, below, disk, chunk);
        if !events.is_empty() {
            assert!(next.transcripts.is_empty());
            return events;
        }
        below = next;
    }
    panic!("read must settle within fixture bound");
}
fn appended(disk: &[u8], pending: &[u8]) -> Box<[u8]> {
    let mut extended = skein_lib::Writer::new(disk.len().checked_add(pending.len()).expect("fixture disk"));
    extended.put(disk).expect("committed frames");
    extended.put(pending).expect("pending frame");
    extended.finish()
}
fn reopen(store: Store) -> Store {
    let mut store = fixtures::over(store.into_stores());
    fixtures::run(&mut store, crate::Request::Open { owner: Token::new(800), root: fixtures::ROOT });
    store
}
fn unread(events: &[Event]) {
    let [Event::Unread { owner }] = events else {
        panic!("unread terminal: {events:?}");
    };
    assert_eq!(*owner, Token::new(801));
}
fn page(events: &[Event]) -> (Token, &[TurnRecord], Option<u64>) {
    let [Event::Turns { owner, turns, next }] = events else {
        panic!("turn page: {events:?}");
    };
    (*owner, turns, *next)
}
fn read_turns(
    store: &mut Store,
    disk: &[u8],
    read: crate::TranscriptRead,
    rows: u32,
) -> (Box<[TurnRecord]>, Option<u64>) {
    let (events, below) = ask(store, 801, read, rows, 4096);
    assert!(events.is_empty());
    let events = answer_reads(store, below, disk, 4096);
    let (owner, turns, next) = page(&events);
    assert_eq!(owner, Token::new(801));
    (Box::from(turns), next)
}
fn window(after: Option<u64>) -> crate::TranscriptRead {
    crate::TranscriptRead::Window { task: LIVE, attempt: 2, turn: 1, after }
}
fn forwards(after: Option<u64>) -> crate::TranscriptRead {
    crate::TranscriptRead::Page { task: LIVE, backwards: false, after }
}
fn backwards(after: Option<u64>) -> crate::TranscriptRead {
    crate::TranscriptRead::Page { task: LIVE, backwards: true, after }
}
fn pair() -> [TurnRecord; 2] {
    [fixtures::window_turn(LIVE, 2, 1, b"one"), fixtures::turn(LIVE, 2, 2, b"two")]
}
#[test]
fn a_window_read_starts_at_the_windows_first_turn() {
    let turns = [
        fixtures::turn(LIVE, 1, 1, b"old"),
        fixtures::window_turn(LIVE, 2, 1, b"window"),
        fixtures::turn(LIVE, 2, 2, b"next"),
    ];
    let (mut store, disk) = readable(&turns);
    let (found, next) = read_turns(&mut store, &disk, window(None), 2);
    assert_eq!(&*found, turns.get(1..).expect("window turns"));
    assert_eq!(next, None);
}
#[test]
fn a_window_named_at_another_turn_reads_as_unread() {
    let (mut store, disk) = readable(&pair());
    let (events, below) =
        ask(&mut store, 801, crate::TranscriptRead::Window { task: LIVE, attempt: 2, turn: 2, after: None }, 2, 4096);
    assert!(events.is_empty());
    let events = answer_reads(&mut store, below, &disk, 4096);
    unread(&events);
}
#[test]
fn a_turn_is_found_from_its_attempts_first_turn() {
    let turns = [
        fixtures::turn(LIVE, 1, 1, b"old"),
        fixtures::turn(LIVE, 2, 1, b"first"),
        fixtures::turn(LIVE, 2, 2, b"found"),
    ];
    let (mut store, disk) = readable(&turns);
    let (_, below) = ask(&mut store, 801, crate::TranscriptRead::Turn { task: LIVE, attempt: 2, turn: 2 }, 2, 4096);
    let request = below.transcripts.iter().next().expect("first read");
    let file::Request::ReadAt { offset, .. } = request else {
        panic!("read attempt start");
    };
    assert_eq!(*offset, 67);
    let events = answer_reads(&mut store, below, &disk, 4096);
    let (_, found, next) = page(&events);
    assert_eq!(next, None);
    assert_eq!(found, turns.get(2..).expect("named turn"));
}
#[test]
fn a_backwards_page_uses_the_trailing_lengths() {
    let turns = pair();
    let (mut store, disk) = readable(&turns);
    let (found, next) = read_turns(&mut store, &disk, backwards(None), 1);
    assert_eq!(&*found, turns.get(1..).expect("last turn"));
    let (found, next) = read_turns(&mut store, &disk, backwards(next), 1);
    assert_eq!(&*found, turns.get(..1).expect("first turn"));
    assert_eq!(next, None);
}
#[test]
fn a_tail_page_comes_back_oldest_first() {
    let turns = pair();
    let (mut store, disk) = readable(&turns);
    assert_eq!(read_turns(&mut store, &disk, backwards(None), 2), (Box::from(turns), None));
}
#[test]
fn a_read_never_passes_the_committed_length() {
    let turns = [fixtures::window_turn(LIVE, 2, 1, b"one")];
    let (mut store, mut disk) = readable(&turns);
    let (_, below) = submit(&mut store, 3, &[fixtures::turn(LIVE, 2, 2, b"uncommitted")]);
    let (pending, _, _sync) = written_frame(&mut store, below);
    disk = appended(&disk, &pending);
    // Crash before the pending frame's sync and commit, retaining the durable map.
    store = reopen(store);
    let (events, below) = ask(&mut store, 801, window(None), 2, 4096);
    assert!(events.is_empty());
    let events = answer_reads(&mut store, below, &disk, 4096);
    let (_, found, next) = page(&events);
    assert_eq!(next, None);
    assert_eq!(found, &turns);
    assert_eq!(store.stores.store.last(), 2);
}
#[test]
fn a_frame_failing_its_checksum_reads_as_unread() {
    let (mut store, mut disk) = readable(&pair());
    *disk.get_mut(4).expect("first body byte") ^= 1;
    let (_, below) = ask(&mut store, 801, forwards(None), 2, 4096);
    let events = answer_reads(&mut store, below, &disk, 4096);
    unread(&events);
}
#[test]
fn a_window_page_continues_from_its_offset_cursor() {
    let turns = pair();
    let (mut store, disk) = readable(&turns);
    let (first, next) = read_turns(&mut store, &disk, window(None), 1);
    assert_eq!(&*first, turns.get(..1).expect("first turn"));
    assert_eq!(next, Some(67));
    let (second, next) = read_turns(&mut store, &disk, window(next), 1);
    assert_eq!(&*second, turns.get(1..).expect("second turn"));
    assert_eq!(next, None);
}
#[test]
fn a_forward_page_stops_within_its_byte_budget() {
    let turns = pair();
    let (mut store, disk) = readable(&turns);
    let (_, below) = ask(&mut store, 801, forwards(None), 2, 67);
    let events = answer_reads(&mut store, below, &disk, 4096);
    let (_, found, next) = page(&events);
    assert_eq!(found, turns.get(..1).expect("first turn"));
    assert_eq!(next, Some(67));
}
#[test]
fn a_short_read_is_filled_before_the_frame_is_decoded() {
    let turns = pair();
    let (mut store, disk) = readable(&turns);
    let (_, below) = ask(&mut store, 801, window(None), 2, 4096);
    let events = answer_reads(&mut store, below, &disk, 3);
    let (_, found, next) = page(&events);
    assert_eq!(next, None);
    assert_eq!(found, &turns);
}
#[test]
fn a_frame_with_a_length_past_the_committed_end_is_unread() {
    let (mut store, mut disk) = readable(&pair());
    *disk.get_mut(3).expect("leading length low byte") = 200;
    let (_, below) = ask(&mut store, 801, forwards(None), 2, 4096);
    let events = answer_reads(&mut store, below, &disk, 4096);
    unread(&events);
}
#[test]
fn a_transcript_without_a_place_is_unread() {
    let mut store = fixtures::opened();
    let (events, below) = ask(&mut store, 801, forwards(None), 2, 4096);
    unread(&events);
    assert!(below.transcripts.is_empty());
}
#[test]
fn a_missing_transcript_file_is_unread() {
    let (store, _) = readable(&pair());
    let mut store = reopen(store);
    let (_, below) = ask(&mut store, 801, forwards(None), 2, 4096);
    let events = fixtures::script(&mut store, below, &[Answer::Missing]);
    unread(&events);
}
#[test]
fn a_read_queued_behind_another_owner_gets_its_own_page() {
    let turns = pair();
    let (mut store, disk) = readable(&turns);
    let (_, below) = ask(&mut store, 801, forwards(None), 1, 4096);
    let (events, waiting) = ask(&mut store, 802, backwards(None), 1, 4096);
    assert!(events.is_empty());
    assert!(waiting.transcripts.is_empty());
    // First terminal starts the waiting owner's read in the same up step.
    let mut below = below;
    for _ in 0_u32..2 {
        let (events, next) = read_answer(&mut store, below, &disk, 4096);
        if !events.is_empty() {
            assert_eq!(page(&events).0, Token::new(801));
        }
        below = next;
    }
    let events = answer_reads(&mut store, below, &disk, 4096);
    let (owner, found, _) = page(&events);
    assert_eq!(owner, Token::new(802));
    assert_eq!(found, turns.get(1..).expect("tail turn"));
}
#[test]
fn writing_reopens_a_file_that_was_opened_for_reading() {
    let (store, disk) = readable(&pair());
    let mut store = reopen(store);
    read_turns(&mut store, &disk, forwards(None), 2);
    let (_, mut below) = submit(&mut store, 3, &[fixtures::turn(LIVE, 2, 3, b"new")]);
    let Some(file::Request::Close { file, .. }) = below.transcripts.pop() else {
        panic!("close read token");
    };
    assert_eq!(file, Token::new(701));
    below.transcripts.push(file::Request::Close { owner: Token::new(0x5452), file });
    let events = fixtures::script(
        &mut store,
        below,
        &[Answer::Closed, Answer::Opened { len: 134 }, Answer::Written, Answer::Synced],
    );
    let [Event::Committed { number: 3, .. }] = &*events else {
        panic!("write after read token closes");
    };
}
#[test]
fn a_read_script_delivers_one_complete_frame() {
    let turns = pair();
    let (mut store, disk) = readable(&turns);
    let (_, below) = ask(&mut store, 801, forwards(None), 1, 4096);
    let events = fixtures::script(
        &mut store,
        below,
        &[
            Answer::Read { bytes: Box::new([0, 0, 0, 67]) },
            Answer::Read { bytes: Box::from(disk.get(..67).expect("first frame")) },
        ],
    );
    let (_, found, next) = page(&events);
    assert_eq!(next, Some(67));
    assert_eq!(found, turns.get(..1).expect("first turn"));
}
#[test]
#[should_panic(expected = "startup bounds cover transcript reads")]
fn a_read_past_the_bounded_table_is_a_contract_break() {
    let (mut store, _) = readable(&pair());
    for owner in 801..806 {
        ask(&mut store, owner, forwards(None), 1, 4096);
    }
}
#[test]
fn a_window_with_no_row_room_still_checks_its_named_start() {
    let (mut store, disk) = readable(&pair());
    let (_, below) =
        ask(&mut store, 801, crate::TranscriptRead::Window { task: LIVE, attempt: 2, turn: 2, after: None }, 0, 4096);
    let events = answer_reads(&mut store, below, &disk, 4096);
    unread(&events);
}
#[test]
fn a_full_file_set_keeps_the_file_read_more_recently() {
    let mut store = two_open_transcripts();
    let turn = fixtures::turn(LIVE, 1, 1, b"one");
    let disk = crate::transcripts::frame::encode(&turn, &fixtures::codec_limits()).expect("fixture frame");
    read_turns(&mut store, &disk, forwards(None), 1);
    let (_, mut below) = submit(&mut store, 3, &[fixtures::turn(23, 1, 1, b"third")]);
    let Some(file::Request::Close { file, .. }) = below.transcripts.pop() else {
        panic!("bounded file set closes");
    };
    assert_eq!(file, Token::new(1_003));
    assert_eq!(store.transcripts.files.len(), 2);
}
#[test]
fn a_read_waiting_on_a_writer_keeps_its_durable_places() {
    let turns = [fixtures::window_turn(LIVE, 2, 1, b"one")];
    let (mut store, disk) = readable(&turns);
    let (_, below) = submit(&mut store, 3, &[fixtures::turn(LIVE, 2, 2, b"pending")]);
    let (pending, _, mut sync) = written_frame(&mut store, below);
    let disk = appended(&disk, &pending);
    let (events, waiting) = ask(&mut store, 801, window(None), 2, 4096);
    assert!(events.is_empty());
    assert!(waiting.transcripts.is_empty());
    let Some(file::Request::Sync { owner, .. }) = sync.transcripts.pop() else {
        panic!("pending writer sync");
    };
    let (_, below) =
        fixtures::answer(&mut store, Io::Transcripts(Arrived { event: file::Event::Synced { owner }, fd: None }));
    assert_eq!(store.stores.store.last(), 3);
    let events = answer_reads(&mut store, below, &disk, 4096);
    let (_, found, next) = page(&events);
    assert_eq!(found, &turns);
    assert_eq!(next, None);
}
