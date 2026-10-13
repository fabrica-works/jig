use super::{bytes, fits, worst_case};
use crate::{
    Bounds,
    payload::{self, Payload},
};
use jig_domain_client::contract::*;
use skein_lib::{List, Wall};
fn chip(n: u64) -> Chip {
    Chip {
        task: n,
        title: DocumentText { bytes: Text::new(Box::from(b"tree".as_slice()), 8).unwrap(), cut: true },
        executor: Executor::Agent { charter: u32::MAX },
        phase: Phase::Running,
        details: Box::new(ChipPhase::Running { attempt: u64::MAX, since: Wall::EPOCH, tool: None }),
    }
}
fn tree() -> OwnerTree {
    let mut rows = List::with_capacity(3);
    let mut dependencies = List::with_capacity(3);
    dependencies.push(OwnerTaskLink::Live(chip(3))).unwrap();
    dependencies.push(OwnerTaskLink::Retained { chip: chip(4), at: None }).unwrap();
    dependencies.push(OwnerTaskLink::Stored { number: u64::MAX, status: None }).unwrap();
    let mut proposals = List::with_capacity(3);
    proposals.push(ProposalKey { proposer: Proposer::Party(u64::MAX), number: u64::MAX }).unwrap();
    rows.push(OwnerTreeRow {
        chip: chip(u64::MAX),
        parent: 0,
        depth: u32::MAX,
        proposals,
        spent: u64::MAX,
        dependencies,
        cut: true,
    })
    .unwrap();
    OwnerTree {
        root: chip(0),
        rows,
        folded: List::with_capacity(3),
        held_below: u32::MAX,
        next: Some(Cursor::new(Box::from(UTF8_TEXTS[0]), 8).unwrap()),
    }
}
fn bounds() -> Bounds {
    Bounds { rows: 3, text: 8, document_bytes: 65536, ..Bounds::default() }
}
#[test]
fn owner_tree_admission_prices_every_reserved_collection_link_box_and_opaque_cursor_whole() {
    let value = tree();
    let b = bounds();
    let expected = u64::try_from(core::mem::size_of::<OwnerTree>()).unwrap()
        + value.root.heap(&b).unwrap()
        + List::<OwnerTreeRow>::worst_case(3).unwrap()
        + value.rows.as_slice()[0].chip.heap(&b).unwrap()
        + List::<ProposalKey>::worst_case(3).unwrap()
        + List::<OwnerTaskLink>::worst_case(3).unwrap()
        + value.rows.as_slice()[0].dependencies.as_slice()[0].heap(&b).unwrap()
        + value.rows.as_slice()[0].dependencies.as_slice()[1].heap(&b).unwrap()
        + List::<OwnerTreeFold>::worst_case(3).unwrap()
        + 3;
    assert_eq!(bytes(&value, &b), Some(expected));
    for (size, accepted) in [(expected - 1, false), (expected, true), (expected + 1, true)] {
        assert_eq!(fits(&value, &Bounds { document_bytes: size, ..b }), accepted);
    }
    let delivery = Delivery::OwnerTree(Box::new(value.clone()));
    let size = u64::try_from(core::mem::size_of::<Delivery>()).unwrap() + delivery.heap(&b).unwrap();
    assert!(!payload::fits(&delivery, &Bounds { document_bytes: size - 1, ..b }));
    assert!(payload::fits(&delivery, &Bounds { document_bytes: size, ..b }));
    assert_eq!(value, tree());
    assert_eq!(worst_case(&b), Some(b.document_bytes));
}
#[test]
fn owner_tree_over_reserved_empty_nested_lists_and_cursor_bytes_refuse_without_shortening() {
    let b = bounds();
    let value = tree();
    assert!(bytes(&value, &Bounds { rows: 2, ..b }).is_none());
    let mut oversized = value.clone();
    oversized.rows = List::with_capacity(4);
    assert!(!fits(&oversized, &b));
    oversized = value.clone();
    oversized.folded = List::with_capacity(4);
    assert!(!fits(&oversized, &b));
    oversized = value.clone();
    oversized.rows.get_mut(0).unwrap().dependencies = List::with_capacity(4);
    assert!(!fits(&oversized, &b));
    oversized = value.clone();
    oversized.rows.get_mut(0).unwrap().proposals = List::with_capacity(4);
    assert!(!fits(&oversized, &b));
    oversized = value.clone();
    oversized.next = Some(Cursor::new(Box::from(b"123456789".as_slice()), 9).unwrap());
    assert!(!fits(&oversized, &b));
    let mut empty = value.clone();
    empty.rows = List::with_capacity(0);
    empty.next = None;
    assert!(fits(&empty, &b));
    assert!(bytes(&empty, &b).unwrap() < bytes(&value, &b).unwrap());
    assert_eq!(value, tree());
}
#[test]
fn owner_tree_empty_dependencies_keep_row_loss_and_absent_or_empty_cursor_independent_of_reserved_folds() {
    let b = bounds();
    for cut in [false, true] {
        for next in [None, Some(Cursor::new(Box::from(b"".as_slice()), 8).unwrap())] {
            let mut value = tree();
            value.rows.get_mut(0).unwrap().dependencies = List::with_capacity(3);
            value.rows.get_mut(0).unwrap().cut = cut;
            value.next = next.clone();
            let original = value.clone();
            let amount = bytes(&value, &b).unwrap();
            assert!(fits(&value, &Bounds { document_bytes: amount, ..b }));
            assert!(!fits(&value, &Bounds { document_bytes: amount - 1, ..b }));
            assert_eq!(value, original);
            assert_eq!(value.rows.get(0).unwrap().cut, cut);
            assert_eq!(value.next, next);
            assert_eq!(value.folded.capacity(), 3);
            assert_eq!(value.rows.get(0).unwrap().dependencies.capacity(), 3);
        }
    }
}

const UTF8_TEXTS: [&[u8]; 1] = [b"~\0/"];
