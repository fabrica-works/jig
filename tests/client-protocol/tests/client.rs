use jig_client::{Asker, Handle, Latest, Link, Readable, Readout, State, Watchable, contract};
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
