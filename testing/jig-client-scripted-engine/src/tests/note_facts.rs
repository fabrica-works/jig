use super::*;
use crate::payload::{self, Payload};
use core::mem::size_of;
use skein_lib::Wall;
#[test]
fn whole_note_admission_keeps_writer_scalars_and_time_presence_with_all_nested_storage() {
    for writer in [NoteAuthor::Party { party: u64::MAX }, NoteAuthor::Task { task: u64::MAX, attempt: u64::MAX }] {
        for changed in [None, Some(Wall::EPOCH)] {
            let mut tasks = List::with_capacity(2);
            tasks.push(u64::MAX).unwrap();
            let note = Note {
                number: u64::MAX,
                revision: Revision { value: u64::MAX },
                scope: NoteScope::Task { project: u32::MAX, task: u64::MAX },
                description: DocumentText::complete(text(b"name")),
                body: DocumentText { bytes: text(b"body"), cut: true },
                writer,
                tasks,
                changed,
                acts: List::with_capacity(1),
                cut: true,
            };
            let mut bounds = Bounds { rows: 2, text: 4, document_bytes: u64::MAX, ..Bounds::default() };
            assert_eq!(note.writer.heap(&bounds), Some(0));
            let whole = u64::try_from(size_of::<Note>()).unwrap() + note.heap(&bounds).unwrap();
            assert_eq!(
                whole,
                u64::try_from(size_of::<Note>() + 2 * size_of::<u64>() + size_of::<Offered>()).unwrap() + 8
            );
            bounds.document_bytes = whole;
            assert!(payload::fits(&note, &bounds));
            bounds.document_bytes = whole - 1;
            assert!(!payload::fits(&note, &bounds));
            bounds.document_bytes = whole + 1;
            bounds.rows = 1;
            assert!(!payload::fits(&note, &bounds));
            bounds.rows = 2;
            bounds.text = 3;
            assert!(!payload::fits(&note, &bounds));
        }
    }
}
