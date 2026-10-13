#![expect(clippy::manual_map, reason = "step code matches optional owner facts without closures")]
//! Whole live document facts (protocol/client.md, 3 and 7).
use super::{live, pieces, task, w};
use alloc::boxed::Box;
use jig_domain_core::documents as d;
use skein_lib::{List, Writer};
pub fn object(value: d::InboxObject, limits: &w::Limits) -> Result<w::OwnerInboxObject, w::Problem> {
    Ok(match value {
        d::InboxObject::Question { message } => w::OwnerInboxObject::Question(w::OwnerInboxObjectQuestion::new(
            limits,
            w::OwnerInboxObjectQuestionParts { message },
        )?),
        d::InboxObject::Proposal { proposer, number } => {
            w::OwnerInboxObject::Proposal(w::OwnerInboxObjectProposal::new(
                limits,
                w::OwnerInboxObjectProposalParts {
                    proposer: match proposer {
                        jig_domain_people::Proposer::Task(value) => {
                            w::Proposer::Task(w::ProposerTask::new(limits, w::ProposerTaskParts { value })?)
                        }
                        jig_domain_people::Proposer::Party(value) => {
                            w::Proposer::Party(w::ProposerParty::new(limits, w::ProposerPartyParts { value })?)
                        }
                    },
                    number,
                },
            )?)
        }
        d::InboxObject::Escalation { revision } => w::OwnerInboxObject::Escalation(w::OwnerInboxObjectEscalation::new(
            limits,
            w::OwnerInboxObjectEscalationParts { revision },
        )?),
        d::InboxObject::PersonTask => w::OwnerInboxObject::PersonTask,
        d::InboxObject::Reply { message } => w::OwnerInboxObject::Reply(w::OwnerInboxObjectReply::new(
            limits,
            w::OwnerInboxObjectReplyParts { message },
        )?),
        d::InboxObject::Result { position } => w::OwnerInboxObject::Result(w::OwnerInboxObjectResult::new(
            limits,
            w::OwnerInboxObjectResultParts { position },
        )?),
    })
}
pub fn cursor(value: d::Cursor, limits: &w::Limits) -> Result<w::OwnerInboxCursor, w::Problem> {
    Ok(match value {
        d::Cursor::Number { number } => w::OwnerInboxCursor::Number(w::OwnerInboxCursorNumber::new(
            limits,
            w::OwnerInboxCursorNumberParts { number },
        )?),
        d::Cursor::Inbox { task, object: value, at } => w::OwnerInboxCursor::Inbox(w::OwnerInboxCursorInbox::new(
            limits,
            w::OwnerInboxCursorInboxParts {
                task,
                object: object(value, limits)?,
                at: match at {
                    Some(item) => Some(item.as_nanos()),
                    None => None,
                },
            },
        )?),
    })
}
pub fn entry(value: &d::InboxEntry, limits: &w::Limits) -> Result<w::OwnerInboxEntry, w::Problem> {
    w::OwnerInboxEntry::new(
        limits,
        w::OwnerInboxEntryParts {
            reply: match &value.reply {
                Some(item) => Some(live::conversation(item, limits)?),
                None => None,
            },
            task: task::link(&value.task, limits)?,
            project: value.project,
            object: object(value.object, limits)?,
            activity: value.activity,
            at: match value.at {
                Some(item) => Some(item.as_nanos()),
                None => None,
            },
            unread: value.unread,
            cursor: cursor(value.cursor, limits)?,
        },
    )
}
pub fn selection(
    value: Option<&List<d::InboxSelection>>,
    limits: &w::Limits,
    bound: u32,
    path: w::Path,
) -> Result<Option<Box<[u8]>>, w::Problem> {
    let Some(value) = value else { return Ok(None) };
    let mut rows = List::with_capacity(value.len());
    for row in value {
        let mut goals = List::with_capacity(row.goals.len());
        for number in &row.goals {
            goals.push(*number).expect("supplied roots fit");
        }
        rows.push(w::OwnerInboxSelection::new(
            limits,
            w::OwnerInboxSelectionParts { cursor: cursor(row.cursor, limits)?, goals },
        )?)
        .expect("supplied selections fit");
    }
    Ok(Some(selection_pieces(&rows, bound, path)?))
}
pub fn document(value: &d::Inbox, limits: &w::Limits) -> Result<w::OwnerInbox, w::Problem> {
    let mut needs = List::with_capacity(value.needs.len());
    for row in &value.needs {
        needs.push(entry(row, limits)?).expect("supplied rows fit");
    }
    let needs = entry_pieces(&needs, limits.owner_inbox_needs, w::Path::OwnerInboxNeeds)?;
    let mut updates = List::with_capacity(value.updates.len());
    for row in &value.updates {
        updates.push(entry(row, limits)?).expect("supplied rows fit");
    }
    let updates = entry_pieces(&updates, limits.owner_inbox_updates, w::Path::OwnerInboxUpdates)?;

    w::OwnerInbox::new(
        limits,
        w::OwnerInboxParts {
            party: value.party,
            needs,
            updates,
            next: match value.next {
                Some(item) => Some(cursor(item, limits)?),
                None => None,
            },
            cached_needs: value.cached_needs,
            cached_updates: live::count(value.cached_updates, limits)?,
            results_seen: value.results_seen,
            updates_time_complete: value.updates_time_complete,
            selection: selection(
                value.selection.as_ref(),
                limits,
                limits.owner_inbox_selection_some,
                w::Path::OwnerInboxSelection,
            )?,
        },
    )
}
fn entry_pieces(rows: &List<w::OwnerInboxEntry>, bound: u32, path: w::Path) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(usize::try_from(row.measure()).expect("u32 measure fits usize"));
        match row.encode(&mut out) {
            Ok(()) => (),
            Err(_) => return Err(w::Problem { path, reason: skein_codec::Reason::Bound }),
        }
        encoded.push(out.finish()).expect("supplied rows fit");
    }
    pieces(&encoded, bound, path)
}
fn selection_pieces(rows: &List<w::OwnerInboxSelection>, bound: u32, path: w::Path) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(usize::try_from(row.measure()).expect("u32 measure fits usize"));
        match row.encode(&mut out) {
            Ok(()) => (),
            Err(_) => return Err(w::Problem { path, reason: skein_codec::Reason::Bound }),
        }
        encoded.push(out.finish()).expect("supplied rows fit");
    }
    pieces(&encoded, bound, path)
}
