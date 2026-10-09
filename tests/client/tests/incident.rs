//! The two independent incident journeys of pages.md, section 10.
use jig_client::{
    address::{Address, TaskSection},
    contract::*,
};
use jig_client_view::Role;
use jig_client_world::{Scripted, Test, Until, World, stories};
use jig_fake_person::{Find, Step};
use jig_scripted_engine::{Change, Rows, Source};
use jig_test_client_domain::{Address as RootAddress, family};
use skein_lib::{Duration, Wall};
const WITHIN: Duration = Duration::from_secs(1);
fn see(words: &[u8]) -> Step<RootAddress> {
    Step::See { find: Find::text(words), within: WITHIN }
}
fn press(name: &[u8]) -> Step<RootAddress> {
    Step::Press { find: Find::named(Role::Button, name) }
}
fn finish(w: &mut World<Test, Scripted>, withdrawn: bool) {
    let now = w.now();
    let mut state = w.front().engine.application().clone();
    let subject = family::Subject { scope: family::Scope::Project(1), kind: family::Kind::Item(1) };
    let family::Document::Item(item) = state.documents.get_mut(&subject).unwrap() else { unreachable!() };
    item.state = family::ItemState::Ended;
    item.revision.value += 1;
    let mut delegate = stories::task();
    delegate.chip.task = 10;
    delegate.chip.title = stories::text(b"Delegate ten");
    let lead = stories::reported(stories::task(), b"Incident resolved");
    let delegate = stories::reported(delegate, if withdrawn { b"Already healthy by hand" } else { b"Service healthy" });
    let mut proposal = stories::proposal();
    proposal.ending = Some(if withdrawn {
        Ending::Withdrawn { reason: stories::text(b"Already healthy by hand") }
    } else {
        Ending::Decided { by: stories::frame(7).party, how: Decision::Accepted, at: Wall::EPOCH }
    });
    w.front()
        .engine
        .act_application(
            now,
            Wall::EPOCH,
            jig_test_client_domain::FIRST,
            1,
            state,
            vec![
                Change::Object { project: 1, key: ObjectKey::Task(9), value: ReadValue::Task(Box::new(lead)) },
                Change::Object { project: 1, key: ObjectKey::Task(10), value: ReadValue::Task(Box::new(delegate)) },
                Change::Object {
                    project: 1,
                    key: ObjectKey::Proposal(proposal.key),
                    value: ReadValue::Proposal(Box::new(proposal)),
                },
                Change::Source(Source {
                    party: 7,
                    read: Read::Inbox { project: None, kind: None, goal: None, after: None },
                    rows: Rows::Inbox(vec![]),
                }),
            ],
        )
        .expect("resolved incident is one durable application and jig commit");
}
#[test]
fn an_incident_delegates_accepts_on_the_leads_plan_and_resolves_with_its_report() {
    for seed in [51, 52] {
        let mut w = stories::incident_world(seed);
        let c = w.client(7, stories::incident());
        let inbox = w.client(7, stories::inbox());
        w.person(
            c,
            vec![
                see(b"Item 1"),
                see(b"Chat nine"),
                Step::Go { address: stories::chat() },
                see(b"delegate"),
                Step::Go { address: RootAddress::Jig(Address::Task { task: 9, section: TaskSection::Plan }) },
                see(b"Delegate ten"),
                see(b"Resolve the incident"),
                press(b"Accept"),
                press(b"Confirm"),
                see(b"Accepted"),
                Step::Go { address: RootAddress::Jig(Address::Task { task: 10, section: TaskSection::Panels }) },
                see(b"Value: 31"),
            ],
        );
        w.person(inbox, vec![see(b"Resolve the incident"), see(b"Accepted")]);
        w.run(Until::Done);
        let now = w.now();
        let mut state = w.front().engine.application().clone();
        let panel = family::Subject { scope: family::Scope::Project(1), kind: family::Kind::Panel(10) };
        let family::Document::Part(part) = state.documents.get_mut(&panel).unwrap() else { unreachable!() };
        part.revision.value += 1;
        part.value = 32;
        w.front().engine.act_application(now, Wall::EPOCH, jig_test_client_domain::FIRST, 1, state, vec![]).unwrap();
        w.person(c, vec![see(b"Value: 32")]);
        w.run(Until::Done);
        finish(&mut w, false);
        w.person(c, vec![Step::Go { address: stories::incident() }, see(b"Ended"), see(b"Incident resolved")]);
        w.run(Until::Done);
        assert_eq!(w.front().engine.commits().iter().filter(|c| c.key.is_some()).count(), 1);
    }
}
#[test]
fn an_incident_proposal_withdrawn_while_waiting_leaves_with_its_delegates_reason() {
    for seed in [53, 54] {
        let mut w = stories::incident_world(seed);
        let c = w.client(7, stories::inbox());
        w.person(c, vec![see(b"Resolve the incident")]);
        w.run(Until::Done);
        finish(&mut w, true);
        w.person(
            c,
            vec![
                see(b"Withdrawn"),
                see(b"Already healthy by hand"),
                Step::Go { address: RootAddress::Jig(Address::Task { task: 10, section: TaskSection::Overview }) },
                see(b"Already healthy by hand"),
            ],
        );
        w.run(Until::Done);
        assert!(w.front().engine.commits().iter().all(|c| c.key.is_none()));
    }
}
