#![expect(clippy::disallowed_methods, reason = "ordinary proofs query whole native facts and original read receipts")]
use super::{
    objects::{self, live, read_token, watched},
    spine::drive,
};
use crate::{
    Event,
    action::Act,
    contract::*,
    objects::{Body, Card, Left, Source},
};
use skein_lib::{Time, Wall};
fn question(terminal: bool) -> OwnerHistorical {
    let q = objects::question();
    let answer = if terminal {
        Some(OwnerActivityLine {
            task: q.task.task,
            position: 12,
            at: Wall::from_nanos(17),
            line: OwnerActivity::Words {
                message: 9,
                by: OwnerRequester::Deployment { project: u32::MAX },
                kind: OwnerMessageKind::Answer { question: q.message },
                words: objects::text(&[255, 0]),
            },
        })
    } else {
        None
    };
    OwnerHistorical {
        key: ObjectKey::Question { task: q.task.task, message: q.message },
        value: OwnerHistoricalValue::Question { question: q, answer, ending: None },
        at: None,
        by: None,
    }
}
#[test]
fn historical_normal_reads_keep_whole_terminal_facts_and_linger_once_without_current_offers() {
    let (mut domain, page) = live();
    let supplied = question(true);
    let key = supplied.key;
    let first = watched(&mut domain, page, &[key]);
    let read = read_token(&first);
    let id = domain.objects.find(key).unwrap();
    let output = drive(
        &mut domain,
        Time::from_nanos(30),
        Event::Read { read, result: ReadResult::Value(ReadValue::Historical(Box::new(supplied.clone()))) },
    );
    assert!(output.is_empty());
    let original = Card::Leaving {
        how: Left::Historical { facts: Box::new(supplied.clone()) },
        until: Time::from_nanos(5_000_000_030),
    };
    assert_eq!(domain.objects.get(id).unwrap().card, original);
    assert_eq!(domain.objects.get(id).unwrap().body, Some(Body::Historical(Box::new(supplied))));
    assert!(
        drive(
            &mut domain,
            Time::from_nanos(40),
            Event::Act { act: Act::Intend { object: id.token(), act: ActKind::Answer } }
        )
        .is_empty()
    );
    assert!(domain.confirmation().is_none());
    assert_eq!(domain.objects.get(id).unwrap().card, original);
    drive(&mut domain, Time::from_nanos(50), Event::Read { read, result: ReadResult::None(ReadNone::Gone) });
    assert_eq!(domain.objects.get(id).unwrap().card, original);
}
#[test]
fn historical_wrong_nested_message_leaves_original_receipt_and_existing_body_available_for_the_matching_read() {
    let (mut domain, page) = live();
    let supplied = question(true);
    let key = supplied.key;
    let first = watched(&mut domain, page, &[key]);
    let read = read_token(&first);
    let mut wrong = supplied.clone();
    let OwnerHistoricalValue::Question { answer: Some(answer), .. } = &mut wrong.value else { panic!("whole answer") };
    let OwnerActivity::Words { kind, .. } = &mut answer.line else { panic!("answer words") };
    *kind = OwnerMessageKind::Answer { question: 99 };
    drive(
        &mut domain,
        Time::ZERO,
        Event::Read { read, result: ReadResult::Value(ReadValue::Historical(Box::new(wrong))) },
    );
    let id = domain.objects.find(key).unwrap();
    assert!(domain.objects.get(id).unwrap().body.is_none());
    assert!(domain.reads.slab.get(skein_lib::Id::from_token(read)).is_some());
    drive(
        &mut domain,
        Time::ZERO,
        Event::Read { read, result: ReadResult::Value(ReadValue::Historical(Box::new(supplied.clone()))) },
    );
    assert_eq!(domain.objects.get(id).unwrap().body, Some(Body::Historical(Box::new(supplied))));
}
#[test]
fn historical_pending_sourced_facts_stay_open_and_source_less_facts_prune_without_unknown_or_new_linger() {
    for drop in [false, true] {
        let (mut domain, page) = live();
        let supplied = question(false);
        let key = supplied.key;
        let first = watched(&mut domain, page, &[key]);
        let read = read_token(&first);
        if drop {
            assert!(watched(&mut domain, page, &[]).is_empty());
        }
        drive(
            &mut domain,
            Time::ZERO,
            Event::Read { read, result: ReadResult::Value(ReadValue::Historical(Box::new(supplied.clone()))) },
        );
        if drop {
            assert!(domain.objects.find(key).is_none());
        } else {
            let id = domain.objects.find(key).unwrap();
            let fact = domain.objects.get(id).unwrap();
            assert_eq!(fact.card, Card::Open);
            assert_eq!(fact.body, Some(Body::Historical(Box::new(supplied))));
            assert!(!fact.sources.is_empty());
        }
    }
}
#[test]
fn historical_disappearing_terminal_keeps_each_source_and_ignores_later_compact_refresh() {
    let (mut domain, page) = live();
    let supplied = question(true);
    let key = supplied.key;
    let first = watched(&mut domain, page, &[key]);
    let read = read_token(&first);
    assert!(watched(&mut domain, page, &[]).is_empty());
    drive(
        &mut domain,
        Time::ZERO,
        Event::Read { read, result: ReadResult::Value(ReadValue::Historical(Box::new(supplied.clone()))) },
    );
    let id = domain.objects.find(key).unwrap();
    assert!(domain.objects.put(key, Body::Question(Box::new(objects::question()))));
    assert_eq!(
        domain.objects.get(id).unwrap().card,
        Card::Leaving { how: Left::Historical { facts: Box::new(supplied) }, until: Time::from_nanos(5_000_000_000) }
    );
}
#[test]
fn historical_pending_request_and_answered_card_fences_survive_terminal_read_and_direct_put_rejects_wrong_identity() {
    let (mut domain, page) = live();
    let supplied = question(true);
    let key = supplied.key;
    let first = watched(&mut domain, page, &[key]);
    let read = read_token(&first);
    let id = domain.objects.find(key).unwrap();
    let card = Card::Deciding { request: Some(skein_lib::Token::new(999)), progress: crate::intents::Progress::Parked };
    domain.objects.get_mut(id).unwrap().card = card.clone();
    drive(
        &mut domain,
        Time::ZERO,
        Event::Read { read, result: ReadResult::Value(ReadValue::Historical(Box::new(supplied.clone()))) },
    );
    assert_eq!(domain.objects.get(id).unwrap().card, card);
    assert!(domain.objects.get(id).unwrap().body.is_none());
    let mut wrong = supplied.clone();
    wrong.key = ObjectKey::Question { task: 8, message: 2 };
    assert!(!domain.objects.put(key, Body::Historical(Box::new(wrong))));
    assert_eq!(domain.objects.get(id).unwrap().card, card);
    // A genuine body refresh is allowed, but never substitutes for a keyed answer.
    assert!(domain.objects.put(key, Body::Historical(Box::new(supplied))));
    assert_eq!(domain.objects.get(id).unwrap().card, card);
    assert!(domain.objects.named(key, Source::Watch(skein_lib::Id::from_token(page))));
    // The already durable answered-card fixture is an independent fence.
    let (mut answered, answered_page) = live();
    let first = watched(&mut answered, answered_page, &[key]);
    let read = read_token(&first);
    let id = answered.objects.find(key).unwrap();
    let answer = Answer::Done(Outcome::Changed);
    let card =
        Card::Leaving { how: Left::Answered { outcome: Outcome::Changed }, until: Time::from_nanos(5_000_000_000) };
    answered.objects.get_mut(id).unwrap().card = card.clone();
    answered.objects.get_mut(id).unwrap().answer = Some(answer.clone());
    drive(
        &mut answered,
        Time::ZERO,
        Event::Read { read, result: ReadResult::Value(ReadValue::Historical(Box::new(question(true)))) },
    );
    assert_eq!(answered.objects.get(id).unwrap().card, card);
    assert_eq!(answered.objects.get(id).unwrap().answer, Some(answer));
    assert!(answered.objects.get(id).unwrap().body.is_none());
}
