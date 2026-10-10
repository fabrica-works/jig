use super::*;
#[test]
fn note_cards_name_the_whole_stored_attempt_and_only_render_supplied_times() {
    for writer in [NoteAuthor::Party { party: u64::MAX }, NoteAuthor::Task { task: u64::MAX, attempt: u64::MAX }] {
        for changed in [None, Some(Wall::EPOCH)] {
            let note = Note {
                number: 77,
                revision: Revision { value: u64::MAX },
                scope: NoteScope::Goal { project: u32::MAX, goal: u64::MAX },
                description: DocumentText::complete(text(b"note title")),
                body: DocumentText { bytes: text(b"note body"), cut: true },
                writer,
                tasks: list([91]),
                changed,
                acts: offered(ActKind::DeleteNote),
                cut: true,
            };
            let (domain, id) = note_loaded(note);
            let words = words();
            let mut bindings = JigBindings::new(256, CLIENT.text);
            let mut builder = Builder::new(&limits());
            cards::card(&mut builder, &domain, id, &cards::Place::Elsewhere, &words, &mut bindings);
            let tree = builder.finish();
            assert!(contains(&tree, b"note body"));
            assert!(contains(&tree, b"Only part of this text was supplied"));
            assert!(contains(&tree, b"This note has more references"));
            assert_eq!(contains(&tree, b"Note changed time not supplied"), changed.is_none());
            assert_eq!(
                tree.nodes().iter().filter(|node| node.role == Role::Time).count(),
                usize::from(changed.is_some())
            );
            match writer {
                NoteAuthor::Party { .. } => {
                    assert!(contains(&tree, b"Written by party 18446744073709551615"));
                    assert!(!contains(&tree, b"Written by task"));
                }
                NoteAuthor::Task { .. } => {
                    assert!(contains(&tree, b"Written by task 18446744073709551615, attempt 18446744073709551615"));
                    assert!(!contains(&tree, b"Written by party"));
                }
            }
        }
    }
}
