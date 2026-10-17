use super::*;
use crate::payload::{self, Payload};
use core::mem::size_of;

#[test]
fn scripted_task_ending_refusals_keep_whole_owner_facts_after_keyed_durability_and_replay() {
    let ending = OwnerChipEnding::Cancelled {
        reason: DocumentText { bytes: text(UTF8_TEXTS[0]), cut: true },
        result: Some(ResultValue::Approved {
            state: Box::from([255_u8; 32]),
            words: Some(DocumentText { bytes: text(b""), cut: true }),
        }),
        by: OwnerRequester::Deployment { project: u32::MAX },
        at: Wall::from_nanos(u64::MAX),
    };
    let mut setup = script();
    let mut row = task();
    row.result = Some(TaskResult::Ending(Box::new(ending.clone())));
    setup.truth.objects.insert(ObjectKey::Task(8), ReadValue::Task(Box::new(row)));
    let ask = Ask::Cancel { project: 1, task: 8, reason: text(b"cancel"), grace: None };
    let expected = Answer::Refused { refusal: Refusal::Ended { how: Ending::Task(Box::new(ending)) }, at: None };
    let mut engine = Engine::new(setup, 1).unwrap();
    send(&mut engine, 0, 11, 1, 1, ask.clone());
    send(&mut engine, 1, 12, 1, 2, ask.clone());
    assert!(engine.poll(Time::from_nanos(9_999_999)).is_empty());
    let answers = engine.poll(Time::from_nanos(10_000_001));
    assert_eq!(answers.len(), 2);
    assert_eq!(final_answer(&answers[0]), &expected);
    assert_eq!(final_answer(&answers[1]), &expected);
    assert_eq!(engine.commits().len(), 1);
    assert_eq!(engine.commits()[0].ask, Some(ask.clone()));
    send(&mut engine, 20_000_000, 11, 1, 3, ask);
    let replay = engine.poll(Time::from_nanos(20_000_000));
    assert_eq!(replay.len(), 1);
    assert_eq!(final_answer(&replay[0]), &expected);
    assert_eq!(engine.commits().len(), 1);
    let Answer::Refused { refusal, .. } = expected else { panic!("expected refusal") };
    assert_eq!(EntranceRefusal::new(refusal.clone()), Err(refusal));
}

#[test]
fn whole_task_endings_price_actual_boxes_raw_state_text_loss_and_optional_retained_result_without_trimming() {
    let bounds = Bounds { rows: 2, text: 32, document_bytes: u64::MAX, ..Bounds::default() };
    for retained in [
        None,
        Some(ResultValue::Approved {
            state: Box::from([255_u8; 32]),
            words: Some(DocumentText { bytes: text(b""), cut: true }),
        }),
    ] {
        let ending = OwnerChipEnding::Cancelled {
            reason: DocumentText { bytes: text(UTF8_TEXTS[0]), cut: true },
            result: retained.clone(),
            by: OwnerRequester::Deployment { project: u32::MAX },
            at: Wall::from_nanos(u64::MAX),
        };
        let mut row = task();
        let absent_heap = row.heap(&bounds).unwrap();
        row.result = Some(TaskResult::Ending(Box::new(ending.clone())));
        let expected =
            u64::try_from(size_of::<OwnerChipEnding>()).unwrap() + 32 + if retained.is_some() { 32 } else { 0 };
        assert_eq!(row.heap(&bounds).unwrap() - absent_heap, expected);
        let exact = u64::try_from(size_of::<Task>()).unwrap() + absent_heap + expected;
        let before = row.clone();
        assert!(payload::fits(&row, &Bounds { document_bytes: exact, ..bounds }));
        assert!(!payload::fits(&row, &Bounds { document_bytes: exact - 1, ..bounds }));
        assert!(!payload::fits(&row, &Bounds { text: 31, ..bounds }));
        assert_eq!(row, before);
        assert_eq!(row.result, Some(TaskResult::Ending(Box::new(ending))));
    }
    let value = Result {
        task: task().chip,
        ending: Ending::Done,
        value: Some(ResultValue::Report(DocumentText { bytes: text(b"whole result"), cut: true })),
        at: None,
        acts: List::with_capacity(2),
        details: None,
    };
    let mut row = task();
    let absent = row.heap(&bounds).unwrap();
    row.result = Some(TaskResult::Document(Box::new(value.clone())));
    assert_eq!(
        row.heap(&bounds).unwrap() - absent,
        u64::try_from(size_of::<Result>()).unwrap() + value.heap(&bounds).unwrap()
    );
    assert_eq!(row.result, Some(TaskResult::Document(Box::new(value))));
}

const UTF8_TEXTS: [&[u8]; 1] = [&[b'!'; 32]];
