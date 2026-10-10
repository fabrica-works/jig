//! Receiving never reconstructs or clips a whole owner authority.
use super::*;
use crate::payload::{self, Payload};
use core::mem::size_of;

fn supplied() -> OwnerAuthorityDocument {
    let mut segments = List::with_capacity(2);
    segments.push(vec![0xff; 32].into_boxed_slice()).unwrap();
    let pattern = OwnerPattern { segments, last: OwnerPatternEnd::Exact(vec![0xfe; 32].into_boxed_slice()) };
    let mut value = owner_authority();
    value.authority.tools = u64::MAX;
    value.authority.notes = u8::MAX;
    value.authority.grants = List::with_capacity(2);
    value.authority.grants.push(OwnerGrant { connector: u16::MAX, kind: u16::MAX, pattern: pattern.clone() }).unwrap();
    value.authority.delegation.kinds = List::with_capacity(2);
    value.authority.delegation.kinds.push(OwnerAuthorityExecutor::Procedure(u32::MAX)).unwrap();
    value.authority.note_resources = List::with_capacity(2);
    value.authority.note_resources.push(OwnerResourceScope { connector: u16::MAX, pattern }).unwrap();
    value
}

#[test]
fn task_authority_receiving_prices_whole_documents_replacements_and_reserved_nested_permissions() {
    let mut bounds = Bounds { rows: 2, text: 32, document_bytes: u64::MAX, ..Bounds::default() };
    for cut in [false, true] {
        let mut row = task();
        row.authority = supplied();
        row.authority.cut = cut;
        let before = row.clone();
        let bytes = u64::try_from(size_of::<Task>()).unwrap() + row.heap(&bounds).unwrap();
        bounds.document_bytes = bytes;
        assert!(payload::fits(&row, &bounds));
        bounds.document_bytes -= 1;
        assert!(!payload::fits(&row, &bounds));
        assert_eq!(row, before);
        bounds.document_bytes = u64::MAX;
        let ask = Ask::Amend {
            project: 1,
            task: 8,
            revision: Revision { value: 1 },
            release: None,
            change: Amendment {
                addressed: None,
                spec: None,
                wake: None,
                dependencies: None,
                authority: Some(row.authority.authority.clone()),
                instructions: None,
            },
        };
        let bytes = u64::try_from(size_of::<Ask>()).unwrap() + ask.heap(&bounds).unwrap();
        bounds.document_bytes = bytes;
        assert!(payload::fits(&ask, &bounds));
        bounds.document_bytes -= 1;
        assert!(!payload::fits(&ask, &bounds));
        bounds.document_bytes = u64::MAX;
    }
    for dimension in 0_u8..5 {
        let mut row = task();
        row.authority = supplied();
        match dimension {
            0 => row.authority.authority.grants = List::with_capacity(3),
            1 => row.authority.authority.delegation.kinds = List::with_capacity(3),
            2 => row.authority.authority.note_resources = List::with_capacity(3),
            3 => row.authority.authority.grants.get_mut(0).unwrap().pattern.segments = List::with_capacity(3),
            4 => {
                row.authority.authority.note_resources.get_mut(0).unwrap().pattern.last =
                    OwnerPatternEnd::Open(vec![0; 33].into_boxed_slice());
            }
            _ => unreachable!(),
        }
        let before = row.clone();
        assert!(!payload::fits(&row, &bounds));
        assert_eq!(row, before);
    }
}
