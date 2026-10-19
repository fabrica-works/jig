use jig_client::{Ask, Asker, Handle, Key, Readable, Readout, Watchable, contract as c};
use jig_client_protocol_world::{client::Story, fixtures};
use skein_lib::Duration;
// These handles belong to this story's single read, activity page and task watch.
const TURN: Handle = Handle { asker: Asker::JIG, number: 31 };
const ACTIVITY: Handle = Handle { asker: Asker::JIG, number: 32 };
const TASK: Handle = Handle { asker: Asker::JIG, number: 33 };
// The cancellation is independent of the message request.
const END: Key = Key { asker: Asker::JIG, number: 92 };
const WORDS: Key = Key { asker: Asker::JIG, number: 91 };
// The fixture creates its first task at 1; 9 is absent.
const ABSENT: u64 = 9;
const PROJECT: u32 = 1;
// The worker's turn follows the run beginning and durable message at position 4.
const COMMITTED_TURN: u64 = 4;
const WAIT: Duration = Duration::from_secs(1);
// A disconnect shorter than the read deadline lets the same store reopen.
const OFFLINE: Duration = Duration::from_millis(100);

#[test]
fn an_absent_turn_is_unknown_through_the_client() {
    let mut story = Story::live(fixtures::SEED);
    assert_eq!(
        read_once(&mut story, c::Read::Turn { task: ABSENT, attempt: 1, turn: 1 }),
        Readout::Jig(c::ReadResult::None(c::ReadNone::Unknown))
    );
}

fn committed_story(mut story: Story) -> (Story, u64) {
    let task = story.running_task();
    let words = c::Text::new(Box::from(b"Please continue".as_slice()), 32).expect("bounded words");
    story.send(WORDS, Ask::Jig(Box::new(c::Ask::Message { project: PROJECT, task, words })));
    assert!(story.run_until(|s| !s.answered(WORDS).is_empty(), WAIT));
    story.watch(TASK, Watchable::Jig(c::Subject::Task { task }));
    assert!(story.run_until(|s| s.thread_activity(TASK, task) >= COMMITTED_TURN, WAIT));
    (story, task)
}

fn committed_turn(story: Story) -> (Story, c::ArchivedTurn) {
    let (mut story, task) = committed_story(story);
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
    assert_eq!(read_archive(&mut story, &archive), archive);
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

// Wait past the terminal for another exchange, and retain every Page for this handle.
fn read_once(story: &mut Story, read: c::Read) -> Readout {
    story.read(TURN, Readable::Jig(read));
    assert!(story.run_until(|s| !s.pages(TURN).is_empty(), WAIT));
    let exchanges = story.exchanges();
    assert!(!story.run_until(|_| false, WAIT));
    assert_eq!(story.exchanges(), exchanges);
    let pages = story.pages(TURN);
    let [page] = pages.as_slice() else { panic!("one terminal: {pages:?}") };
    page.clone()
}

fn archive_read(archive: &c::ArchivedTurn) -> c::Read {
    c::Read::Turn { task: archive.task, attempt: archive.name.attempt, turn: archive.name.number }
}

fn read_archive(story: &mut Story, archive: &c::ArchivedTurn) -> c::ArchivedTurn {
    let page = read_once(story, archive_read(archive));
    let Readout::Jig(c::ReadResult::Value(c::ReadValue::Turn(turn))) = page else { panic!("whole turn: {page:?}") };
    *turn
}

#[test]
fn a_page_sized_turn_is_read_whole_once() {
    let body = fixtures::callable_turn();
    let bytes = u64::try_from(body.len()).expect("source size");
    let story = Story::reading_chat_with(
        fixtures::SEED,
        body.clone(),
        fixtures::domain_limits(),
        fixtures::turn_overlap_limits(),
    );
    let (mut story, mut archive) = committed_turn(story);
    let (single, activity) = story.turn_row_costs(&archive, body);
    assert!(single <= fixtures::TURN_OVERLAP_BYTES);
    assert!(fixtures::TURN_OVERLAP_BYTES < activity);
    assert_eq!(archive.turn, c::TurnRead::Undecodable { why: c::TurnProblem::TooLarge { bytes } });
    let single_turn = read_archive(&mut story, &archive);
    let c::TurnRead::Decoded(turn) = &single_turn.turn else { panic!("decoded turn") };
    assert!(!turn.cut);
    let [message] = turn.messages.as_slice() else { panic!("source message") };
    let [c::Block::Text(text), c::Block::Call(call)] = message.blocks.as_slice() else { panic!("source blocks") };
    assert_eq!(text.bytes(), fixtures::TURN_TEXT);
    assert_eq!(call.tool.bytes(), fixtures::TURN_TOOL);
    assert_eq!(call.input.bytes(), fixtures::TURN_INPUT);
    assert_eq!(call.number, fixtures::TURN_CALL_NUMBER);
    archive.turn = single_turn.turn.clone();
    assert_eq!(single_turn, archive);
}

fn restart_story(story: &mut Story) {
    story.restart_engine(OFFLINE);
    assert!(story.run_until(|s| s.client.link().state == jig_client::Link::Live, WAIT));
}

#[test]
fn a_turn_read_after_restart_keeps_its_archive() {
    let (mut story, archive) = committed_turn(Story::reading_chat(fixtures::SEED));
    restart_story(&mut story);
    assert_eq!(read_archive(&mut story, &archive), archive);
}

fn oversized_turn(client: jig_client_domain::Limits, core: &jig_test_domain::Limits) {
    let body = fixtures::large_turn();
    assert_eq!(u64::try_from(body.len()).expect("source size"), fixtures::LARGE_TURN_BYTES);
    let story = Story::reading_chat_with(fixtures::SEED, body, client, *core);
    let (mut story, task) = committed_story(story);
    assert_eq!(
        read_once(&mut story, c::Read::Turn { task, attempt: 1, turn: 1 }),
        Readout::Jig(c::ReadResult::None(c::ReadNone::TooLarge { bytes: fixtures::LARGE_TURN_BYTES }))
    );
}

#[test]
fn a_single_turn_past_the_engine_bound_keeps_its_size() {
    oversized_turn(fixtures::domain_limits(), &fixtures::turn_page_limits());
}

#[test]
fn a_single_turn_past_the_client_bound_keeps_its_size() {
    let mut client = fixtures::domain_limits();
    // The body fits the engine's usual read bound, but not this client document.
    client.document_bytes = u64::from(fixtures::TURN_PAGE_BYTES);
    oversized_turn(client, &fixtures::owner_limits());
}

#[test]
fn an_invisible_turn_is_unknown_after_its_task_ends() {
    let (mut story, archive) = committed_turn(Story::reading_chat(fixtures::SEED));
    story.send(
        END,
        Ask::Jig(Box::new(c::Ask::Cancel {
            project: PROJECT,
            task: archive.task,
            reason: c::Text::new(b"done".as_slice().into(), 32).expect("bounded reason"),
            grace: None,
        })),
    );
    assert!(story.run_until(|s| s.phase(archive.task) == Some(c::Phase::Cancelled), WAIT));
    story.hide_turn_task();
    restart_story(&mut story);
    assert_eq!(read_once(&mut story, archive_read(&archive)), Readout::Jig(c::ReadResult::None(c::ReadNone::Unknown)));
}
