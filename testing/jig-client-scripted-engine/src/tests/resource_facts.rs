use super::*;
use crate::payload::{self, Payload};
use core::mem::size_of;
#[test]
fn whole_segmented_resource_admission_prices_reserved_slots_terminal_bytes_and_explicit_project() {
    let mut path = List::with_capacity(3);
    path.push(Box::<[u8]>::from(*b"a/b\0")).unwrap();
    path.push(Box::<[u8]>::from([255, 0, 128])).unwrap();
    let resource = Part::Resource { connector: u16::MAX, path };
    let mut bounds = Bounds { text: 4, rows: 3, document_bytes: u64::MAX, ..Bounds::default() };
    let whole = u64::try_from(size_of::<Part>()).unwrap() + resource.heap(&bounds).unwrap();
    assert_eq!(whole, u64::try_from(size_of::<Part>() + 3 * size_of::<Box<[u8]>>()).unwrap() + 7);
    bounds.document_bytes = whole;
    assert!(payload::fits(&resource, &bounds));
    bounds.document_bytes = whole - 1;
    assert!(!payload::fits(&resource, &bounds));
    bounds.document_bytes = whole + 1;
    bounds.rows = 2;
    assert!(!payload::fits(&resource, &bounds));
    bounds.rows = 3;
    bounds.text = 3;
    assert!(!payload::fits(&resource, &bounds));
    let mut segments = List::with_capacity(3);
    segments.push(Box::<[u8]>::from(*b"a/b\0")).unwrap();
    let scope = NoteScope::Resource {
        project: u32::MAX,
        connector: u16::MAX,
        pattern: OwnerPattern { segments, last: OwnerPatternEnd::Open(Box::from([255, 0, 128])) },
    };
    bounds.text = 4;
    bounds.document_bytes = u64::MAX;
    let scoped = u64::try_from(size_of::<NoteScope>()).unwrap() + scope.heap(&bounds).unwrap();
    assert_eq!(scoped, u64::try_from(size_of::<NoteScope>() + 3 * size_of::<Box<[u8]>>()).unwrap() + 7);
    bounds.document_bytes = scoped;
    assert!(payload::fits(&scope, &bounds));
    bounds.document_bytes = scoped - 1;
    assert!(!payload::fits(&scope, &bounds));
    let goal = NoteScope::Goal { project: u32::MAX, goal: u64::MAX };
    bounds.document_bytes = u64::try_from(size_of::<NoteScope>()).unwrap();
    assert!(payload::fits(&goal, &bounds));
    assert_eq!(goal, NoteScope::Goal { project: u32::MAX, goal: u64::MAX });
}
