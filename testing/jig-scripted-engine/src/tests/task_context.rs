use super::*;
use crate::payload::{self, Payload};
use core::mem::size_of;

#[test]
fn task_context_parent_prices_actual_optional_box_and_all_reserved_children_without_filling_absence() {
    let bounds = Bounds { rows: 2, text: 32, document_bytes: u64::MAX, ..Bounds::default() };
    let mut row = task();
    let mut context = row.context.take().unwrap();
    let absent = row.heap(&bounds).unwrap();
    let mut path = List::with_capacity(2);
    path.push(Box::<[u8]>::from([255_u8; 32])).unwrap();
    context.parts = List::with_capacity(2);
    context.parts.push(Part::Resource { connector: u16::MAX, path }).unwrap();
    context.spend = List::with_capacity(2);
    context.writer_slots = List::with_capacity(2);
    context.escalation = None;
    context.closing = None;
    context.runtime = Runtime::Running { attempt: u64::MAX, since: Wall::EPOCH, tool: Some(text(&[254_u8; 32])) };
    row.context = Some(context.clone());
    let child = u64::try_from(size_of::<TaskContext>()).unwrap()
        + 2 * u64::try_from(size_of::<Part>()).unwrap()
        + 2 * u64::try_from(size_of::<Box<[u8]>>()).unwrap()
        + 2 * u64::try_from(size_of::<SpendLine>()).unwrap()
        + 2 * u64::try_from(size_of::<WriterSlot>()).unwrap()
        + 64;
    assert_eq!(row.heap(&bounds).unwrap() - absent, child);
    let exact = u64::try_from(size_of::<Task>()).unwrap() + absent + child;
    let before = row.clone();
    assert!(payload::fits(&row, &Bounds { document_bytes: exact, ..bounds }));
    assert!(!payload::fits(&row, &Bounds { document_bytes: exact - 1, ..bounds }));
    assert!(!payload::fits(&row, &Bounds { rows: 1, ..bounds }));
    assert!(!payload::fits(&row, &Bounds { text: 31, ..bounds }));
    assert_eq!(row, before);
    assert_eq!(row.context, Some(context));
    row.context = None;
    assert_eq!(row.heap(&bounds), Some(absent));
    assert!(payload::fits(
        &row,
        &Bounds { document_bytes: u64::try_from(size_of::<Task>()).unwrap() + absent, ..bounds }
    ));
    assert_eq!(row.context, None);
}
