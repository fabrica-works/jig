use super::*;
use crate::payload::{self, Payload};
use core::mem::size_of;

#[test]
fn task_link_payloads_price_whole_chips_and_every_reserved_reference_list() {
    for requester in [
        OwnerRequester::Task(u64::MAX),
        OwnerRequester::Person(u64::MAX),
        OwnerRequester::Deployment { project: u32::MAX },
    ] {
        let mut row = task();
        row.requester = requester;
        row.cut = true;
        row.inputs = List::with_capacity(2);
        row.inputs.push(OwnerTaskLink::Live(row.chip.clone())).unwrap();
        row.dependencies = List::with_capacity(2);
        row.dependencies.push(OwnerTaskLink::Retained { chip: row.chip.clone(), at: None }).unwrap();
        row.references = List::with_capacity(2);
        row.references.push(OwnerTaskLink::Stored { number: u64::MAX, status: None }).unwrap();
        let mut bounds = Bounds { rows: 2, document_bytes: u64::MAX, ..Bounds::default() };
        let before = row.clone();
        let whole = u64::try_from(size_of::<Task>()).unwrap() + row.heap(&bounds).unwrap();
        bounds.document_bytes = whole;
        assert!(payload::fits(&row, &bounds));
        bounds.document_bytes = whole - 1;
        assert!(!payload::fits(&row, &bounds));
        bounds.document_bytes = whole;
        assert_eq!(row.requester.heap(&bounds), Some(0));
        assert_eq!(row.references.get(0).unwrap().heap(&bounds), Some(0));
        assert_eq!(row, before);
        for field in 0..3 {
            let mut oversized = row.clone();
            match field {
                0 => oversized.inputs = List::with_capacity(3),
                1 => oversized.dependencies = List::with_capacity(3),
                2 => oversized.references = List::with_capacity(3),
                _ => unreachable!(),
            }
            bounds.document_bytes = u64::MAX;
            assert!(!payload::fits(&oversized, &bounds), "allocated empty reference rows still count");
        }
        let mut nested = row.clone();
        nested.chip.title.bytes = Text::new(vec![b't'; 4097].into_boxed_slice(), 4097).unwrap();
        nested.inputs = list(vec![OwnerTaskLink::Live(nested.chip.clone())]);
        assert!(!payload::fits(&nested, &bounds));
    }
}
