use jig_client::{Ask, Asker, Handle, Key, Readable, Readout, Watchable, contract as c};
use jig_client_protocol_world::{client::Story, fixtures};
use skein_lib::Duration;
// These handles belong to this story's single read, activity page and task watch.
const TURN: Handle = Handle { asker: Asker::JIG, number: 31 };
const ACTIVITY: Handle = Handle { asker: Asker::JIG, number: 32 };
const TASK: Handle = Handle { asker: Asker::JIG, number: 33 };
const WORDS: Key = Key { asker: Asker::JIG, number: 91 };
// The fixture creates its first task at 1; 9 is absent.
const ABSENT: u64 = 9;
const PROJECT: u32 = 1;
// The worker's turn follows the run beginning and durable message at position 4.
const COMMITTED_TURN: u64 = 4;
const WAIT: Duration = Duration::from_secs(1);

#[test]
fn an_absent_turn_is_unknown_through_the_client() {
    let mut story = Story::live(fixtures::SEED);
    story.read(TURN, Readable::Jig(c::Read::Turn { task: ABSENT, attempt: 1, turn: 1 }));
    assert!(story.run_until(|s| !s.pages(TURN).is_empty(), WAIT));
    assert_eq!(story.pages(TURN), vec![Readout::Jig(c::ReadResult::None(c::ReadNone::Unknown))]);
}

fn committed_turn(mut story: Story) -> (Story, c::ArchivedTurn) {
    let task = story.running_task();
    let words = c::Text::new(Box::from(b"Please continue".as_slice()), 32).expect("bounded words");
    story.send(WORDS, Ask::Jig(Box::new(c::Ask::Message { project: PROJECT, task, words })));
    assert!(story.run_until(|s| !s.answered(WORDS).is_empty(), WAIT));
    story.watch(TASK, Watchable::Jig(c::Subject::Task { task }));
    assert!(story.run_until(|s| s.thread_activity(TASK, task) >= COMMITTED_TURN, WAIT));
    story.read(ACTIVITY, Readable::Jig(c::Read::Activity { task, before: None }));
    assert!(story.run_until(|s| !s.pages(ACTIVITY).is_empty(), WAIT));
    let pages = story.pages(ACTIVITY);
    let [Readout::Jig(c::ReadResult::Value(c::ReadValue::Activity(page)))] = pages.as_slice() else {
        panic!("activity page: {pages:?}")
    };
    let archive = page
        .rows
        .iter()
        .find_map(|line| {
            if let c::Activity::Owner { decoded: Some(archive), .. } = &line.event {
                Some(archive.clone())
            } else {
                None
            }
        })
        .expect("the committed turn's archive");
    (story, archive)
}

#[test]
fn a_committed_turn_is_read_whole_through_the_client() {
    let (mut story, archive) = committed_turn(Story::reading_chat(fixtures::SEED));
    let c::TurnRead::Decoded(turn) = &archive.turn else { panic!("decoded activity turn") };
    assert!(!turn.cut);
    story.read(
        TURN,
        Readable::Jig(c::Read::Turn { task: archive.task, attempt: archive.name.attempt, turn: archive.name.number }),
    );
    assert!(story.run_until(|s| !s.pages(TURN).is_empty(), WAIT));
    assert_eq!(story.pages(TURN), vec![Readout::Jig(c::ReadResult::Value(c::ReadValue::Turn(Box::new(archive))))]);
}

#[test]
fn an_activity_size_marker_keeps_its_bytes_through_the_client() {
    let body = fixtures::large_turn();
    let bytes = u64::try_from(body.len()).expect("source bytes");
    assert!(bytes > u64::from(fixtures::TURN_PAGE_BYTES));
    let story = Story::reading_chat_with(fixtures::SEED, body, fixtures::domain_limits(), fixtures::turn_page_limits());
    let (_, archive) = committed_turn(story);
    assert_eq!(archive.turn, c::TurnRead::Undecodable { why: c::TurnProblem::TooLarge { bytes } });
}
