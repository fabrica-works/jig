//! Independent whole Question allocation and malformed-packet proofs.
use super::{Payload, fits};
use crate::Bounds;
use core::mem::size_of;
use jig_client_domain::contract::*;
use skein_lib::{List, Wall, bytes::copy_of};
fn question(from: Option<OwnerRequester>) -> Question {
    let mut dependencies = List::with_capacity(3);
    dependencies.push(u64::MAX).unwrap();
    let mut acts = List::with_capacity(2);
    acts.push(Offered { act: ActKind::Answer, admission: Admission::Allowed }).unwrap();
    Question {
        task: Chip {
            task: u64::MAX,
            title: DocumentText::complete(Text::new(copy_of(UTF8_TEXTS[0]), 3).unwrap()),
            executor: Executor::Agent { charter: u32::MAX },
            phase: Phase::Waiting,
            details: Box::new(ChipPhase::Waiting { dependencies, hold: None }),
        },
        from,
        message: u64::MAX,
        words: Text::new(copy_of(UTF8_TEXTS[1]), 3).unwrap(),
        since: Wall::from_nanos(u64::MAX),
        acts,
        ending: None,
    }
}
fn heap() -> u64 {
    6_u64
        .checked_add(u64::try_from(size_of::<ChipPhase>()).unwrap())
        .unwrap()
        .checked_add(List::<u64>::worst_case(3).unwrap())
        .unwrap()
        .checked_add(List::<Offered>::worst_case(2).unwrap())
        .unwrap()
}
#[test]
fn question_sender_whole_box_prices_reserved_chip_and_offer_allocations_at_exact_byte_ceiling() {
    for from in [
        None,
        Some(OwnerRequester::Task(0)),
        Some(OwnerRequester::Task(u64::MAX)),
        Some(OwnerRequester::Person(0)),
        Some(OwnerRequester::Person(u64::MAX)),
        Some(OwnerRequester::Deployment { project: 0 }),
        Some(OwnerRequester::Deployment { project: u32::MAX }),
    ] {
        let value = question(from);
        let packet = ReadValue::Question(Box::new(value.clone()));
        let before = packet.clone();
        let mut bounds = Bounds { text: 3, rows: 3, document_bytes: u64::MAX, ..Bounds::default() };
        assert_eq!(value.heap(&bounds), Some(heap()));
        let allocation = u64::try_from(size_of::<Question>()).unwrap().checked_add(heap()).unwrap();
        assert_eq!(packet.heap(&bounds), Some(allocation));
        bounds.document_bytes = u64::try_from(size_of::<ReadValue>()).unwrap().checked_add(allocation).unwrap();
        assert!(fits(&packet, &bounds));
        bounds.document_bytes = bounds.document_bytes.checked_sub(1).unwrap();
        assert!(!fits(&packet, &bounds));
        assert_eq!(packet, before);
        assert_eq!(value.from, from);
        assert_eq!(value.task.title.bytes.bytes(), UTF8_TEXTS[0]);
        assert_eq!(value.words.bytes(), UTF8_TEXTS[1]);
        assert_eq!(value.acts.capacity(), 2);
    }
}
#[test]
fn question_sender_malformed_nested_capacity_and_words_are_refused_whole_without_mutation() {
    let mut bounds = Bounds { text: 3, rows: 3, document_bytes: u64::MAX, ..Bounds::default() };
    let value = question(Some(OwnerRequester::Deployment { project: u32::MAX }));
    let before = value.clone();
    assert!(fits(&ReadValue::Question(Box::new(value.clone())), &bounds));
    bounds.rows = 2;
    assert!(!fits(&ReadValue::Question(Box::new(value.clone())), &bounds), "unused dependency capacity counts");
    bounds.rows = 3;
    bounds.text = 2;
    assert!(!fits(&ReadValue::Question(Box::new(value.clone())), &bounds));
    assert_eq!(value, before);
    let mut empty_offers = value.clone();
    // Offers use the closed vocabulary ceiling, independently of rows=3.
    // Keep the original unused reserved-capacity refusal at its actual bound.
    empty_offers.acts = List::with_capacity(MAX_OFFERED_ACTS.checked_add(1).unwrap());
    bounds.text = 3;
    assert!(!fits(&ReadValue::Question(Box::new(empty_offers)), &bounds), "reserved empty offer capacity is checked");
    let mut malformed = value;
    malformed.words = Text::new(copy_of(b"abcd"), 4).unwrap();
    assert!(!fits(&ReadValue::Question(Box::new(malformed)), &bounds));
}

const UTF8_TEXTS: [&[u8]; 2] = [b"~\0^", b"^\0~"];
