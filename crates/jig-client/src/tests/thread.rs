use super::fixtures::{self, thread::*};
use crate::{
    contract::*,
    thread::{Line, Thread, Words, window::Edge},
};
use skein_lib::{List, Wall};
#[test]
fn activity_reads_newest_then_older_and_keeps_reverse_cursor_when_sliding() {
    let mut c = Thread::new(9, &crate::thread::Limits { lines: 3, bytes: 65536, text: 4096 });
    assert_eq!(c.moved(6), Some(Read::Activity { task: 9, before: None }));
    assert_eq!(c.moved(6), None);
    assert_eq!(c.accept(page(&[6, 5, 4], Some(b"older"), None)), None);
    assert_eq!(positions(&c).as_ref(), &[6, 5, 4]);
    assert_eq!(c.older(), Some(Read::Activity { task: 9, before: Some(cursor(b"older")) }));
    assert_eq!(c.accept(page(&[3, 2, 1], None, Some(b"newer"))), None);
    assert_eq!(positions(&c).as_ref(), &[3, 2, 1]);
    assert_eq!(c.lines.edges.before, Edge::More(cursor(b"newer")));
    assert_eq!(c.older(), None);
    assert_eq!(c.moved(7), None);
    assert_eq!(c.newer(), Some(Read::Activity { task: 9, before: Some(cursor(b"newer")) }));
    assert_eq!(c.accept(page(&[6, 5, 4], Some(b"older"), None)), Some(Read::Activity { task: 9, before: None }));
    assert_eq!(c.accept(page(&[7, 6, 5], Some(b"older"), None)), None);
    assert_eq!(positions(&c).as_ref(), &[7, 6, 5]);
}
#[test]
fn activity_document_movement_reads_only_new_lines_and_bridges_small_pages() {
    let mut c = Thread::new(9, &crate::thread::Limits { lines: 4, bytes: 65536, text: 4096 });
    assert!(c.moved(2).is_some());
    assert_eq!(c.accept(page(&[2, 1], None, None)), None);
    assert_eq!(c.moved(2), None);
    assert!(c.moved(5).is_some());
    assert_eq!(
        c.accept(page(&[5, 4], Some(b"tail"), None)),
        Some(Read::Activity { task: 9, before: Some(cursor(b"tail")) })
    );
    assert_eq!(c.accept(page(&[3, 2], Some(b"past"), Some(b"reverse"))), None);
    assert_eq!(positions(&c).as_ref(), &[5, 4, 3, 2]);
    assert_eq!(c.loaded, 5);
    assert_eq!(c.moved(5), None);
    assert!(c.moved(6).is_some());
    c.failed();
    assert!(c.moved(6).is_some());
    assert_eq!(
        c.accept(page(&[11, 10], Some(b"remaining"), None)),
        Some(Read::Activity { task: 9, before: Some(cursor(b"remaining")) })
    );
    assert_eq!(c.accept(page(&[9, 8, 7], Some(b"overflow"), Some(b"back"))), None);
    assert_eq!(positions(&c).as_ref(), &[11, 10]);
    assert_eq!(c.lines.edges.after, Edge::More(cursor(b"remaining")));
}

const FIRST_WORDS: crate::Key = crate::Key { asker: crate::Asker::JIG, number: 11 };
const SECOND_WORDS: crate::Key = crate::Key { asker: crate::Asker::JIG, number: 12 };
const FULL_WORDS: crate::Key = crate::Key { asker: crate::Asker::JIG, number: 13 };
const ARCHIVE_WORDS: crate::Key = crate::Key { asker: crate::Asker::JIG, number: 77 };
const ATTEMPT: u64 = 1;
const TURN: u32 = 1;
const MESSAGE: u64 = 41;
const FIRST_MESSAGE: u64 = 40;
const TURN_AT: Wall = Wall::from_nanos(17);
const FIRST_WORDS_AT: Wall = Wall::from_nanos(11);
const SECOND_WORDS_AT: Wall = Wall::from_nanos(13);
fn run() -> RunState {
    RunState { task: fixtures::SUBJECT_TASK, attempt: Some(ATTEMPT), committed_turn: 0, writing: None, call: None }
}
fn provisional(number: u32, words: &[u8]) -> RunText {
    RunText {
        attempt: ATTEMPT,
        began: None,
        turn: ProvisionalTurnName { attempt: ATTEMPT, number, retry: 0 },
        from: 0,
        through: u32::try_from(words.len()).unwrap(),
        text: text(words),
        cut: false,
        call: None,
    }
}
fn commit(thread: &mut Thread, turn: TurnRead) {
    let mut page = page(&[], None, None);
    page.rows
        .push(ActivityLine {
            position: 1,
            at: TURN_AT,
            event: Activity::Turn { name: TurnName { attempt: ATTEMPT, number: TURN }, turn },
        })
        .unwrap();
    assert!(thread.moved(1).is_some());
    assert_eq!(thread.page(page), None);
}
#[test]
fn a_committed_turn_replaces_its_provisional_text() {
    let mut thread = model();
    thread.run(&run(), Some(&provisional(TURN, b"first")));
    thread.run(&run(), Some(&provisional(TURN + 1, b"second")));
    assert_eq!(thread.live.len(), 2);
    let turn = Turn {
        messages: List::with_capacity(0),
        used: Usage { input: 0, output: 0, cached: 0, cache_written: 0 },
        spent: 0,
        read_through: None,
        cut: false,
    };
    commit(&mut thread, TurnRead::Decoded(turn.clone()));
    assert_eq!(thread.live.len(), 1);
    let lines = thread.lines();
    assert_eq!(
        lines.as_slice(),
        &[
            Line::Turn { name: TurnName { attempt: ATTEMPT, number: TURN }, turn: &turn, at: TURN_AT },
            Line::Provisional { name: provisional(TURN + 1, b"").turn, text: b"second", cut: false }
        ]
    );
}
#[test]
fn a_turn_that_does_not_decode_stays_a_line_saying_why() {
    let mut thread = model();
    commit(&mut thread, TurnRead::Undecodable { why: TurnProblem::Version { found: 99 } });
    let lines = thread.lines();
    assert_eq!(
        lines.as_slice(),
        &[Line::Undecoded {
            name: TurnName { attempt: ATTEMPT, number: TURN },
            why: TurnProblem::Version { found: 99 },
            at: TURN_AT
        }]
    );
}
#[test]
fn words_sent_in_a_compound_are_numbered_by_their_place() {
    let mut thread = model();
    assert!(thread.sending(fixtures::STOP, fixtures::PARTY, text(b"second operation"), 1));
    let by = thread.words.rows.get(0).unwrap().by.as_ref();
    assert_eq!(
        thread.lines().as_slice(),
        &[Line::Words { by, words: b"second operation", state: Words::BeingSent, at: None }]
    );
    thread.answered(
        fixtures::STOP,
        &Answer::Done(Outcome::Owner(OwnerOutcome::Compound {
            project: fixtures::PROJECT,
            operations: 2,
            first_message: FIRST_MESSAGE,
        })),
    );
    assert_eq!(thread.words.rows.get(0).unwrap().state, Words::Sent { message: MESSAGE });
    let by = thread.words.rows.get(0).unwrap().by.as_ref();
    assert_eq!(
        thread.lines().as_slice(),
        &[Line::Words { by, words: b"second operation", state: Words::Sent { message: MESSAGE }, at: None }]
    );
}

#[test]
fn words_keep_the_submitted_snapshot_and_restore_sent_messages_until_read() {
    use crate::thread::{KeptWords, Words};
    let mut words = KeptWords::new(2);
    let a = FIRST_WORDS;
    let b = SECOND_WORDS;
    assert!(words.begin(a, None, text(b"first")));
    assert!(words.begin(a, None, text(b"later edit")));
    assert_eq!(words.rows.get(0).unwrap().words, text(b"first"));
    assert_eq!(words.rows.get(0).unwrap().state, Words::BeingSent);
    assert!(words.begin(b, None, text(b"second")));
    assert!(!words.begin(FULL_WORDS, None, text(b"full")));
    words.read(1);
    assert!(words.sent(a, 1));
    assert!(words.sent(b, 2));
    assert_eq!(words.rows.get(0).unwrap().state, Words::Read { message: 1 });
    assert_eq!(words.rows.get(1).unwrap().state, Words::Sent { message: 2 });
    words.read(2);
    assert_eq!(words.rows.get(1).unwrap().state, Words::Read { message: 2 });
    assert!(words.admits());
    assert_eq!(words.rows.len(), 0);
    words.bind(Some(7));
    assert!(words.begin(a, None, text(b"private pending")));
    words.bind(Some(8));
    assert!(words.rows.is_empty());
    let mut joined = KeptWords::new(2);
    assert!(joined.begin(a, None, text(b"original")));
    assert!(joined.restore(party(7), 3, text(b"activity copy")));
    assert!(joined.sent(a, 3));
    assert_eq!(joined.rows.len(), 1);
    assert_eq!(joined.rows.get(0).unwrap().words, text(b"original"));
    let mut reload = Thread::new(8, &crate::thread::Limits { lines: 3, bytes: 65536, text: 4096 });
    assert!(reload.moved(2).is_some());
    assert_eq!(reload.accept(words_page()), None);
    assert_eq!(reload.words.rows.len(), 2);
    assert_eq!(reload.words.rows.get(0).unwrap().request, None);
    assert_eq!(reload.words.rows.get(0).unwrap().state, Words::Sent { message: 2 });
    let author = party(7);
    assert_eq!(
        reload.lines().as_slice(),
        &[
            Line::Words {
                by: Some(&author),
                words: b"first",
                state: Words::Sent { message: 1 },
                at: Some(FIRST_WORDS_AT)
            },
            Line::Words {
                by: Some(&author),
                words: b"second",
                state: Words::Sent { message: 2 },
                at: Some(SECOND_WORDS_AT)
            }
        ]
    );
    assert!(reload.moved(3).is_some());
    let mut committed = ActivityLine {
        position: 3,
        at: Wall::EPOCH,
        event: Activity::Turn {
            name: TurnName { attempt: ATTEMPT, number: TURN },
            turn: TurnRead::Decoded(empty_turn()),
        },
    };
    if let Activity::Turn { turn: TurnRead::Decoded(turn), .. } = &mut committed.event {
        turn.read_through = Some(3);
    }
    let mut rows = List::with_capacity(1);
    rows.push(committed).unwrap();
    assert_eq!(reload.accept(ActivityPage { cut: false, rows, before: None, after: None }), None);
    assert_eq!(reload.words.rows.get(0).unwrap().state, Words::Read { message: 2 });
    assert_eq!(reload.words.rows.get(1).unwrap().state, Words::Read { message: 1 });
    assert_eq!(reload.words.rows.get(1).unwrap().words, text(b"first"));
}
fn words_page() -> ActivityPage {
    let mut rows = List::with_capacity(2);
    for (message, words, at) in [(2, b"second".as_slice(), SECOND_WORDS_AT), (1, b"first".as_slice(), FIRST_WORDS_AT)] {
        rows.push(ActivityLine {
            position: message,
            at,
            event: Activity::Words { by: party(7), message, words: text(words) },
        })
        .unwrap();
    }
    ActivityPage { cut: false, rows, before: None, after: None }
}

#[test]
fn owner_archive_reference_keeps_committed_read_and_local_author_separate_from_raw_sender() {
    use crate::contract::{OwnerActivity, OwnerActivityLine, OwnerMessageKind, OwnerRequester};
    use crate::thread::words::{KeptWords, Words};
    let mut kept = KeptWords::new(2);
    let request = ARCHIVE_WORDS;
    assert!(kept.begin(request, Some(party(7)), text(b"local")));
    assert!(kept.sent(request, 0));
    let line = |position, source| ActivityLine {
        position,
        at: Wall::EPOCH,
        event: Activity::Owner {
            source: Box::new(OwnerActivityLine { task: 8, position, at: Wall::EPOCH, line: source }),
            decoded: None,
        },
    };
    let mut rows = List::with_capacity(2);
    rows.push(line(2, OwnerActivity::Turn { reply: 9, attempt: 3, turn: 4, read: Some(0) })).unwrap();
    rows.push(line(
        1,
        OwnerActivity::Words {
            message: 0,
            by: OwnerRequester::Task(7),
            kind: OwnerMessageKind::Question,
            words: text(b"task words"),
        },
    ))
    .unwrap();
    let page = ActivityPage { rows, before: None, after: None, cut: false };
    kept.observe(&page);
    assert_eq!(kept.rows.len(), 1);
    assert_eq!(kept.rows.get(0).unwrap().state, Words::Read { message: 0 });
    assert_eq!(kept.rows.get(0).unwrap().by.as_ref().unwrap().name.bytes(), b"Pat");
    assert_eq!(page.rows.get(1).unwrap().event.word_message().unwrap().1.bytes(), b"task words");
    assert!(page.rows.get(0).unwrap().event.decoded_turn().is_none());
    assert_eq!(page.rows.get(0).unwrap().event.committed_turn(), Some(TurnName { attempt: 3, number: 4 }));
}

fn party(number: u64) -> Party {
    Party { number, kind: PartyKind::Person, name: text(b"Pat") }
}
fn empty_turn() -> Turn {
    Turn {
        messages: List::with_capacity(0),
        used: Usage { input: 0, output: 0, cached: 0, cache_written: 0 },
        spent: 0,
        read_through: None,
        cut: false,
    }
}

#[test]
fn a_thread_keeps_the_newest_text_within_its_bounds_and_names_its_loss() {
    let mut thread = Thread::new(fixtures::SUBJECT_TASK, &crate::thread::Limits { lines: 1, bytes: 2, text: 2 });
    thread.run(&run(), Some(&provisional(TURN, b"first")));
    thread.run(&run(), Some(&provisional(TURN + 1, b"second")));
    let lines = thread.lines();
    assert_eq!(
        lines.as_slice(),
        &[Line::Left { lines: 1 }, Line::Provisional { name: provisional(TURN + 1, b"").turn, text: b"nd", cut: true }]
    );
}
