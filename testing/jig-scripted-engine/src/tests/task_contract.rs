use super::*;
use crate::payload::{self, Payload};
use core::mem::size_of;

#[test]
fn task_contract_payloads_price_reserved_choices_fields_and_whole_opaque_approval_bytes() {
    let mut fields = List::with_capacity(2);
    fields.push(FieldRule { name: vec![0xff; 32].into_boxed_slice(), max: u32::MAX }).unwrap();
    let mut choices = List::with_capacity(2);
    choices.push(VerdictChoice { code: u32::MAX, words: u32::MAX, followups: u32::MAX, fields }).unwrap();
    let mut path = List::with_capacity(2);
    path.push(vec![0xff; 32].into_boxed_slice()).unwrap();
    for contract in [
        TaskContractDocument::Report { words: u32::MAX },
        TaskContractDocument::Approval {
            state: vec![0xff; 32].into_boxed_slice(),
            part: OwnerPart { connector: u16::MAX, kind: OwnerPartKind::Resource, key: OwnerPartKey::Path(path) },
        },
        TaskContractDocument::Verdict { choices, cut: true },
        TaskContractDocument::Change { connector: u16::MAX, kind: u16::MAX, words: u32::MAX },
    ] {
        let mut row = task();
        row.result_contract = contract;
        row.goal = Some(u64::MAX);
        let mut bounds = Bounds { rows: 2, text: 32, document_bytes: u64::MAX, ..Bounds::default() };
        let before = row.clone();
        let whole = u64::try_from(size_of::<Task>()).unwrap() + row.heap(&bounds).unwrap();
        bounds.document_bytes = whole;
        assert!(payload::fits(&row, &bounds));
        bounds.document_bytes = whole - 1;
        assert!(!payload::fits(&row, &bounds));
        assert_eq!(row, before);
    }
    let mut row = task();
    row.result_contract = TaskContractDocument::Verdict { choices: List::with_capacity(3), cut: false };
    let bounds = Bounds { rows: 2, text: 32, document_bytes: u64::MAX, ..Bounds::default() };
    assert!(!payload::fits(&row, &bounds), "reserved omitted choices still count");
    let mut fields = List::with_capacity(3);
    fields.push(FieldRule { name: Box::from(*b"field"), max: u32::MAX }).unwrap();
    row.result_contract = TaskContractDocument::Verdict {
        choices: list(vec![VerdictChoice { code: 0, words: 0, followups: 0, fields }]),
        cut: false,
    };
    assert!(!payload::fits(&row, &bounds), "reserved nested field rules still count");
    row.result_contract = TaskContractDocument::Approval {
        state: vec![0; 33].into_boxed_slice(),
        part: OwnerPart { connector: 0, kind: OwnerPartKind::Result, key: OwnerPartKey::Number(0) },
    };
    assert!(!payload::fits(&row, &bounds), "opaque state is admitted whole");
}
