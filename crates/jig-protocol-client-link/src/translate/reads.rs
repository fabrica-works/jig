//! One read per exchange with opaque cursor bytes (protocol/client.md, 6).
use super::asks::{wire_note_scope, wire_proposal_key};
use crate::Readable;
use jig_domain_client::contract as c;
use jig_protocol_client_wire::VERSION;
use jig_protocol_client_wire::v3 as w;
use skein_lib::{Writer, bytes};

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub fn read(read: &c::Read, limits: &w::Limits) -> Result<w::Read, w::Problem> {
    Ok(match read {
        c::Read::Frame => w::Read::Frame,
        c::Read::Task { task } => w::Read::Task(w::ReadTask::new(limits, w::ReadTaskParts { task: *task })?),
        c::Read::Proposal { proposal } => w::Read::Proposal(w::ReadProposal::new(
            limits,
            w::ReadProposalParts { proposal: wire_proposal_key(proposal, limits)? },
        )?),
        c::Read::Escalation { task } => {
            w::Read::Escalation(w::ReadEscalation::new(limits, w::ReadEscalationParts { task: *task })?)
        }
        c::Read::Question { task, message } => {
            w::Read::Question(w::ReadQuestion::new(limits, w::ReadQuestionParts { task: *task, message: *message })?)
        }
        c::Read::PersonTask { task } => {
            w::Read::PersonTask(w::ReadPersonTask::new(limits, w::ReadPersonTaskParts { task: *task })?)
        }
        c::Read::Result { task } => w::Read::Result(w::ReadResult::new(limits, w::ReadResultParts { task: *task })?),
        c::Read::Ended { task, after } => w::Read::Ended(w::ReadEnded::new(
            limits,
            w::ReadEndedParts { task: *task, after: cursor(after.as_ref(), limits)? },
        )?),
        c::Read::History { task, subtree, after } => w::Read::History(w::ReadHistory::new(
            limits,
            w::ReadHistoryParts { task: *task, subtree: *subtree, after: cursor(after.as_ref(), limits)? },
        )?),
        c::Read::Activity { task, before } => w::Read::Activity(w::ReadActivity::new(
            limits,
            w::ReadActivityParts { task: *task, before: cursor(before.as_ref(), limits)? },
        )?),
        c::Read::Inbox { group, project, kind, goal, after } => w::Read::Inbox(w::ReadInbox::new(
            limits,
            w::ReadInboxParts {
                group: inbox_group(*group),
                project: *project,
                kind: inbox_kind(*kind),
                goal: *goal,
                after: cursor(after.as_ref(), limits)?,
            },
        )?),
        c::Read::Conversations { after } => w::Read::Conversations(w::ReadConversations::new(
            limits,
            w::ReadConversationsParts { after: cursor(after.as_ref(), limits)? },
        )?),
        c::Read::Goals { project, after } => w::Read::Goals(w::ReadGoals::new(
            limits,
            w::ReadGoalsParts { project: *project, after: cursor(after.as_ref(), limits)? },
        )?),
        c::Read::Spend { project, after } => w::Read::Spend(w::ReadSpend::new(
            limits,
            w::ReadSpendParts { project: *project, after: cursor(after.as_ref(), limits)? },
        )?),
        c::Read::Project { project } => {
            w::Read::Project(w::ReadProject::new(limits, w::ReadProjectParts { project: *project })?)
        }
        c::Read::Notes { scope, description, after } => {
            let description = match description {
                Some(value) => Some(bytes::copy_of(value.bytes())),
                None => None,
            };
            w::Read::Notes(w::ReadNotes::new(
                limits,
                w::ReadNotesParts {
                    scope: wire_note_scope(scope, limits)?,
                    description,
                    after: cursor(after.as_ref(), limits)?,
                },
            )?)
        }
        c::Read::Deployment => w::Read::Deployment,
    })
}
fn cursor(value: Option<&c::Cursor>, limits: &w::Limits) -> Result<Option<w::Cursor>, w::Problem> {
    match value {
        Some(value) => Ok(Some(w::Cursor::new(limits, w::CursorParts { value: bytes::copy_of(value.value.bytes()) })?)),
        None => Ok(None),
    }
}
fn inbox_group(value: c::InboxReadGroup) -> w::InboxReadGroup {
    match value {
        c::InboxReadGroup::Needs => w::InboxReadGroup::Needs,
        c::InboxReadGroup::Updates => w::InboxReadGroup::Updates,
    }
}
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
fn inbox_kind(value: Option<c::InboxGroup>) -> Option<w::InboxGroup> {
    match value {
        Some(value) => Some(match value {
            c::InboxGroup::Questions => w::InboxGroup::Questions,
            c::InboxGroup::Proposals => w::InboxGroup::Proposals,
            c::InboxGroup::Escalations => w::InboxGroup::Escalations,
            c::InboxGroup::PersonTasks => w::InboxGroup::PersonTasks,
            c::InboxGroup::Results => w::InboxGroup::Results,
            c::InboxGroup::Replies => w::InboxGroup::Replies,
        }),
        None => None,
    }
}

/// The read exchange's owner envelope; family records remain opaque.
pub fn owned(value: Readable, limits: &w::Limits) -> Result<w::Owned, w::Problem> {
    let (owner, version, record) = match value {
        Readable::Jig(value) => {
            let value = read(&value, limits)?;
            let mut writer = Writer::new(usize::try_from(value.measure()).expect("u32 fits usize"));
            value.encode(&mut writer).expect("measured read");
            (w::Owner::Jig, VERSION, writer.finish())
        }
        Readable::Family(value) => {
            let owner = w::Family::new(limits, w::FamilyParts { number: value.family })?;
            (w::Owner::Family(owner), value.version, value.bytes)
        }
    };
    w::Owned::new(limits, w::OwnedParts { owner, version, record })
}
#[must_use]
pub fn none(value: &w::ReadNone) -> c::ReadNone {
    match value {
        w::ReadNone::Unknown => c::ReadNone::Unknown,
        w::ReadNone::Refused => c::ReadNone::Refused,
        w::ReadNone::Gone => c::ReadNone::Gone,
        w::ReadNone::Busy => c::ReadNone::Busy,
        w::ReadNone::SignedOut => c::ReadNone::SignedOut,
    }
}
