use super::super::{
    objects::{chip, live, person},
    spine::{LIMITS, drive},
};
use super::{answered, install, read_token};
use crate::{
    Event, Request,
    intents::send_card,
    objects::{Body, Card, Source},
};
use jig_client::contract::*;
use skein_lib::{List, Queue, Time, Token};

fn document(taken_by: Option<u64>, offered: &[ActKind]) -> OwnerPersonTask {
    let mut task = chip(8);
    task.details = Box::new(ChipPhase::Owner {
        phase: OwnerChipPhase::Person { address: PersonAddress::Role(u32::MAX), taken_by, since: None },
        details: Box::new(super::super::chip_facts::details()),
    });
    let mut acts = List::with_capacity(3);
    for act in offered {
        acts.push(Offered { act: *act, admission: Admission::Allowed }).unwrap();
    }
    OwnerPersonTask {
        task,
        revision: Revision { value: 91 },
        requester: OwnerRequester::Task(u64::MAX),
        spec: person().spec,
        address: PersonAddress::Role(u32::MAX),
        taken_by,
        since: None,
        stall: None,
        deadline: None,
        contract: TaskContractDocument::Report { words: 3 },
        results: List::with_capacity(3),
        cut: false,
        acts,
    }
}

fn send(domain: &mut crate::Domain, act: ActKind) -> Token {
    let ask = if act == ActKind::Take {
        Ask::Take { project: u32::MAX, task: 8 }
    } else {
        Ask::HandBack { project: u32::MAX, task: 8 }
    };
    let mut out = Queue::with_capacity(crate::max_out(&LIMITS));
    let token = send_card(domain, ObjectKey::PersonTask(8), act, Box::new(ask), &mut out).unwrap();
    assert_eq!(out.len(), 1);
    token
}

#[test]
fn person_take_and_hand_back_read_whole_current_facts_and_offers_after_the_durable_answer() {
    for act in [ActKind::Take, ActKind::HandBack] {
        let (mut domain, _) = live();
        let original = document(if act == ActKind::Take { None } else { Some(u64::MAX) }, &[act]);
        let id = install(&mut domain, Body::OwnerPersonTask(Box::new(original.clone())));
        let request = send(&mut domain, act);
        assert_eq!(domain.objects().get(id).unwrap().body, Some(Body::OwnerPersonTask(Box::new(original))));
        let outcome = if act == ActKind::Take {
            OwnerOutcome::PersonTaken { task: 8 }
        } else {
            OwnerOutcome::PersonHandedBack { task: 8 }
        };
        let answer = Answer::Done(Outcome::Owner(outcome));
        let rows = answered(&mut domain, request, answer.clone());
        assert_eq!(rows.len(), 1);
        let Request::Read { what, .. } = &rows[0] else { panic!("one fresh object read") };
        assert_eq!(*what, Read::PersonTask { task: 8 });
        assert_eq!(domain.objects().get(id).unwrap().card, Card::Open);
        assert_eq!(domain.objects().get(id).unwrap().answer, Some(answer.clone()));
        assert!(answered(&mut domain, request, answer.clone()).is_empty());
        let current = if act == ActKind::Take {
            document(Some(u64::MAX), &[ActKind::Answer, ActKind::HandBack])
        } else {
            document(None, &[ActKind::Take])
        };
        drive(
            &mut domain,
            Time::ZERO,
            Event::Read {
                read: read_token(&rows),
                result: ReadResult::Value(ReadValue::OwnerPersonTask(Box::new(current.clone()))),
            },
        );
        let object = domain.objects().get(id).unwrap();
        assert_eq!(object.body, Some(Body::OwnerPersonTask(Box::new(current.clone()))));
        assert_eq!(object.answer, Some(answer));
        assert_eq!(object.card, Card::Open);
        for (kind, offered) in [
            (ActKind::Take, act == ActKind::HandBack),
            (ActKind::HandBack, act == ActKind::Take),
            (ActKind::Answer, act == ActKind::Take),
        ] {
            let expected = if offered { Admission::Allowed } else { Admission::Refused(Refusal::NotOffered) };
            assert_eq!(crate::acts::offered(object, kind), expected);
        }
    }
}

#[test]
fn person_take_abandons_older_matching_reads_without_discarding_other_obligations() {
    let (mut domain, _) = live();
    let original = document(None, &[ActKind::Take]);
    let id = install(&mut domain, Body::OwnerPersonTask(Box::new(original.clone())));
    let mut out = Queue::with_capacity(crate::max_out(&LIMITS));
    let older = domain.reads.ask(domain.generation, Source::Card(id), Read::PersonTask { task: 8 }, &mut out).unwrap();
    let other = domain.reads.ask(domain.generation, Source::Card(id), Read::PersonTask { task: 9 }, &mut out).unwrap();
    let request = send(&mut domain, ActKind::Take);
    let rows = answered(&mut domain, request, Answer::Done(Outcome::Changed));
    assert_eq!(rows.len(), 1);
    assert_ne!(read_token(&rows), older.token());
    assert!(domain.reads.pending(&Read::PersonTask { task: 9 }));
    let current = document(Some(u64::MAX), &[ActKind::Answer]);
    drive(
        &mut domain,
        Time::ZERO,
        Event::Read {
            read: read_token(&rows),
            result: ReadResult::Value(ReadValue::OwnerPersonTask(Box::new(current.clone()))),
        },
    );
    drive(
        &mut domain,
        Time::ZERO,
        Event::Read { read: older.token(), result: ReadResult::Value(ReadValue::OwnerPersonTask(Box::new(original))) },
    );
    assert_eq!(domain.objects().get(id).unwrap().body, Some(Body::OwnerPersonTask(Box::new(current))));
    assert!(domain.reads.pending(&Read::PersonTask { task: 9 }));
    let receipt = domain.reads.terminal(other.token()).unwrap();
    assert!(!receipt.abandoned);
    assert!(domain.reads.terminal(older.token()).is_none());
}

#[test]
fn person_take_hidden_cards_notify_without_refreshing_and_full_reads_refuse_without_resending() {
    let (mut domain, _) = live();
    install(&mut domain, Body::OwnerPersonTask(Box::new(document(None, &[ActKind::Take]))));
    let request = send(&mut domain, ActKind::Take);
    drive(&mut domain, Time::ZERO, Event::Went { place: crate::address::Place::Elsewhere });
    let answer = Answer::Done(Outcome::Changed);
    assert!(answered(&mut domain, request, answer.clone()).is_empty());
    assert!(domain.objects().find(ObjectKey::PersonTask(8)).is_none());
    let (notice, ()) = domain.notices().ids().iter().next().unwrap();
    assert_eq!(
        domain.notices().get(*notice).unwrap().kind,
        crate::notices::kept::Kind::Answer { object: ObjectKey::PersonTask(8), answer }
    );

    let (mut domain, _) = live();
    let original = document(None, &[ActKind::Take]);
    let id = install(&mut domain, Body::OwnerPersonTask(Box::new(original.clone())));
    domain.reads.reclaim();
    for task in 1..=LIMITS.reads {
        let mut out = Queue::with_capacity(crate::max_out(&LIMITS));
        let what = if task == 1 { Read::PersonTask { task: 8 } } else { Read::Task { task: u64::from(task) } };
        assert!(domain.reads.ask(domain.generation, Source::Card(id), what, &mut out).is_some());
    }
    let request = send(&mut domain, ActKind::Take);
    let rows = answered(&mut domain, request, Answer::Done(Outcome::Changed));
    assert_eq!(rows.as_ref(), &[Request::Fact { fact: crate::facts::Fact::Refused { which: Limit::Reads } }]);
    assert_eq!(domain.objects().get(id).unwrap().body, Some(Body::OwnerPersonTask(Box::new(original))));
    assert_eq!(domain.objects().get(id).unwrap().card, Card::Open);
    assert!(!domain.reads.pending(&Read::PersonTask { task: 8 }));
    let (notice, ()) = domain.notices().ids().iter().next().unwrap();
    assert_eq!(domain.notices().get(*notice).unwrap().kind, crate::notices::kept::Kind::Refused(Limit::Reads));
    assert!(answered(&mut domain, request, Answer::Done(Outcome::Changed)).is_empty());
}
