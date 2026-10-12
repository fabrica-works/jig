//! Whole note owner facts and literal scopes (contract.md, 5.2; objects.md, 10).
use super::{offers, owner};
use jig_domain_client::{Limits, contract as client};
use jig_domain_core::documents;
use jig_domain_notes as notes;

/// # Errors
/// Refuses any supplied text, pattern or reference allocation above the receiving bounds.
pub fn scope(value: &documents::NoteLocation, limits: &Limits) -> Result<client::NoteScope, owner::Invalid> {
    Ok(match value {
        documents::NoteLocation::Deployment => client::NoteScope::Deployment,
        documents::NoteLocation::Project(project) => client::NoteScope::Project(*project),
        documents::NoteLocation::Goal { project, task } => client::NoteScope::Goal { project: *project, goal: *task },
        documents::NoteLocation::Resources { project, scope } => client::NoteScope::Resource {
            project: *project,
            connector: scope.connector,
            pattern: owner::pattern(&scope.segments, &scope.last, limits)?,
        },
    })
}

/// # Errors
/// Refuses any supplied text, pattern or reference allocation above the receiving bounds.
pub fn note(value: &documents::Note, limits: &Limits) -> Result<client::Note, owner::Invalid> {
    Ok(client::Note {
        number: value.number,
        revision: client::Revision { value: u64::from(value.revision) },
        scope: scope(&value.scope, limits)?,
        description: owner::text(&value.description, limits)?,
        body: owner::text(&value.body, limits)?,
        writer: match value.writer {
            notes::Author::Party { party } => client::NoteAuthor::Party { party },
            notes::Author::Task { task, attempt } => client::NoteAuthor::Task { task, attempt },
        },
        tasks: owner::list(&value.tasks, limits, |number| Ok(*number))?,
        changed: value.changed,
        acts: offers::offered(&value.acts, limits)?,
        cut: value.cut,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use jig_domain_core::acts;
    use jig_domain_people as people;
    use skein_lib::{List, Wall};

    fn document() -> documents::Note {
        let mut segments: List<Box<[u8]>> = List::with_capacity(4);
        segments.push(Box::from([255, b'/', 0])).expect("whole literal segment");
        let mut tasks = List::with_capacity(4);
        tasks.push(0).expect("known zero reference");
        tasks.push(u64::MAX).expect("full reference");
        let mut offers = List::with_capacity(2);
        offers
            .push(acts::Offered { act: acts::Act::EditNote, admission: acts::Admission::Allowed })
            .expect("actual edit offer");
        offers
            .push(acts::Offered {
                act: acts::Act::DeleteNote,
                admission: acts::Admission::Refused(people::Refusal::KeyConflict),
            })
            .expect("actual deletion judgment");
        documents::Note {
            acts: offers,
            number: u64::MAX,
            scope: documents::NoteLocation::Resources {
                project: u32::MAX,
                scope: documents::NoteScope {
                    connector: u16::MAX,
                    segments,
                    last: documents::PatternEnd::Open(Box::from([b'/', 255])),
                },
            },
            description: documents::Text { bytes: Box::new([]), cut: false },
            body: documents::Text { bytes: Box::from([255, 0]), cut: true },
            writer: notes::Author::Task { task: u64::MAX, attempt: u64::MAX },
            tasks,
            changed: None,
            revision: u32::MAX,
            cut: true,
        }
    }

    #[test]
    fn whole_note_keeps_stored_author_attempt_optional_clock_and_independent_text_reference_and_scope_facts() {
        let limits = Limits { rows: 4, ..crate::test::limits().jig };
        for (writer, expected) in [
            (notes::Author::Party { party: 0 }, client::NoteAuthor::Party { party: 0 }),
            (notes::Author::Party { party: u64::MAX }, client::NoteAuthor::Party { party: u64::MAX }),
            (
                notes::Author::Task { task: u64::MAX, attempt: 0 },
                client::NoteAuthor::Task { task: u64::MAX, attempt: 0 },
            ),
            (
                notes::Author::Task { task: u64::MAX, attempt: u64::MAX },
                client::NoteAuthor::Task { task: u64::MAX, attempt: u64::MAX },
            ),
        ] {
            for changed in [None, Some(Wall::EPOCH)] {
                let mut source = document();
                source.writer = writer;
                source.changed = changed;
                let before = source.clone();
                let value = note(&source, &limits).expect("whole note fits");
                assert_eq!(value.writer, expected);
                assert_eq!(value.changed, changed);
                assert_eq!(value.number, u64::MAX);
                assert_eq!(value.revision.value, u64::from(u32::MAX));
                assert_eq!(value.description.bytes.bytes(), b"");
                assert!(!value.description.cut);
                assert_eq!(value.body.bytes.bytes(), &[255, 0]);
                assert!(value.body.cut && value.cut);
                assert_eq!(value.tasks.capacity(), 4);
                assert_eq!(value.tasks.as_slice(), &[0, u64::MAX]);
                assert_eq!(value.acts.capacity(), 2);
                assert_eq!(
                    value.acts.as_slice(),
                    &[
                        client::Offered { act: client::ActKind::EditNote, admission: client::Admission::Allowed },
                        client::Offered {
                            act: client::ActKind::DeleteNote,
                            admission: client::Admission::Refused(client::Refusal::KeyConflict)
                        }
                    ]
                );
                let client::NoteScope::Resource { project, connector, pattern } = value.scope else {
                    panic!("supplied resource scope")
                };
                assert_eq!(project, u32::MAX);
                assert_eq!(connector, u16::MAX);
                assert_eq!(pattern.segments.capacity(), 4);
                assert_eq!(pattern.segments.get(0).expect("literal segment").as_ref(), &[255, b'/', 0]);
                assert_eq!(pattern.last, client::OwnerPatternEnd::Open(Box::from([b'/', 255])));
                assert_eq!(source, before);
            }
        }
    }

    #[test]
    fn note_receiver_preserves_whole_allocated_bounds_and_each_supplied_scope_without_a_page_fallback() {
        let source = document();
        let before = source.clone();
        let limits = Limits { rows: 4, ..crate::test::limits().jig };
        assert_eq!(note(&source, &Limits { rows: 3, ..limits }), Err(owner::Invalid::TooLarge));
        assert_eq!(note(&source, &Limits { text: 2, ..limits }), Err(owner::Invalid::TooLarge));
        assert_eq!(source, before);
        for (source, expected) in [
            (documents::NoteLocation::Deployment, client::NoteScope::Deployment),
            (documents::NoteLocation::Project(u32::MAX), client::NoteScope::Project(u32::MAX)),
            (
                documents::NoteLocation::Goal { project: 0, task: u64::MAX },
                client::NoteScope::Goal { project: 0, goal: u64::MAX },
            ),
        ] {
            assert_eq!(scope(&source, &limits), Ok(expected));
        }
    }
}
