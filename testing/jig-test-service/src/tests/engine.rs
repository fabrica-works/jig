use super::fixtures;
use crate::{End, Stop, StoreAnswer, StoreRequest};
use jig_domain_core as core;

#[test]
fn the_restart_asks_one_load_at_a_time() {
    let mut service = fixtures::started(&fixtures::engine_document());
    fixtures::turn(&mut service);
    fixtures::turn(&mut service);
    let engine = fixtures::engine(&mut service);
    assert_eq!(engine.store_requests().len(), 1);
    let Some(StoreRequest::Restart(step)) = engine.store_requests().pop() else {
        panic!("restart load requested");
    };
    assert_eq!(step, core::RestartStep::LoadCore);
    engine.store_answers().push(StoreAnswer::Restored { step: core::RestartStep::LoadCore, rows: Box::new([]) });
    fixtures::turn(&mut service);
    let engine = fixtures::engine(&mut service);
    let Some(StoreRequest::Commit { .. }) = engine.store_requests().pop() else {
        panic!("restart writes commit first");
    };
    assert_eq!(engine.store_requests().len(), 0);
}

fn period(driven: &mut fixtures::Driven, number: u64) {
    driven.party(core::Event::Period { project: 1, period: number, budget: 100, next: None });
}

#[test]
fn a_store_answer_is_taken_with_the_journal_full() {
    let mut driven = fixtures::Driven::started(&fixtures::engine_document());
    for number in 2..=4 {
        period(&mut driven, number);
    }
    let first = driven.commit_asked();
    driven.commit(first);
    assert_eq!(fixtures::engine(&mut driven.service).store_answers().len(), 0);
    period(&mut driven, 5);
    assert_eq!(driven.commits.len(), 3);
}

#[test]
fn a_party_request_is_returned_busy_with_the_journal_full() {
    let mut driven = fixtures::Driven::started(&fixtures::engine_document());
    for number in 2..=4 {
        period(&mut driven, number);
    }
    driven.chat(101, b"retained");
    let event = driven.busy.pop().expect("unadmitted party input");
    let jig_test_domain::Event::Core(core::Event::People(jig_domain_people::Event::Ask { ask, .. })) = *event else {
        panic!("busy returns the same party request");
    };
    let jig_domain_people::Ask::StartChat { words, .. } = ask else {
        panic!("same chat request");
    };
    assert_eq!(words.as_ref(), b"retained");
}

#[test]
fn a_commit_leaves_in_the_iteration_that_made_it() {
    let mut driven = fixtures::Driven::started(&fixtures::engine_document());
    period(&mut driven, 2);
    assert_eq!(driven.commits.len(), 1);
}

#[test]
fn a_failed_commit_releases_nothing_after_it() {
    let mut driven = fixtures::Driven::started(&fixtures::engine_document());
    driven.chat(101, b"one");
    let first = driven.commit_asked();
    driven.chat(102, b"two");
    let second = driven.commit_asked();
    driven.fail(first);
    driven.commit(second);
    assert_eq!(driven.released_to_parties(), 0);
    assert_eq!(fixtures::engine(&mut driven.service).store_answers().len(), 0);
    assert_eq!(driven.end(), Some(&End::Stopped(Stop::CommitFailed { number: first })));
}

#[test]
fn the_service_waits_while_its_load_is_outstanding() {
    let mut service = fixtures::started(&fixtures::engine_document());
    fixtures::turn(&mut service);
    assert!(!service.work_pending(skein_lib::Time::from_nanos(1)), "waiting for the store is not work");
}

#[test]
fn a_store_answer_is_work_when_party_outputs_are_full() {
    let mut driven = fixtures::Driven::started(&fixtures::engine_document());
    period(&mut driven, 2);
    let number = driven.commit_asked();
    let engine = fixtures::engine(&mut driven.service);
    for _ in 0..engine.parties_out().room() {
        engine.parties_out().push(crate::PartyOutput::Busy(Box::new(jig_test_domain::Event::Core(
            core::Event::Period { project: 1, period: 3, budget: 100, next: None },
        ))));
    }
    engine.store_answers().push(StoreAnswer::Committed { number, fill: 0 });
    assert!(driven.service.work_pending(skein_lib::Time::from_nanos(1)));
    fixtures::turn(&mut driven.service);
    assert_eq!(fixtures::engine(&mut driven.service).store_answers().len(), 0);
}

fn path() -> jig_test_connector::Path {
    let mut segments = skein_lib::List::with_capacity(1);
    segments.push(b"one".as_slice().into()).expect("one segment");
    jig_test_connector::Path::new(segments, 8).expect("fixture path")
}
fn effect(target: u64) -> jig_test_connector::Effect {
    jig_test_connector::Effect {
        kind: 1,
        resources: Box::new([path()]),
        purpose: 1,
        condition: None,
        target,
        state: target,
    }
}
fn key() -> jig_test_connector::Key {
    jig_test_connector::Key { deployment: [1; 16], task: 1, purpose: 1, attempt: 1, completion: 1, position: 1 }
}
#[test]
fn the_system_applies_an_effect_and_reads_it_back() {
    use jig_test_connector::{ApplyResult, Fact, Form, Origin, Recovery, SystemEvent, SystemRequest};
    let mut system = crate::system::System::new(2, 2);
    assert_eq!(
        system.answer(SystemRequest::Apply {
            entry: 1,
            attempt: 1,
            key: key(),
            effect: effect(7),
            form: Form::Set,
            recovery: Recovery::Keyed
        }),
        SystemEvent::Applied { entry: 1, attempt: 1, result: ApplyResult::Made { state: 7 } }
    );
    assert_eq!(
        system.answer(SystemRequest::ReadFact { resource: path(), observed: skein_lib::Wall::EPOCH }),
        SystemEvent::Fact {
            resource: path(),
            fact: Fact { state: Some(7), observed: skein_lib::Wall::EPOCH, pending: false },
            origin: Origin::Own
        }
    );
}

#[test]
fn a_keyed_retry_keeps_the_first_result() {
    use jig_test_connector::{ApplyResult, Form, Recovery, SystemEvent, SystemRequest};
    let mut system = crate::system::System::new(2, 2);
    let _first = system.answer(SystemRequest::Apply {
        entry: 1,
        attempt: 1,
        key: key(),
        effect: effect(7),
        form: Form::Set,
        recovery: Recovery::Keyed,
    });
    assert_eq!(
        system.answer(SystemRequest::Apply {
            entry: 1,
            attempt: 2,
            key: key(),
            effect: effect(9),
            form: Form::Set,
            recovery: Recovery::Keyed
        }),
        SystemEvent::Applied { entry: 1, attempt: 2, result: ApplyResult::Made { state: 7 } }
    );
}
