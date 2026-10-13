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
