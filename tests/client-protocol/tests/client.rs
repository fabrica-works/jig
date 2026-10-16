use jig_client::{
    Answered, Ask, Asker, Handle, Key, Latest, Link, Outcome, Readable, Readout, State, Watchable, contract,
};
use jig_client_protocol_world::{
    client::{Referee, Story},
    fixtures,
};
use skein_lib::Duration;
const ME: Handle = Handle { asker: Asker::JIG, number: 1 };
const PROJECT: Handle = Handle { asker: Asker::JIG, number: 2 };
const INBOX: Handle = Handle { asker: Asker::JIG, number: 3 };
const TASK: u64 = 9;
#[test]
fn a_client_signed_in_by_the_printed_link_sees_me_live() {
    let mut story = Story::new(fixtures::SEED);
    story.sign_in();
    assert!(story.run_until(|s| s.client.link().state == Link::Live, Duration::from_secs(1)));
    assert_eq!(story.told_links(), vec![Link::Starting, Link::Live]);
    assert_eq!(story.client.link().party, Some(fixtures::FIRST_OWNER));
}
#[test]
fn a_client_reads_the_project_me_names_by_its_handle() {
    let mut story = Story::live(fixtures::SEED);
    story.watch(ME, Watchable::Jig(contract::Subject::Frame));
    let Some(Latest::Jig(contract::Delivery::Frame(me))) = story.client.document(ME).expect("watched handle").latest
    else {
        panic!("me");
    };
    let project = me.projects.as_slice().first().expect("owner's project").project;
    story.read(PROJECT, Readable::Jig(contract::Read::Project { project }));
    assert!(story.run_until(|s| !s.pages(PROJECT).is_empty(), Duration::from_secs(1)));
    let pages = story.pages(PROJECT);
    let [Readout::Jig(contract::ReadResult::Value(contract::ReadValue::OwnerProject(page)))] = pages.as_slice() else {
        panic!("one project page");
    };
    assert_eq!(page.number, project);
}
#[test]
fn a_dropped_stream_makes_each_handle_behind_then_live_with_me() {
    let mut story = Story::live(fixtures::SEED);
    story.watch(ME, Watchable::Jig(contract::Subject::Frame));
    story.watch(INBOX, Watchable::Jig(contract::Subject::Inbox { window: 2 }));
    assert!(
        story.run_until(
            |s| s.client.document(INBOX).expect("watched handle").state == State::Live,
            Duration::from_secs(1)
        )
    );
    story.cut_stream();
    assert!(story.run_until(|s| s.client.link().state == Link::Offline, Duration::from_secs(1)));
    assert_eq!(story.client.document(ME).expect("watched handle").state, State::Behind);
    assert_eq!(story.client.document(INBOX).expect("watched handle").state, State::Behind);
    assert!(story.run_until(
        |s| s.client.link().state == Link::Live
            && s.client.document(INBOX).expect("watched handle").state == State::Live,
        Duration::from_secs(1)
    ));
    assert_eq!(story.client.document(ME).expect("watched handle").state, State::Live);
}
#[test]
fn each_delivery_the_client_took_reaches_the_core_as_taken() {
    let mut story = Story::live(fixtures::SEED);
    story.watch(INBOX, Watchable::Jig(contract::Subject::Inbox { window: 2 }));
    assert!(
        story.run_until(
            |s| s.client.document(INBOX).expect("watched handle").state == State::Live,
            Duration::from_secs(1)
        )
    );
    story.event(jig_client::Event::Unwatch { handle: INBOX });
    assert!(story.run_until(Story::deliveries_taken, Duration::from_secs(1)));
    assert_eq!(story.client.document(INBOX), None);
}
#[test]
fn a_document_older_than_one_told_is_caught() {
    let mut referee = Referee::default();
    let subject = Watchable::Jig(contract::Subject::Task { task: TASK });
    assert_eq!(referee.document(subject.clone(), 2), Ok(()));
    assert_eq!(referee.document(subject, 1), Err("document revision moved backwards"));
}

const STOP: Key = Key { asker: Asker(1), number: 1 };
const SECOND_STOP: Key = Key { asker: Asker(1), number: 2 };
const PROJECT_NUMBER: u32 = 1;
const WIRE_KEY: [u8; 16] = [19; 16];
fn stop(task: u64) -> Ask {
    Ask::Jig(Box::new(contract::Ask::Stop { project: PROJECT_NUMBER, task }))
}
fn assert_one_refusal(story: &Story) {
    let answers = story.answered(STOP);
    let [Outcome::Answered(Answered::Jig(contract::Answer::Refused(_)))] = answers.as_slice() else {
        panic!("one refusal: {answers:?}");
    };
    assert_eq!(story.made(fixtures::FIRST_OWNER), 1);
}
#[test]
fn a_stop_is_answered_once_after_its_commit_is_durable() {
    let mut story = Story::live(fixtures::SEED);
    story.send(STOP, stop(TASK));
    assert!(story.run_until(|s| !s.answered(STOP).is_empty(), Duration::from_secs(1)));
    assert_one_refusal(&story);
    assert!(story.told_at(STOP) >= story.durable_at(fixtures::FIRST_OWNER));
}
#[test]
fn a_stop_kept_across_a_reload_is_answered_once() {
    let mut story = Story::live(fixtures::SEED);
    story.cut_answer();
    story.send(STOP, stop(TASK));
    assert!(story.run_until(|s| s.made(fixtures::FIRST_OWNER) == 1, Duration::from_secs(1)));
    assert_eq!(story.answered(STOP), vec![]);
    story.reload();
    assert!(story.run_until(|s| !s.answered(STOP).is_empty(), Duration::from_secs(1)));
    assert!(story.kept(STOP));
    assert_one_refusal(&story);
}
#[test]
fn a_stop_whose_record_is_refused_is_answered_not_kept_and_never_made() {
    let mut story = Story::live(fixtures::SEED);
    story.refuse_next_save();
    story.send(STOP, stop(TASK));
    assert!(story.run_until(|s| !s.answered(STOP).is_empty(), Duration::from_secs(1)));
    assert_eq!(story.answered(STOP), vec![Outcome::NotKept]);
    assert_eq!(story.made(fixtures::FIRST_OWNER), 0);
}
#[test]
fn the_engine_restarting_under_a_pending_stop_answers_it_once() {
    let mut story = Story::live(fixtures::SEED);
    story.cut_answer();
    story.send(STOP, stop(TASK));
    assert!(story.run_until(|s| s.made(fixtures::FIRST_OWNER) == 1, Duration::from_secs(1)));
    assert_eq!(story.answered(STOP), vec![]);
    story.restart_engine(Duration::from_millis(10));
    assert!(story.run_until(|s| !s.answered(STOP).is_empty(), Duration::from_secs(1)));
    assert_one_refusal(&story);
}
#[test]
fn two_clients_stopping_one_task_are_answered_once_each() {
    let mut story = Story::live(fixtures::SEED);
    let task = story.start_task();
    let made = story.made(fixtures::FIRST_OWNER);
    story.add_second_client();
    story.send(STOP, stop(task));
    story.send_second(SECOND_STOP, stop(task));
    assert!(story.run_until(
        |s| !s.answered(STOP).is_empty() && !s.second().answered(SECOND_STOP).is_empty(),
        Duration::from_secs(1)
    ));
    let answers = story.answered(STOP);
    assert_eq!(answers.len(), 1);
    assert_eq!(story.second().answered(SECOND_STOP), answers);
    assert_eq!(story.made(fixtures::FIRST_OWNER), made + 2);
}
#[test]
fn a_keyed_exchange_before_its_store_is_caught() {
    let referee = Referee::default();
    assert_eq!(
        referee.keyed(WIRE_KEY, Some(fixtures::FIRST_OWNER)),
        Err("keyed exchange left before its record was stored")
    );
}
