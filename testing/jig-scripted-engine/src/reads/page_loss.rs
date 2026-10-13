use crate::{Bounds, Engine, Script};
use jig_domain_client::contract::*;
use skein_lib::List;

use crate::payload::{self, Payload};
use core::mem::size_of;

fn page_values(capacity: u32, cut: bool) -> [ReadValue; 8] {
    let before = Some(Cursor::new(Box::from(UTF8_TEXTS[0]), 1024).expect("bounded opaque cursor"));
    let after = Some(Cursor::new(Box::from(b"after/opaque".as_slice()), 1024).expect("bounded opaque cursor"));
    [
        ReadValue::Ended(Box::new(ObjectPage {
            cut,
            rows: List::with_capacity(capacity),
            before: before.clone(),
            after: after.clone(),
        })),
        ReadValue::History(Box::new(HistoryPage {
            cut,
            rows: List::with_capacity(capacity),
            before: before.clone(),
            after: after.clone(),
        })),
        ReadValue::Activity(Box::new(ActivityPage {
            cut,
            rows: List::with_capacity(capacity),
            before: before.clone(),
            after: after.clone(),
        })),
        ReadValue::Inbox(Box::new(InboxPage {
            cut,
            rows: List::with_capacity(capacity),
            before: before.clone(),
            after: after.clone(),
        })),
        ReadValue::Conversations(Box::new(ConversationPage {
            cut,
            rows: List::with_capacity(capacity),
            before: before.clone(),
            after: after.clone(),
        })),
        ReadValue::Goals(Box::new(GoalPage {
            cut,
            rows: List::with_capacity(capacity),
            before: before.clone(),
            after: after.clone(),
        })),
        ReadValue::Spend(Box::new(SpendPage {
            cut,
            rows: List::with_capacity(capacity),
            before: before.clone(),
            after: after.clone(),
        })),
        ReadValue::Notes(Box::new(NotePage { cut, rows: List::with_capacity(capacity), before, after })),
    ]
}

#[test]
fn page_loss_envelopes_admit_exact_whole_boxes_and_refuse_reserved_capacity_without_trimming() {
    let original = Bounds { rows: 4, document_bytes: u64::MAX, ..Bounds::default() };
    for cut in [false, true] {
        for value in page_values(4, cut) {
            let before = value.clone();
            let whole = u64::try_from(size_of::<ReadValue>()).unwrap() + value.heap(&original).unwrap();
            let exact = Bounds { document_bytes: whole, ..original };
            assert!(payload::fits(&value, &exact));
            assert!(!payload::fits(&value, &Bounds { document_bytes: whole - 1, ..exact }));
            assert!(!payload::fits(&value, &Bounds { rows: 3, ..original }));
            assert_eq!(value, before, "refusing any allocated row capacity keeps the source whole");
            let mut engine = Engine::new(Script::default(), 90).unwrap();
            // This is the actual final read boundary, not a copied size formula.
            engine.bounds = exact;
            assert_eq!(engine.checked_read(Some(value.clone())), Ok(value.clone()));
            engine.bounds.document_bytes -= 1;
            assert_eq!(engine.checked_read(Some(value.clone())), Err(ReadNone::Busy));
            assert_eq!(value, before);
        }
    }
}

const UTF8_TEXTS: [&[u8]; 1] = [b"\x7e\0before"];
