//! Notes rows and their bounded resource patterns (protocol/store.md, 5.1).
use crate::notes_v1 as wire;
use alloc::boxed::Box;
use jig_domain_notes as domain;
use skein_lib::{List, Reader, Wall, Writer};

/// Translate the notes child's bounds, refusing a schema ceiling.
#[must_use]
pub fn limits(domain: &domain::Limits) -> Option<wire::Limits> {
    let ceilings = &wire::CEILINGS;
    if domain.pattern_bytes > ceilings.segment_bytes
        || domain.pattern_bytes > ceilings.pattern_segments
        || domain.description_bytes > ceilings.entry_description
        || domain.description_bytes > ceilings.line_description
        || domain.body_bytes > ceilings.entry_body
        || domain.references > ceilings.entry_references
    {
        return None;
    }
    Some(wire::Limits {
        segment_bytes: domain.pattern_bytes,
        pattern_segments: domain.pattern_bytes,
        entry_description: domain.description_bytes,
        line_description: domain.description_bytes,
        entry_body: domain.body_bytes,
        entry_references: domain.references,
    })
}

/// Encode one versioned notes row within the configured bounds.
pub fn encode(row: &domain::Record, limits: &wire::Limits) -> Result<Box<[u8]>, wire::Problem> {
    let value = match row {
        domain::Record::Entry(domain::Entry {
            name,
            scope,
            description,
            body,
            references,
            author,
            revision,
            changed,
        }) => {
            let author = match author {
                domain::Author::Party { party } => {
                    wire::Author::Party(wire::Party::new(limits, wire::PartyParts { party: *party })?)
                }
                domain::Author::Task { task, attempt } => {
                    wire::Author::Task(wire::Task::new(limits, wire::TaskParts { task: *task, attempt: *attempt })?)
                }
            };
            let mut translated_references = List::with_capacity(references.capacity());
            for task in references {
                translated_references
                    .push(wire::Reference::new(limits, wire::ReferenceParts { task: *task })?)
                    .expect("same number of references");
            }
            wire::Value::Entry(wire::Entry::new(
                limits,
                wire::EntryParts {
                    name: *name,
                    scope: encode_scope(scope, limits)?,
                    description: description.clone(),
                    body: body.clone(),
                    references: translated_references,
                    author,
                    revision: *revision,
                    changed: changed.as_nanos(),
                },
            )?)
        }
        domain::Record::Line(domain::Line { scope, name, description, revision }) => {
            wire::Value::Line(wire::Line::new(
                limits,
                wire::LineParts {
                    scope: encode_scope(scope, limits)?,
                    name: *name,
                    description: description.clone(),
                    revision: *revision,
                },
            )?)
        }
    };
    let row = wire::Row::new(limits, wire::RowParts { value })?;
    let mut writer = Writer::new(usize::try_from(row.measure()).expect("encoded row fits usize"));
    row.encode(&mut writer).expect("measured room");
    Ok(writer.finish())
}

/// Decode one notes row, refusing another version or trailing bytes.
pub fn decode(bytes: &[u8], limits: &wire::Limits) -> Result<domain::Record, wire::Problem> {
    let wire::RowParts { value } = wire::Row::decode(limits, &mut Reader::new(bytes))?.into_parts();
    Ok(match value {
        wire::Value::Entry(entry) => {
            let wire::EntryParts { name, scope, description, body, references, author, revision, changed } =
                entry.into_parts();
            let author = match author {
                wire::Author::Party(party) => {
                    let wire::PartyParts { party } = party.into_parts();
                    domain::Author::Party { party }
                }
                wire::Author::Task(task) => {
                    let wire::TaskParts { task, attempt } = task.into_parts();
                    domain::Author::Task { task, attempt }
                }
            };
            let mut translated_references = List::with_capacity(references.len());
            for reference in references.into_boxed() {
                let wire::ReferenceParts { task } = reference.into_parts();
                translated_references.push(task).expect("same number of references");
            }
            domain::Record::Entry(domain::Entry {
                name,
                scope: decode_scope(scope, limits)?,
                description,
                body,
                references: translated_references,
                author,
                revision,
                changed: Wall::from_nanos(changed),
            })
        }
        wire::Value::Line(line) => {
            let wire::LineParts { scope, name, description, revision } = line.into_parts();
            domain::Record::Line(domain::Line { scope: decode_scope(scope, limits)?, name, description, revision })
        }
    })
}

fn encode_scope(scope: &domain::Scope, limits: &wire::Limits) -> Result<wire::Scope, wire::Problem> {
    Ok(match scope {
        domain::Scope::Deployment => wire::Scope::Deployment,
        domain::Scope::Project { project } => {
            wire::Scope::Project(wire::Project::new(limits, wire::ProjectParts { project: *project })?)
        }
        domain::Scope::Goal { project, goal } => {
            wire::Scope::Goal(wire::Goal::new(limits, wire::GoalParts { project: *project, goal: *goal })?)
        }
        domain::Scope::Resources { project, connector, pattern } => {
            let domain::Pattern { segments, last } = pattern;
            let mut bytes = match last {
                domain::Last::Exact(bytes) | domain::Last::Open(bytes) => bytes.len(),
            };
            let mut members = List::with_capacity(limits.pattern_segments);
            for segment in segments {
                bytes = bytes.saturating_add(segment.len());
                let segment = wire::Segment::new(limits, wire::SegmentParts { bytes: segment.clone() })?;
                if members.push(segment).is_err() {
                    return Err(pattern_problem());
                }
            }
            if bytes > usize::try_from(limits.segment_bytes).expect("limit fits usize") {
                return Err(pattern_problem());
            }
            let last = match last {
                domain::Last::Exact(bytes) => {
                    wire::Last::Exact(wire::Segment::new(limits, wire::SegmentParts { bytes: bytes.clone() })?)
                }
                domain::Last::Open(bytes) => {
                    wire::Last::Open(wire::Segment::new(limits, wire::SegmentParts { bytes: bytes.clone() })?)
                }
            };
            let pattern = wire::Pattern::new(limits, wire::PatternParts { segments: members, last })?;
            wire::Scope::Resources(wire::Resources::new(
                limits,
                wire::ResourcesParts { project: *project, connector: *connector, pattern },
            )?)
        }
    })
}

fn decode_scope(scope: wire::Scope, limits: &wire::Limits) -> Result<domain::Scope, wire::Problem> {
    Ok(match scope {
        wire::Scope::Deployment => domain::Scope::Deployment,
        wire::Scope::Project(project) => {
            let wire::ProjectParts { project } = project.into_parts();
            domain::Scope::Project { project }
        }
        wire::Scope::Goal(goal) => {
            let wire::GoalParts { project, goal } = goal.into_parts();
            domain::Scope::Goal { project, goal }
        }
        wire::Scope::Resources(resources) => {
            let wire::ResourcesParts { project, connector, pattern } = resources.into_parts();
            let wire::PatternParts { segments, last } = pattern.into_parts();
            let last = match last {
                wire::Last::Exact(segment) => domain::Last::Exact(segment.into_parts().bytes),
                wire::Last::Open(segment) => domain::Last::Open(segment.into_parts().bytes),
            };
            let mut bytes = match &last {
                domain::Last::Exact(bytes) | domain::Last::Open(bytes) => bytes.len(),
            };
            let mut members = List::with_capacity(segments.len());
            for segment in segments.into_boxed() {
                let wire::SegmentParts { bytes: segment } = segment.into_parts();
                bytes = bytes.saturating_add(segment.len());
                members.push(segment).expect("same number of segments");
            }
            if bytes > usize::try_from(limits.segment_bytes).expect("limit fits usize") {
                return Err(pattern_problem());
            }
            domain::Scope::Resources {
                project,
                connector,
                pattern: domain::Pattern { segments: members.into_boxed(), last },
            }
        }
    })
}

fn pattern_problem() -> wire::Problem {
    wire::Problem { path: wire::Path::PatternSegments, reason: skein_codec::Reason::Bound }
}
