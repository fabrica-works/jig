//! Physical continuations for durable indexes (store.md 7.4).
use super::{Filter, Group, Position, Scope};
use crate::documents::Proposer;
use crate::{Key, documents};
use jig_domain_tasks as tasks;

/// Continue a live window in the index's physical order.
#[must_use]
pub fn index_key(scope: &Scope, party: u64, cursor: documents::Cursor, activity: Option<u64>) -> Option<Key> {
    if *scope == Scope::Conversations {
        return match cursor {
            documents::Cursor::Number { number } => {
                Some(Key::Tasks(tasks::Key::ConversationNewest { party, task: tasks::Newest::of(number) }))
            }
            documents::Cursor::Inbox { .. } => None,
        };
    }
    let Scope::Inbox { group: Group::Updates | Group::Results | Group::Replies, .. } = scope else { return None };
    let documents::Cursor::Inbox { task, at: Some(at), object } = cursor else { return None };
    let object = match object {
        documents::InboxObject::Result { .. } => tasks::UpdateObject::Result,
        documents::InboxObject::Reply { message } => {
            tasks::UpdateObject::Reply { message: tasks::Newest::of(message), activity: tasks::Newest::of(activity?) }
        }
        documents::InboxObject::Question { .. }
        | documents::InboxObject::Proposal { .. }
        | documents::InboxObject::Escalation { .. }
        | documents::InboxObject::PersonTask => return None,
    };
    Some(Key::Tasks(tasks::Update { party, at, task, object }.key()))
}

pub(super) fn inbox_position(
    party: u64,
    group: Group,
    cursor: documents::Cursor,
    activity: Option<u64>,
) -> Option<Position> {
    if group.updates() {
        let key =
            index_key(&Scope::Inbox { group, filter: Filter { project: None, goal: None } }, party, cursor, activity)?;
        return Some(Position::Owner(key));
    }
    match cursor {
        documents::Cursor::Inbox { task, object, at } => {
            let key = match group {
                Group::Updates | Group::Results | Group::Replies => return None,
                Group::Needs | Group::Proposals | Group::Escalations | Group::Questions | Group::PersonTasks => {
                    match object {
                        documents::InboxObject::Proposal { proposer: Proposer::Party(_), number } => {
                            Key::Tasks(tasks::Key::PersonProposal(number))
                        }
                        documents::InboxObject::Question { .. }
                        | documents::InboxObject::Proposal { proposer: Proposer::Task(_), .. }
                        | documents::InboxObject::Escalation { .. }
                        | documents::InboxObject::PersonTask => Key::Tasks(tasks::Key::Live(task)),
                        documents::InboxObject::Result { .. } | documents::InboxObject::Reply { .. } => return None,
                    }
                }
            };
            Some(Position::Inbox { at, task, object, key })
        }
        documents::Cursor::Number { .. } => None,
    }
}

/// Verify the index's party and its joins before projecting an update.
pub(super) fn valid_update(party: u64, row: &super::Row) -> bool {
    let crate::Record::Tasks(tasks::Stored::Update(update)) = &row.value else { return false };
    let Some(owner) = &row.owner else { return false };
    if update.party != party || update.task != owner.number {
        return false;
    }
    let (object, at) = match update.object {
        tasks::UpdateObject::Result => {
            if let Some(at) = owner.ended_at
                && at != update.at
            {
                return false;
            }
            (documents::InboxObject::Result { position: owner.result_position }, owner.ended_at)
        }
        tasks::UpdateObject::Reply { message, activity } => {
            let Some(crate::Record::Tasks(tasks::Stored::Activity(line))) = &row.related else { return false };
            if line.task != update.task || line.position != activity.value() || line.at != update.at {
                return false;
            }
            let number = match &line.line {
                tasks::Line::Turn { reply, .. } => *reply,
                tasks::Line::Notified { word, .. } => word.number,
                tasks::Line::Words { .. }
                | tasks::Line::Heard(_)
                | tasks::Line::RunBegan { .. }
                | tasks::Line::RunEnded { .. }
                | tasks::Line::Decided { .. }
                | tasks::Line::Milestone { .. } => return false,
            };
            if number != message.value() {
                return false;
            }
            (documents::InboxObject::Reply { message: number }, Some(update.at))
        }
    };
    row.inbox == Some(documents::Cursor::Inbox { task: update.task, object, at })
}

pub(super) fn is_update(key: &Key) -> bool {
    match key {
        Key::Tasks(tasks::Key::Update { .. }) => true,
        Key::Tasks(
            tasks::Key::Live(_)
            | tasks::Key::Ended(_)
            | tasks::Key::Stub(_)
            | tasks::Key::Ledger(_)
            | tasks::Key::ClosedLedger(_)
            | tasks::Key::DecidedProposal(_)
            | tasks::Key::Writer(_)
            | tasks::Key::Pool(_)
            | tasks::Key::PersonProposal(_)
            | tasks::Key::History { .. }
            | tasks::Key::Milestone { .. }
            | tasks::Key::Activity { .. }
            | tasks::Key::Result { .. }
            | tasks::Key::ConversationNewest { .. }
            | tasks::Key::Goal { .. }
            | tasks::Key::EndedDelegate { .. }
            | tasks::Key::Delegate { .. }
            | tasks::Key::ConversationOldest { .. },
        )
        | Key::Core(_)
        | Key::People(_)
        | Key::Notes(_) => false,
    }
}

/// Name the same conversation in the family for its continuation's direction.
pub(super) fn conversation_position(position: &Position, newest: bool) -> Option<Position> {
    let (party, task) = match position {
        Position::Owner(Key::Tasks(tasks::Key::ConversationNewest { party, task })) => (*party, task.value()),
        Position::Owner(Key::Tasks(tasks::Key::ConversationOldest { party, task })) => (*party, *task),
        Position::Owner(_) | Position::Inbox { .. } => return None,
    };
    let key = if newest {
        tasks::Key::ConversationNewest { party, task: tasks::Newest::of(task) }
    } else {
        tasks::Key::ConversationOldest { party, task }
    };
    Some(Position::Owner(Key::Tasks(key)))
}

pub(super) fn conversation_row(
    core: &crate::Core,
    limits: &crate::Limits,
    range: &super::Range,
    row: &super::Row,
) -> Result<Option<super::PageRow>, documents::Missing> {
    let conversation = match &row.value {
        crate::Record::Tasks(tasks::Stored::ConversationNewest(row)) if range.backwards => row,
        crate::Record::Tasks(tasks::Stored::ConversationOldest(row)) if !range.backwards => row,
        crate::Record::Tasks(
            tasks::Stored::Result(_)
            | tasks::Stored::ConversationNewest(_)
            | tasks::Stored::Goal(_)
            | tasks::Stored::EndedDelegate(_)
            | tasks::Stored::Delegate(_)
            | tasks::Stored::ConversationOldest(_)
            | tasks::Stored::ClosedLedger(_)
            | tasks::Stored::DecidedProposal(_)
            | tasks::Stored::Update(_)
            | tasks::Stored::Activity(_)
            | tasks::Stored::Live(_)
            | tasks::Stored::Ended(_)
            | tasks::Stored::Stub(_)
            | tasks::Stored::Ledger(_)
            | tasks::Stored::Pool(_)
            | tasks::Stored::Writer(_)
            | tasks::Stored::History(_)
            | tasks::Stored::Milestone(_)
            | tasks::Stored::PersonProposal(_),
        )
        | crate::Record::Core(_)
        | crate::Record::People(_)
        | crate::Record::Notes(_) => return Ok(None),
    };
    let Some(owner) = &row.owner else { return Ok(None) };
    if conversation.party != range.party
        || conversation.task != owner.number
        || core.chat_requester(owner) != Some(range.party)
    {
        return Ok(None);
    }
    let ended = match &owner.phase {
        tasks::Phase::Ended(_) => true,
        tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Held { .. } => false,
    };
    if !ended || core.tasks.task(owner.number).is_some() {
        return Ok(None);
    }
    Ok(Some(super::PageRow::Conversation(documents::conversation_from_row(core, limits, range.party, owner)?)))
}

/// The goal index carries its full ended census; its owner is a point join.
pub(super) fn goal_row(
    core: &crate::Core,
    limits: &crate::Limits,
    project: u32,
    row: &super::Row,
) -> Result<Option<super::PageRow>, documents::Missing> {
    let goal = match &row.value {
        crate::Record::Tasks(tasks::Stored::Goal(goal)) => goal,
        crate::Record::Tasks(
            tasks::Stored::Delegate(_)
            | tasks::Stored::EndedDelegate(_)
            | tasks::Stored::Update(_)
            | tasks::Stored::Result(_)
            | tasks::Stored::ConversationNewest(_)
            | tasks::Stored::ConversationOldest(_)
            | tasks::Stored::Activity(_)
            | tasks::Stored::Milestone(_)
            | tasks::Stored::PersonProposal(_)
            | tasks::Stored::DecidedProposal(_)
            | tasks::Stored::ClosedLedger(_)
            | tasks::Stored::History(_)
            | tasks::Stored::Live(_)
            | tasks::Stored::Ended(_)
            | tasks::Stored::Stub(_)
            | tasks::Stored::Ledger(_)
            | tasks::Stored::Writer(_)
            | tasks::Stored::Pool(_),
        )
        | crate::Record::Core(_)
        | crate::Record::People(_)
        | crate::Record::Notes(_) => return Ok(None),
    };
    let Some(owner) = &row.owner else { return Ok(None) };
    if goal.project != project
        || owner.project != project
        || goal.task != owner.number
        || owner.tracked.is_none()
        || core.tasks.task(owner.number).is_some()
    {
        return Ok(None);
    }
    match owner.phase {
        tasks::Phase::Ended(_) => {}
        tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Held { .. } => {
            return Ok(None);
        }
    }
    Ok(Some(super::PageRow::Goal(documents::GoalLine {
        chip: documents::chip_from_row_with_core(core, limits, owner)?,
        priority: owner.tracked.expect("tracked goal"),
        phases: documents::PhaseCounts::from_endings(goal.endings),
        budget: documents::budget_from_row(owner),
        held_below: 0,
    })))
}

/// An ended-delegate index row is joined to its ended owner by task number.
pub(super) fn ended_delegate_row(
    limits: &crate::Limits,
    requester: u64,
    row: &super::Row,
) -> Result<Option<super::PageRow>, documents::Missing> {
    let edge = match &row.value {
        crate::Record::Tasks(tasks::Stored::EndedDelegate(edge)) => edge,
        crate::Record::Tasks(
            tasks::Stored::Delegate(_)
            | tasks::Stored::Goal(_)
            | tasks::Stored::Update(_)
            | tasks::Stored::Result(_)
            | tasks::Stored::ConversationNewest(_)
            | tasks::Stored::ConversationOldest(_)
            | tasks::Stored::Activity(_)
            | tasks::Stored::Milestone(_)
            | tasks::Stored::PersonProposal(_)
            | tasks::Stored::DecidedProposal(_)
            | tasks::Stored::ClosedLedger(_)
            | tasks::Stored::History(_)
            | tasks::Stored::Live(_)
            | tasks::Stored::Ended(_)
            | tasks::Stored::Stub(_)
            | tasks::Stored::Ledger(_)
            | tasks::Stored::Writer(_)
            | tasks::Stored::Pool(_),
        )
        | crate::Record::Core(_)
        | crate::Record::People(_)
        | crate::Record::Notes(_) => return Ok(None),
    };
    let Some(owner) = &row.owner else { return Ok(None) };
    if edge.requester != requester || edge.task != owner.number || owner.requester != tasks::Party::Task(requester) {
        return Ok(None);
    }
    match owner.phase {
        tasks::Phase::Ended(_) => {}
        tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Held { .. } => {
            return Ok(None);
        }
    }
    Ok(Some(super::PageRow::EndedDelegate(documents::ended_delegate_from_row(limits, owner)?)))
}
