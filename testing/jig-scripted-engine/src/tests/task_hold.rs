use super::*;
use crate::payload::{self, Payload};
use core::mem::size_of;

#[test]
fn task_hold_parent_prices_whole_owner_box_reserved_source_and_rejected_words_at_exact_bounds() {
    let bounds = Bounds { rows: 2, text: 32, document_bytes: u64::MAX, ..Bounds::default() };
    let mut row = task();
    row.hold = None;
    let absent = row.heap(&bounds).unwrap();
    let mut path = List::with_capacity(2);
    path.push(Box::<[u8]>::from([255_u8; 32])).unwrap();
    let held = OwnerChipHold {
        source: Some(OwnerPart { connector: u16::MAX, kind: OwnerPartKind::Hold, key: OwnerPartKey::Path(path) }),
        reason: HoldCategory::Uncertain { entry: u64::MAX },
        revision: u64::MAX,
        before: OwnerPriorPhase::Closing(ClosingStage::Effects),
        escalation: Some(OwnerHoldEscalation::Rejected {
            by: u64::MAX,
            reason: DocumentText { bytes: text(&[254_u8; 32]), cut: true },
        }),
        tries: OwnerFailureTries { transient: 1, permanent: 2, run: 3, agent: 4, lost: 5, invalid: u32::MAX },
        hold_wait_since: None,
    };
    row.hold = Some(TaskHold::Owner(Box::new(held.clone())));
    let expected =
        u64::try_from(size_of::<OwnerChipHold>()).unwrap() + 2 * u64::try_from(size_of::<Box<[u8]>>()).unwrap() + 64;
    assert_eq!(row.heap(&bounds).unwrap() - absent, expected);
    let exact = u64::try_from(size_of::<Task>()).unwrap() + absent + expected;
    let before = row.clone();
    assert!(payload::fits(&row, &Bounds { document_bytes: exact, ..bounds }));
    assert!(!payload::fits(&row, &Bounds { document_bytes: exact - 1, ..bounds }));
    assert!(!payload::fits(&row, &Bounds { rows: 1, ..bounds }));
    assert!(!payload::fits(&row, &Bounds { text: 31, ..bounds }));
    assert_eq!(row, before);
    assert_eq!(row.hold, Some(TaskHold::Owner(Box::new(held))));
    let legacy = Hold {
        revision: Revision { value: 7 },
        reason: HoldReason::Budget { spent: 1, limit: 1 },
        before: Phase::Running,
        since: Wall::EPOCH,
        result: Some(ResultValue::Report(DocumentText::complete(text(b"legacy result")))),
        spend: List::with_capacity(2),
    };
    let legacy_heap = legacy.heap(&bounds).unwrap();
    row.hold = Some(TaskHold::Document(Box::new(legacy.clone())));
    assert_eq!(row.heap(&bounds).unwrap() - absent, u64::try_from(size_of::<Hold>()).unwrap() + legacy_heap);
    assert_eq!(row.hold, Some(TaskHold::Document(Box::new(legacy))));
}
