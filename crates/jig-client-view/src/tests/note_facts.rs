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
            cards::card(&mut builder, &domain, id, &cards::Place::Elsewhere, words, &mut bindings);
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

fn settled_note_answer(act: ActKind, cut: bool) {
    let note = Note {
        number: 77,
        revision: Revision { value: 123 },
        scope: NoteScope::Project(4),
        description: DocumentText { bytes: text(b"stored note title"), cut },
        body: DocumentText { bytes: text(b"**stored note body**"), cut },
        writer: NoteAuthor::Task { task: 91, attempt: 12 },
        tasks: list([91, u64::MAX]),
        changed: Some(Wall::EPOCH),
        acts: list([
            Offered { act: ActKind::EditNote, admission: Admission::Allowed },
            Offered { act: ActKind::DeleteNote, admission: Admission::Allowed },
        ]),
        cut,
    };
    let expected_note = note.clone();
    let (mut domain, id) = note_loaded(note);
    let words = words();
    let mut bindings = JigBindings::new(256, CLIENT.text);
    let tree = render(&domain, id, words, &mut bindings);
    assert!(!contains(&tree, b"Note edited"));
    assert!(contains(&tree, b"Edit note"));
    assert!(contains(&tree, b"Delete note"));
    let label = if act == ActKind::EditNote { b"Edit note".as_slice() } else { b"Delete note".as_slice() };
    let action = press(&tree, label, &mut bindings);
    drive(&mut domain, Event::Act { act: action });
    let outgoing = drive(&mut domain, Event::Act { act: Action::Confirm });
    let request = outgoing
        .iter()
        .find_map(|request| match request {
            Request::Send { request, .. } => Some(*request),
            _ => None,
        })
        .unwrap();
    assert!(!contains(&render(&domain, id, words, &mut bindings), b"Note edited"));
    let answer = Answer::Done(Outcome::Owner(OwnerOutcome::NoteEdited { name: 77 }));
    drive(&mut domain, Event::Answered { request, answer: answer.clone() });
    let facts = domain.objects().get(id).unwrap();
    assert_eq!(facts.answer.as_ref(), Some(&answer));
    match &facts.body {
        Some(crate::objects::Body::Note { document, .. }) => assert_eq!(document.as_ref(), &expected_note),
        other => panic!("the answer must preserve the supplied note: {other:?}"),
    }
    let mut envelope = crate::limits::of_words(&CLIENT, words).unwrap();
    envelope.nodes = crate::card_nodes(&CLIENT).unwrap();
    let mut builder = Builder::new(&envelope);
    bindings.begin();
    assert!(cards::card(&mut builder, &domain, id, &cards::Place::Elsewhere, words, &mut bindings));
    let tree = builder.finish();
    assert!(
        tree.nodes().iter().any(|node| node.role == Role::Region && node.name.as_deref() == Some(b"stored note title"))
    );
    assert!(tree.len() <= envelope.nodes);
    assert!(tree.nodes().iter().any(|node| node.role == Role::Status && node.name.as_deref() == Some(b"Note edited")));
    assert!(contains(&tree, b"Note: 77"));
    assert!(contains(&tree, b"stored note body"));
    assert!(contains(&tree, b"Written by task 91, attempt 12"));
    assert_eq!(contains(&tree, b"Only part of this text was supplied"), cut);
    assert_eq!(contains(&tree, b"This note has more references"), cut);
    assert_eq!(tree.nodes().iter().filter(|node| node.role == Role::Time).count(), 1);
    for name in [b"Task 91".as_slice(), b"Task 18446744073709551615".as_slice()] {
        assert!(tree.nodes().iter().any(|node| node.role == Role::Link && node.name.as_deref() == Some(name)));
    }
    assert_eq!(contains(&tree, b"Edit note"), act == ActKind::EditNote);
    assert_eq!(contains(&tree, b"Delete note"), act == ActKind::EditNote);
}

#[test]
fn a_note_card_shows_its_durable_edit_answer_with_the_stored_document_and_offers() {
    settled_note_answer(ActKind::EditNote, false);
}

#[test]
fn a_note_card_shows_its_durable_deletion_answer_with_the_stored_loss_facts() {
    settled_note_answer(ActKind::DeleteNote, true);
}
