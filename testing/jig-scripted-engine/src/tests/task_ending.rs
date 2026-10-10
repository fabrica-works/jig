use super::*;
use crate::payload::{self, Payload};
use core::mem::size_of;

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
            reason: DocumentText { bytes: text(&[254; 32]), cut: true },
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
