//! Read selectors retain their owner query and opaque continuation whole
//! (client/domain/contract.md, 6). The engine owns cursor/query admission.
//! Note selectors preserve explicit project scope and whole literal patterns.
use jig_domain_client::contract as client;
use jig_domain_core::reads;
use jig_domain_notes as notes;
use jig_domain_people as people;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Invalid {
    Cursor,
    InboxGroup,
}

/// # Errors
/// Refuses a cursor for another query or an inbox kind outside its supplied parent group.
pub fn read(value: &client::Read, cursor_bytes: u32) -> Result<reads::Read, Invalid> {
    Ok(match value {
        client::Read::Frame => reads::Read::Frame,
        client::Read::Task { task } => reads::Read::Task { task: *task },
        client::Read::Proposal { proposal } => {
            reads::Read::Proposal { proposer: proposer(proposal.proposer), proposal: proposal.number }
        }
        client::Read::Escalation { task } => reads::Read::Escalation { task: *task },
        client::Read::Question { task, message } => reads::Read::Question { task: *task, message: *message },
        client::Read::PersonTask { task } => reads::Read::PersonTask { task: *task },
        client::Read::Result { task } => reads::Read::Result { task: *task },
        client::Read::Ended { task, after } => {
            reads::Read::EndedDelegates { task: *task, from: cursor(after.as_ref(), cursor_bytes)? }
        }
        client::Read::History { task, subtree, after } => {
            reads::Read::History { task: *task, subtree: *subtree, from: cursor(after.as_ref(), cursor_bytes)? }
        }
        client::Read::Activity { task, before } => {
            reads::Read::Activity { task: *task, before: cursor(before.as_ref(), cursor_bytes)? }
        }
        client::Read::Inbox { group, project, kind, goal, after } => reads::Read::Inbox {
            group: inbox(*group, *kind)?,
            filter: reads::Filter { project: *project, goal: *goal },
            from: cursor(after.as_ref(), cursor_bytes)?,
        },
        client::Read::Conversations { after } => {
            reads::Read::Conversations { from: cursor(after.as_ref(), cursor_bytes)? }
        }
        client::Read::Goals { project, after } => {
            reads::Read::Goals { project: *project, from: cursor(after.as_ref(), cursor_bytes)? }
        }
        client::Read::Spend { project, after } => {
            reads::Read::Spend { project: *project, from: cursor(after.as_ref(), cursor_bytes)? }
        }
        client::Read::Project { project } => reads::Read::Project { project: *project },
        client::Read::Notes { scope, description, after } => reads::Read::ScopedNotes {
            scope: note_scope(scope),
            description: description.as_ref().map(|value| Box::from(value.bytes())),
            from: cursor(after.as_ref(), cursor_bytes)?,
        },
        client::Read::Deployment => reads::Read::Deployment,
    })
}

fn cursor(value: Option<&client::Cursor>, limit: u32) -> Result<Option<reads::Cursor>, Invalid> {
    value.map(|value| super::cursor::decode(value, limit).ok_or(Invalid::Cursor)).transpose()
}

#[must_use]
pub fn proposer(value: client::Proposer) -> people::Proposer {
    match value {
        client::Proposer::Task(number) => people::Proposer::Task(number),
        client::Proposer::Party(number) => people::Proposer::Party(number),
    }
}

fn inbox(parent: client::InboxReadGroup, kind: Option<client::InboxGroup>) -> Result<reads::Group, Invalid> {
    let (expected, group) = match kind {
        None => {
            return Ok(match parent {
                client::InboxReadGroup::Needs => reads::Group::Needs,
                client::InboxReadGroup::Updates => reads::Group::Updates,
            });
        }
        Some(client::InboxGroup::Proposals) => (client::InboxReadGroup::Needs, reads::Group::Proposals),
        Some(client::InboxGroup::Escalations) => (client::InboxReadGroup::Needs, reads::Group::Escalations),
        Some(client::InboxGroup::Questions) => (client::InboxReadGroup::Needs, reads::Group::Questions),
        Some(client::InboxGroup::PersonTasks) => (client::InboxReadGroup::Needs, reads::Group::PersonTasks),
        Some(client::InboxGroup::Results) => (client::InboxReadGroup::Updates, reads::Group::Results),
        Some(client::InboxGroup::Replies) => (client::InboxReadGroup::Updates, reads::Group::Replies),
    };
    if parent != expected {
        return Err(Invalid::InboxGroup);
    }
    Ok(group)
}

pub(crate) fn note_scope(value: &client::NoteScope) -> notes::Scope {
    match value {
        client::NoteScope::Deployment => notes::Scope::Deployment,
        client::NoteScope::Project(project) => notes::Scope::Project { project: *project },
        client::NoteScope::Goal { project, goal } => notes::Scope::Goal { project: *project, goal: *goal },
        client::NoteScope::Resource { project, connector, pattern } => notes::Scope::Resources {
            project: *project,
            connector: *connector,
            pattern: notes::Pattern {
                segments: pattern.segments.iter().cloned().collect(),
                last: match &pattern.last {
                    client::OwnerPatternEnd::Exact(bytes) => notes::Last::Exact(bytes.clone()),
                    client::OwnerPatternEnd::Open(bytes) => notes::Last::Open(bytes.clone()),
                },
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inbox_read_translation_keeps_every_group_filter_and_rejects_a_different_parent() {
        for (parent, kind, expected) in [
            (client::InboxReadGroup::Needs, None, reads::Group::Needs),
            (client::InboxReadGroup::Updates, None, reads::Group::Updates),
            (client::InboxReadGroup::Needs, Some(client::InboxGroup::Proposals), reads::Group::Proposals),
            (client::InboxReadGroup::Needs, Some(client::InboxGroup::Escalations), reads::Group::Escalations),
            (client::InboxReadGroup::Needs, Some(client::InboxGroup::Questions), reads::Group::Questions),
            (client::InboxReadGroup::Needs, Some(client::InboxGroup::PersonTasks), reads::Group::PersonTasks),
            (client::InboxReadGroup::Updates, Some(client::InboxGroup::Results), reads::Group::Results),
            (client::InboxReadGroup::Updates, Some(client::InboxGroup::Replies), reads::Group::Replies),
        ] {
            let query =
                client::Read::Inbox { group: parent, kind, project: Some(u32::MAX), goal: Some(u64::MAX), after: None };
            assert_eq!(
                read(&query, 1024).expect("the complete supplied fixture is admitted"),
                reads::Read::Inbox {
                    group: expected,
                    filter: reads::Filter { project: Some(u32::MAX), goal: Some(u64::MAX) },
                    from: None,
                }
            );
            if let Some(kind) = kind {
                let other = match parent {
                    client::InboxReadGroup::Needs => client::InboxReadGroup::Updates,
                    client::InboxReadGroup::Updates => client::InboxReadGroup::Needs,
                };
                assert_eq!(
                    read(
                        &client::Read::Inbox { group: other, kind: Some(kind), project: None, goal: None, after: None },
                        1024
                    ),
                    Err(Invalid::InboxGroup)
                );
            }
        }
    }

    #[test]
    fn read_translation_keeps_opaque_cursor_identity_full_width_makers_and_supplied_empty_note_search() {
        let retained = reads::Cursor::Store { scope: reads::Scope::Goals { project: u32::MAX }, after: None };
        let encoded = super::super::cursor::encode(&retained, 1024).expect("the complete supplied fixture is admitted");
        assert_eq!(
            read(&client::Read::Goals { project: u32::MAX, after: Some(encoded.clone()) }, 1024)
                .expect("the complete supplied fixture is admitted"),
            reads::Read::Goals { project: u32::MAX, from: Some(retained) }
        );
        let malformed = client::Cursor {
            value: client::Text::new(Box::from([1u8, 255]), 1024).expect("the complete supplied fixture is admitted"),
        };
        assert_eq!(read(&client::Read::Conversations { after: Some(malformed) }, 1024), Err(Invalid::Cursor));
        assert_eq!(
            read(&client::Read::Conversations { after: None }, 1024)
                .expect("the complete supplied fixture is admitted"),
            reads::Read::Conversations { from: None }
        );
        for proposer in [client::Proposer::Task(u64::MAX), client::Proposer::Party(u64::MAX)] {
            assert_eq!(
                read(&client::Read::Proposal { proposal: client::ProposalKey { proposer, number: u64::MAX } }, 1024)
                    .expect("the complete supplied fixture is admitted"),
                reads::Read::Proposal { proposer: super::proposer(proposer), proposal: u64::MAX }
            );
        }
        for description in
            [None, Some(client::Text::new(Box::new([]), 1024).expect("the complete supplied fixture is admitted"))]
        {
            assert_eq!(
                read(
                    &client::Read::Notes {
                        scope: client::NoteScope::Project(u32::MAX),
                        description: description.clone(),
                        after: None
                    },
                    1024
                )
                .expect("the complete supplied fixture is admitted"),
                reads::Read::ScopedNotes {
                    scope: notes::Scope::Project { project: u32::MAX },
                    description: description.map(|value| Box::from(value.bytes())),
                    from: None
                }
            );
        }
    }
    #[test]
    fn scoped_notes_read_keeps_literal_segments_terminals_and_named_query_cursor_whole() {
        use skein_lib::List;
        for open in [false, true] {
            let mut segments: List<Box<[u8]>> = List::with_capacity(3);
            segments.push(Box::from([255, b'/', 0])).expect("supplied literal segment");
            segments.push(Box::new([])).expect("supplied empty segment");
            let terminal = Box::from([b'/', 255]);
            let pattern = client::OwnerPattern {
                segments,
                last: if open {
                    client::OwnerPatternEnd::Open(terminal)
                } else {
                    client::OwnerPatternEnd::Exact(terminal)
                },
            };
            let scope =
                client::NoteScope::Resource { project: u32::MAX, connector: u16::MAX, pattern: pattern.clone() };
            let supplied_scope = note_scope(&scope);
            let retained = reads::Cursor::Store {
                scope: reads::Scope::ScopedNotes {
                    scope: supplied_scope.clone(),
                    description: Some(Box::from([255, 0])),
                },
                after: Some(reads::Position::Owner(jig_domain_core::Key::Notes(notes::Key::Line {
                    scope: supplied_scope,
                    name: u64::MAX,
                }))),
            };
            let encoded = super::super::cursor::encode(&retained, 1024).expect("whole cursor");
            let query = client::Read::Notes {
                scope,
                description: Some(client::Text::new(Box::from([255, 0]), 1024).expect("literal description")),
                after: Some(encoded),
            };
            let before = query.clone();
            assert_eq!(
                read(&query, 1024),
                Ok(reads::Read::ScopedNotes {
                    scope: notes::Scope::Resources {
                        project: u32::MAX,
                        connector: u16::MAX,
                        pattern: notes::Pattern {
                            segments: vec![Box::<[u8]>::from([255, b'/', 0]), Box::<[u8]>::default()]
                                .into_boxed_slice(),
                            last: if open {
                                notes::Last::Open(Box::from([b'/', 255]))
                            } else {
                                notes::Last::Exact(Box::from([b'/', 255]))
                            },
                        },
                    },
                    description: Some(Box::from([255, 0])),
                    from: Some(retained),
                })
            );
            assert_eq!(query, before);
            assert_eq!(pattern.segments.capacity(), 3);
        }
        let query = client::Read::Notes {
            scope: client::NoteScope::Goal { project: u32::MAX, goal: u64::MAX },
            description: None,
            after: None,
        };
        assert_eq!(
            read(&query, 1024),
            Ok(reads::Read::ScopedNotes {
                scope: notes::Scope::Goal { project: u32::MAX, goal: u64::MAX },
                description: None,
                from: None
            })
        );
    }
}
