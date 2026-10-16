use super::{bytes, fits, page_bytes, page_fits, worst_case};
use crate::Bounds;
use core::mem::size_of;
use jig_client_domain::contract::*;
use skein_lib::{List, Wall, bytes::copy_of};
fn doc(value: &[u8]) -> DocumentText {
    DocumentText { bytes: Text::new(copy_of(value), 8).unwrap(), cut: true }
}
fn chip() -> Chip {
    Chip {
        task: u64::MAX,
        title: doc(b"chip"),
        executor: Executor::Party { party: 0 },
        phase: Phase::Waiting,
        details: Box::new(ChipPhase::Waiting { dependencies: List::with_capacity(3), hold: None }),
    }
}
fn budget() -> OwnerBudget {
    OwnerBudget {
        numbers: OwnerNumbers { budget: 0, spent: 0, spent_below: 0, reserved: 0 },
        left: 0,
        funder: Funder::Period { project: 0, period: 0 },
        deadline: None,
    }
}
fn row() -> OwnerInboxEntry {
    OwnerInboxEntry {
        reply: Some(Box::new(ConversationLine {
            chip: chip(),
            project: 0,
            first_words: doc(UTF8_TEXTS[0]),
            runtime: None,
            delegates: 0,
            budget: budget(),
            unread: None,
            cached_replies: u32::MAX,
            seen: 4,
            last_reply: 9,
            changed: None,
        })),
        task: OwnerTaskLink::Retained { chip: chip(), at: None },
        project: Some(0),
        object: OwnerInboxObject::Reply { message: u64::MAX },
        activity: Some(0),
        at: None,
        unread: None,
        cursor: OwnerInboxCursor::Inbox {
            task: u64::MAX,
            object: OwnerInboxObject::Reply { message: u64::MAX },
            at: Some(Wall::EPOCH),
        },
    }
}
fn whole() -> OwnerInbox {
    let mut updates = List::with_capacity(3);
    updates.push(row()).unwrap();
    OwnerInbox {
        party: u64::MAX,
        needs: List::with_capacity(3),
        updates,
        next: Some(OwnerInboxCursor::Number { number: u64::MAX }),
        cached_needs: u32::MAX,
        cached_updates: Count { value: 0, limited: true },
        results_seen: u64::MAX,
        updates_time_complete: false,
        selection: None,
    }
}
fn bounds() -> Bounds {
    Bounds { rows: 3, text: 8, ..Bounds::default() }
}
#[test]
fn owner_inbox_recursive_reserved_groups_reply_and_link_boxes_fit_exact_and_refuse_one_under_unchanged() {
    let source = whole();
    let unchanged = source.clone();
    let chip_heap = u64::try_from(size_of::<ChipPhase>()).unwrap() + List::<u64>::worst_case(3).unwrap() + 4;
    let expected = u64::try_from(size_of::<OwnerInbox>()).unwrap()
        + List::<OwnerInboxEntry>::worst_case(3).unwrap() * 2
        + chip_heap
        + u64::try_from(size_of::<ConversationLine>()).unwrap()
        + chip_heap
        + 8;
    assert_eq!(bytes(&source, &bounds()), Some(expected));
    for (amount, admitted) in [(expected - 1, false), (expected, true), (expected + 1, true)] {
        let b = Bounds { document_bytes: amount, ..bounds() };
        assert_eq!(fits(&source, &b), admitted);
        assert_eq!(worst_case(&b), Some(amount));
    }
    assert_eq!(source, unchanged);
    assert!(bytes(&source, &Bounds { rows: 2, ..bounds() }).is_none());
    assert!(bytes(&source, &Bounds { text: 7, ..bounds() }).is_none());
}
#[test]
fn owner_inbox_whole_page_opaque_bytes_allocation_boundaries_and_stored_links_are_independent() {
    let mut rows = List::with_capacity(3);
    let mut value = row();
    value.reply = None;
    value.task = OwnerTaskLink::Stored { number: u64::MAX, status: None };
    rows.push(value.clone()).unwrap();
    let page = OwnerInboxPage {
        selection: None,
        cut: true,
        rows,
        before: Some(Cursor::new(copy_of(UTF8_TEXTS[0]), 8).unwrap()),
        after: Some(Cursor::new(copy_of(UTF8_TEXTS[1]), 8).unwrap()),
    };
    let expected =
        u64::try_from(size_of::<OwnerInboxPage>()).unwrap() + List::<OwnerInboxEntry>::worst_case(3).unwrap() + 16;
    let unchanged = page.clone();
    assert_eq!(page_bytes(&page, &bounds()), Some(expected));
    assert!(page_fits(&page, &Bounds { document_bytes: expected, ..bounds() }));
    assert!(!page_fits(&page, &Bounds { document_bytes: expected - 1, ..bounds() }));
    assert!(!page_fits(&page, &Bounds { text: 7, ..bounds() }));
    assert_eq!(page, unchanged);
    assert_eq!(page.rows.get(0), Some(&value));
    let source = OwnerInbox {
        party: 0,
        needs: List::with_capacity(3),
        updates: List::with_capacity(3),
        next: None,
        cached_needs: 0,
        cached_updates: Count { value: u32::MAX, limited: false },
        results_seen: 0,
        updates_time_complete: true,
        selection: None,
    };
    let root = u64::try_from(size_of::<OwnerInbox>()).unwrap();
    assert_eq!(bytes(&source, &bounds()), Some(root + List::<OwnerInboxEntry>::worst_case(3).unwrap() * 2));
    assert!(worst_case(&Bounds { document_bytes: root - 1, ..bounds() }).is_none());
}
#[test]
fn owner_inbox_selection_roots_exceed_page_rows_without_clipping_and_price_whole_reserved_capacity() {
    let mut source = whole();
    let before = bytes(&source, &bounds()).unwrap();
    let mut goals = List::with_capacity(5);
    for number in [u64::MAX, 0, 77] {
        goals.push(number).unwrap();
    }
    let mut selection = List::with_capacity(3);
    selection.push(OwnerInboxSelection { cursor: source.updates.get(0).unwrap().cursor, goals }).unwrap();
    source.selection = Some(selection);
    let original = source.clone();
    let expected = before + List::<OwnerInboxSelection>::worst_case(3).unwrap() + List::<u64>::worst_case(5).unwrap();
    assert_eq!(bytes(&source, &bounds()), Some(expected));
    assert!(fits(&source, &Bounds { document_bytes: expected, ..bounds() }));
    assert!(!fits(&source, &Bounds { document_bytes: expected - 1, ..bounds() }));
    assert_eq!(source, original);
    let mut absent = whole();
    absent.needs = List::with_capacity(0);
    absent.updates = List::with_capacity(0);
    let absent_bytes = bytes(&absent, &bounds()).unwrap();
    let mut supplied = absent.clone();
    supplied.selection = Some(List::with_capacity(3));
    assert_ne!(absent, supplied);
    assert_eq!(bytes(&supplied, &bounds()), Some(absent_bytes + List::<OwnerInboxSelection>::worst_case(3).unwrap()));
    assert!(bytes(&supplied, &Bounds { rows: 2, ..bounds() }).is_none());
}

const UTF8_TEXTS: [&[u8]; 2] = [&[b'~'; 8], &[b'!'; 8]];
