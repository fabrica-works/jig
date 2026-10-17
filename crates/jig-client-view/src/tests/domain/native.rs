use super::spine::{LIMITS, delivered, drive, frame, open_token, start};
use crate::{
    Event, Request,
    action::Act,
    address::{Address, Place, ProjectSection, TaskSection},
    page::{Page, Section},
    tests::{
        cards::{contains, tree},
        fixtures::{Fixture, ReadPage},
    },
};
use jig_client::{
    Handle, Up,
    contract::{Delivery, Limit, Read, ReadResult, ReadValue, WatchEvent},
};
use skein_lib::{List, Queue, Time, Wall};
const ME: u64 = 7;
const TASK: u64 = 8;
const PROJECT: u32 = 1;
pub(super) fn ready(limits: &crate::Limits) -> (Fixture, Handle) {
    let mut fixture = Fixture::new(limits);
    let handle = start(&mut fixture, List::with_capacity(0));
    delivered(&mut fixture, Time::ZERO, handle, frame(ME));
    (fixture, handle)
}
#[test]
fn a_page_reads_its_documents_from_the_client_by_handle() {
    let (mut fixture, handle) = ready(&LIMITS);
    let mut changed = frame(ME);
    changed.party.name = jig_client::contract::DocumentText::complete(super::objects::text(b"fresh client name"));
    let mut out = Queue::with_capacity(512);
    fixture.client.step(
        Time::ZERO,
        Wall::EPOCH,
        jig_client::Event::Up {
            up: Up::Watched {
                watch: fixture.watch_token(handle).unwrap(),
                event: WatchEvent::Delivered(Delivery::Frame(Box::new(changed.clone()))),
            },
        },
        &mut out,
    );
    assert_eq!(fixture.frame().document, Some(&changed));
}
#[test]
fn a_handle_the_page_no_longer_holds_is_dropped() {
    let (mut fixture, _) = ready(&LIMITS);
    let opened = drive(&mut fixture, Time::ZERO, Event::Went { place: Place::Jig(Address::task(TASK)) });
    let handle = open_token(&opened);
    drive(&mut fixture, Time::ZERO, Event::Left);
    assert!(!fixture.handles.watches.contains_key(&handle));
    let shown = fixture.shown();
    drive(&mut fixture, Time::ZERO, Event::Client { told: jig_client::Told::Changed { handle } });
    assert_eq!(fixture.shown(), shown);
}
#[test]
fn signed_out_shows_the_sign_in_page_and_keeps_the_place() {
    let (mut fixture, _) = ready(&LIMITS);
    let place = Address::task(TASK);
    drive(&mut fixture, Time::ZERO, Event::Went { place: Place::Jig(place) });
    drive(&mut fixture, Time::ZERO, Up::SignedOut);
    assert_eq!(fixture.page(), &Page::SigningIn);
    assert_eq!(fixture.address, place);
    assert!(contains(&tree(&fixture, None, None), b"Sign in"));
}
#[test]
fn out_of_date_shows_its_page_and_sends_nothing() {
    let (mut fixture, _) = ready(&LIMITS);
    drive(&mut fixture, Time::ZERO, Up::Served { served: false, versions: None });
    assert_eq!(fixture.page(), &Page::OutOfDate);
    assert!(
        drive(&mut fixture, Time::ZERO, Event::Act { action: Act::Go { address: Address::task(TASK) } }).is_empty()
    );
    assert!(contains(&tree(&fixture, None, None), b"Client out of date"));
}
#[test]
fn a_sign_out_that_did_not_reach_the_engine_is_offered_again() {
    let (mut fixture, _) = ready(&LIMITS);
    drive(&mut fixture, Time::ZERO, Event::Act { action: Act::SignOut });
    drive(&mut fixture, Time::ZERO, jig_client::Event::SignOutLost);
    assert_eq!(fixture.frame().party(), Some(ME));
    assert!(contains(&tree(&fixture, None, None), b"Not signed out: the engine was not reached"));
    assert_eq!(
        drive(&mut fixture, Time::ZERO, Event::Act { action: Act::SignOut }).as_ref(),
        &[Request::Client { asked: jig_client::Event::SignOut }]
    );
}
#[test]
fn a_page_past_the_ui_s_handles_opens_what_fits_and_says_what_it_left_out() {
    let (fixture, _) = ready(&crate::Limits { watches: 1, ..LIMITS });
    assert_eq!(fixture.handles.watches.len(), 1);
    let Page::Work { refused, .. } = fixture.page() else { panic!("bounded work page") };
    assert_eq!(*refused, Some(Limit::Watches));
    assert!(contains(&tree(&fixture, None, None), b"Watch limit reached"));
}
fn named() -> (Fixture, Section, Section, Handle) {
    let (mut fixture, _) = ready(&LIMITS);
    drive(&mut fixture, Time::ZERO, Event::Left);
    let task = Section::Task { task: TASK, section: TaskSection::Plan };
    let deployment = Section::Deployment;
    let mut sections = List::with_capacity(2);
    sections.push(task).unwrap();
    sections.push(deployment).unwrap();
    let opened = drive(&mut fixture, Time::ZERO, Event::Named { objects: List::with_capacity(0), sections });
    let handle = open_token(&opened);
    (fixture, task, deployment, handle)
}
#[test]
fn naming_the_same_sections_again_keeps_their_watches_open() {
    let (mut fixture, task, deployment, handle) = named();
    let mut sections = List::with_capacity(2);
    sections.push(task).unwrap();
    sections.push(deployment).unwrap();
    assert_eq!(
        drive(&mut fixture, Time::ZERO, Event::Named { objects: List::with_capacity(0), sections }).as_ref(),
        &[]
    );
    assert!(fixture.handles.watches.contains_key(&handle));
}
#[test]
fn each_named_section_says_behind_in_its_own_region() {
    let (mut fixture, task, deployment, handle) = named();
    drive(
        &mut fixture,
        Time::ZERO,
        crate::tests::fixtures::Ended { watch: handle, end: jig_client::contract::WatchEnd::Dropped },
    );
    assert!(fixture.section_status(task).unwrap().behind);
    assert!(!fixture.section_status(deployment).unwrap().behind);
    assert!(contains(&tree(&fixture, Some(task), None), b"Behind: waiting for current facts"));
    assert!(!contains(&tree(&fixture, Some(deployment), None), b"Behind: waiting for current facts"));
}
#[test]
fn the_sign_in_page_leaves_a_slot_for_the_application_s_ways() {
    let mut fixture = Fixture::new(&LIMITS);
    drive(&mut fixture, Time::ZERO, Up::SignedOut);
    start(&mut fixture, List::with_capacity(0));
    assert_eq!(fixture.page(), &Page::SigningIn);
    let built = tree(&fixture, None, None);
    let mut slots = List::with_capacity(16);
    for node in built.nodes() {
        if let Some(slot) = node.slot {
            slots.push(slot).unwrap();
        }
    }
    assert_eq!(slots.as_slice(), &[crate::signin::SLOT]);
}
#[test]
fn a_handle_the_client_s_limit_ends_is_one_the_page_left_out() {
    let mut fixture = Fixture::with_client_handles(&LIMITS, 1);
    let handle = start(&mut fixture, List::with_capacity(0));
    delivered(&mut fixture, Time::ZERO, handle, frame(ME));
    let Page::Work { refused, .. } = fixture.page() else { panic!("limited page") };
    assert_eq!(*refused, Some(Limit::Watches));
    assert!(contains(&tree(&fixture, None, None), b"Watch limit reached"));
}
#[test]
fn a_read_s_page_stays_until_the_page_is_left_or_its_next_page_replaces_it() {
    let (mut fixture, frame_handle) = ready(&LIMITS);
    let project = Address::Project { project: PROJECT, section: ProjectSection::Settings };
    let opened = drive(&mut fixture, Time::ZERO, Event::Went { place: Place::Jig(project) });
    let read = super::pages::read_named(&opened, &Read::Project { project: PROJECT });
    let value = super::owner_project::source(PROJECT);
    drive(
        &mut fixture,
        Time::ZERO,
        ReadPage { read, result: ReadResult::Value(ReadValue::OwnerProject(Box::new(value.clone()))) },
    );
    delivered(&mut fixture, Time::ZERO, frame_handle, frame(ME));
    assert_eq!(fixture.project_page().owner.as_deref(), Some(&value));
    assert!(contains(&tree(&fixture, None, None), b"18446744073709551615"));
    drive(&mut fixture, Time::ZERO, Event::Left);
    assert_eq!(fixture.project_page().owner, None);
}
