//! Authenticated reads and their one durable answer (protocol/client.md, 6).
use super::{cursor, objects, pages, w};
use alloc::boxed::Box;
use jig_domain_core::reads;
use jig_domain_notes as notes;
use jig_domain_people as people;
use skein_lib::{Reader, Writer};
pub(crate) fn read(value: w::Read) -> Option<reads::Read> {
    Some(match value {
        w::Read::Frame => reads::Read::Frame,
        w::Read::Task(value) => reads::Read::Task { task: value.task() },
        w::Read::Proposal(value) => reads::Read::Proposal {
            proposer: match value.proposal().proposer() {
                w::Proposer::Task(value) => people::Proposer::Task(value.value()),
                w::Proposer::Party(value) => people::Proposer::Party(value.value()),
            },
            proposal: value.proposal().number(),
        },
        w::Read::Escalation(value) => reads::Read::Escalation { task: value.task() },
        w::Read::Question(value) => reads::Read::Question { task: value.task(), message: value.message() },
        w::Read::PersonTask(value) => reads::Read::PersonTask { task: value.task() },
        w::Read::Result(value) => reads::Read::Result { task: value.task() },
        w::Read::Ended(value) => {
            let parts = value.into_parts();
            let from = match parts.after {
                Some(value) => Some(cursor::decode(value.value())?),
                None => None,
            };
            reads::Read::EndedDelegates { task: parts.task, from }
        }
        w::Read::History(value) => {
            let parts = value.into_parts();
            let from = match parts.after {
                Some(value) => Some(cursor::decode(value.value())?),
                None => None,
            };
            reads::Read::History { task: parts.task, subtree: parts.subtree, from }
        }
        w::Read::Activity(value) => {
            let parts = value.into_parts();
            let before = match parts.before {
                Some(value) => Some(cursor::decode(value.value())?),
                None => None,
            };
            reads::Read::Activity { task: parts.task, before }
        }
        w::Read::Inbox(value) => {
            let parts = value.into_parts();
            let group = group(parts.group, parts.kind)?;
            let from = match parts.after {
                Some(value) => Some(cursor::decode(value.value())?),
                None => None,
            };
            reads::Read::Inbox { group, filter: reads::Filter { project: parts.project, goal: parts.goal }, from }
        }
        w::Read::Conversations(value) => {
            let parts = value.into_parts();
            let from = match parts.after {
                Some(value) => Some(cursor::decode(value.value())?),
                None => None,
            };
            reads::Read::Conversations { from }
        }
        w::Read::Goals(value) => {
            let parts = value.into_parts();
            let from = match parts.after {
                Some(value) => Some(cursor::decode(value.value())?),
                None => None,
            };
            reads::Read::Goals { project: parts.project, from }
        }
        w::Read::Spend(value) => {
            let parts = value.into_parts();
            let from = match parts.after {
                Some(value) => Some(cursor::decode(value.value())?),
                None => None,
            };
            reads::Read::Spend { project: parts.project, from }
        }
        w::Read::Project(value) => reads::Read::Project { project: value.project() },
        w::Read::Notes(value) => {
            let parts = value.into_parts();
            let from = match parts.after {
                Some(value) => Some(cursor::decode(value.value())?),
                None => None,
            };
            reads::Read::ScopedNotes { scope: note_scope(parts.scope), description: parts.description, from }
        }
        w::Read::Deployment => reads::Read::Deployment,
    })
}
fn group(parent: w::InboxReadGroup, kind: Option<w::InboxGroup>) -> Option<reads::Group> {
    let (expected, group) = match kind {
        None => {
            return Some(match parent {
                w::InboxReadGroup::Needs => reads::Group::Needs,
                w::InboxReadGroup::Updates => reads::Group::Updates,
            });
        }
        Some(w::InboxGroup::Proposals) => (w::InboxReadGroup::Needs, reads::Group::Proposals),
        Some(w::InboxGroup::Escalations) => (w::InboxReadGroup::Needs, reads::Group::Escalations),
        Some(w::InboxGroup::Questions) => (w::InboxReadGroup::Needs, reads::Group::Questions),
        Some(w::InboxGroup::PersonTasks) => (w::InboxReadGroup::Needs, reads::Group::PersonTasks),
        Some(w::InboxGroup::Results) => (w::InboxReadGroup::Updates, reads::Group::Results),
        Some(w::InboxGroup::Replies) => (w::InboxReadGroup::Updates, reads::Group::Replies),
    };
    if parent != expected {
        return None;
    }
    Some(group)
}
fn note_scope(value: w::NoteScope) -> notes::Scope {
    match value {
        w::NoteScope::Deployment => notes::Scope::Deployment,
        w::NoteScope::Project(value) => notes::Scope::Project { project: value.value() },
        w::NoteScope::Goal(value) => notes::Scope::Goal { project: value.project(), goal: value.goal() },
        w::NoteScope::Resource(value) => {
            let parts = value.into_parts();
            let pattern = parts.pattern.into_parts();
            let last = match pattern.last {
                w::OwnerPatternEnd::Exact(value) => notes::Last::Exact(value.into_parts().value),
                w::OwnerPatternEnd::Open(value) => notes::Last::Open(value.into_parts().value),
            };
            notes::Scope::Resources {
                project: parts.project,
                connector: parts.connector,
                pattern: notes::Pattern { segments: pattern.segments.into_boxed(), last },
            }
        }
    }
}
pub(crate) fn none(why: reads::Why, limits: &w::Limits) -> w::ReadAnswer {
    let value = match why {
        reads::Why::Unknown => w::ReadNone::Unknown,
        reads::Why::Busy => w::ReadNone::Busy,
        reads::Why::Gone => w::ReadNone::Gone,
        reads::Why::SignedOut => w::ReadNone::SignedOut,
    };
    w::ReadAnswer::None(w::ReadAnswerNone::new(limits, w::ReadAnswerNoneParts { value }).expect("no bounded field"))
}
pub(crate) fn answer(
    read: &reads::Read,
    answer: &reads::Answer,
    limits: &w::Limits,
) -> Result<w::ReadAnswer, w::Problem> {
    let value = match answer {
        reads::Answer::None(why) => return Ok(none(*why, limits)),
        reads::Answer::Page(page) => pages::page(read, page, limits)?,
        reads::Answer::Document(_) | reads::Answer::Historical(_) => objects::document(answer, limits)?,
    };
    let mut out = Writer::new(usize::try_from(value.measure()).expect("wire measure fits"));
    value.encode(&mut out).expect("measured value");
    owned(w::Owner::Jig, jig_protocol_client_wire::VERSION, out.finish(), limits)
}
pub(crate) fn owned(
    owner: w::Owner,
    version: u32,
    record: Box<[u8]>,
    limits: &w::Limits,
) -> Result<w::ReadAnswer, w::Problem> {
    Ok(w::ReadAnswer::Value(w::Owned::new(limits, w::OwnedParts { owner, version, record })?))
}
pub(crate) fn decode(bytes: &[u8], limits: &w::Limits) -> Option<reads::Read> {
    let mut reader = Reader::new(bytes);
    let value = w::Read::decode(limits, &mut reader).ok()?;
    if !reader.is_empty() {
        return None;
    }
    read(value)
}
