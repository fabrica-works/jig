//! Independent allocator proof of the bounded reverse-live cells and whole page header.
use jig_core::reads::{self, Cursor, Page, PageRow};
use skein_lib::{List, Queue};
use skein_world::domain::heap::{self, Meter};

#[global_allocator]
static HEAP: heap::Counting = heap::Counting;

#[test]
fn conversation_owner_reverse_tail_and_both_cursor_page_cells_fit_the_counted_bound() {
    let rows = 7;
    let scratch = Queue::<PageRow>::worst_case(rows).expect("bounded reverse tail");
    let header = u64::try_from(core::mem::size_of::<Page>()).expect("page header");
    let page_cells = List::<PageRow>::worst_case(rows).expect("bounded page cells");
    let bound = scratch.checked_add(header).expect("header and scratch").checked_add(page_cells).expect("whole cells");
    let meter = Meter::new();
    meter.start();
    let tail = Queue::<PageRow>::with_capacity(rows);
    let page = Box::new(Page {
        rows: List::with_capacity(rows),
        before: Some(Cursor::ConversationsLiveBefore { before: None }),
        next: Some(Cursor::ConversationsStart),
        cut: false,
        inbox_selection: None,
    });
    let measured = meter.end();
    meter.check(measured, bound, &rows);
    assert_eq!(meter.held(), bound, "allocated capacity includes empty rows and both owner cursor cells");
    assert!(meter.held() > bound - 1);
    assert_eq!(page.rows.capacity(), rows);
    assert_eq!(tail.room(), rows);
    let mut limits = jig_core_world::world::limits().core;
    limits.documents.rows = rows;
    assert!(reads::answer_worst_case(&limits).expect("complete answer") >= header + page_cells);
    assert!(reads::worst_case(&limits).expect("complete read owner") >= bound);
    drop(page);
    drop(tail);
    assert_eq!(meter.held(), 0);
}
