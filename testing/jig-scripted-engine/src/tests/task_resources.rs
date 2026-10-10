use super::*;
use crate::payload::{self, Payload};
use core::mem::size_of;

#[test]
fn owner_task_resources_price_reserved_variant_rows_boxes_and_raw_segments_at_exact_parent_bounds() {
    let bounds = Bounds { rows: 2, text: 32, document_bytes: u64::MAX, ..Bounds::default() };
    let mut row = task();
    row.resources = List::with_capacity(0);
    let absent = row.heap(&bounds).unwrap();
    row.resources = List::with_capacity(2);
    let reserved = row.heap(&bounds).unwrap();
    assert_eq!(reserved - absent, 2 * u64::try_from(size_of::<TaskResource>()).unwrap());
    let mut segments = List::with_capacity(2);
    segments.push(Box::<[u8]>::from([255_u8; 32])).unwrap();
    segments.push(Box::<[u8]>::from([254_u8; 32])).unwrap();
    let owner = OwnerResourceLine {
        part: OwnerPart { connector: u16::MAX, kind: OwnerPartKind::Resource, key: OwnerPartKey::Path(segments) },
        kind: u16::MAX,
        taken: false,
        holder: Some(u64::MAX),
        writer: Some(OwnerResourceWriter::Run { task: u64::MAX, attempt: u64::MAX }),
        writer_lost: true,
    };
    row.resources.push(TaskResource::Owner(Box::new(owner.clone()))).unwrap();
    let expected = u64::try_from(size_of::<OwnerResourceLine>()).unwrap()
        + 2 * u64::try_from(size_of::<Box<[u8]>>()).unwrap()
        + 64;
    assert_eq!(row.heap(&bounds).unwrap() - reserved, expected);
    let exact = u64::try_from(size_of::<Task>()).unwrap() + reserved + expected;
    let before = row.clone();
    assert!(payload::fits(&row, &Bounds { document_bytes: exact, ..bounds }));
    assert!(!payload::fits(&row, &Bounds { document_bytes: exact - 1, ..bounds }));
    assert!(!payload::fits(&row, &Bounds { rows: 1, ..bounds }));
    assert!(!payload::fits(&row, &Bounds { text: 31, ..bounds }));
    assert_eq!(row, before);
    assert_eq!(row.resources.as_slice(), &[TaskResource::Owner(Box::new(owner))]);
}
