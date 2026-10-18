//! Exact Inbox owner facts, separate from application supplied context.
//! See client/domain/contract.md, 5–7 and pages.md, 3. Raw row identity is
//! independent of a coarser card address and both continuation cursors.
use super::{Count, Cursor, Inbox, InboxEntry, InboxGroup, ObjectKey, OwnerTaskLink, ProposalKey, Proposer, TaskLine};
use alloc::boxed::Box;
use skein_lib::{List, Wall};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OwnerInboxObject {
    Question { message: u64 },
    Proposal { proposer: Proposer, number: u64 },
    Escalation { revision: u64 },
    PersonTask,
    Reply { message: u64 },
    Result { position: u64 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OwnerInboxCursor {
    Number { number: u64 },
    Inbox { task: u64, object: OwnerInboxObject, at: Option<Wall> },
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OwnerInboxEntry {
    pub reply: Option<Box<TaskLine>>,
    pub task: OwnerTaskLink,
    pub project: Option<u32>,
    pub object: OwnerInboxObject,
    pub activity: Option<u64>,
    pub at: Option<Wall>,
    pub unread: Option<bool>,
    pub cursor: OwnerInboxCursor,
}
/// Parallel owner selection facts; roots are self followed by the exact supplied
/// bounded ancestry. A row cursor associates context, never replaces row identity.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OwnerInboxSelection {
    pub cursor: OwnerInboxCursor,
    pub goals: List<u64>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OwnerInbox {
    pub party: u64,
    pub needs: List<OwnerInboxEntry>,
    pub updates: List<OwnerInboxEntry>,
    pub next: Option<OwnerInboxCursor>,
    pub cached_needs: u32,
    pub cached_updates: Count,
    pub results_seen: u64,
    pub updates_time_complete: bool,
    /// None is unsupplied; Some(empty) is a supplied empty selection collection.
    pub selection: Option<List<OwnerInboxSelection>>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OwnerInboxPage {
    /// Original retained-row selection facts, independent of current live ancestry.
    pub selection: Option<List<OwnerInboxSelection>>,
    pub cut: bool,
    pub rows: List<OwnerInboxEntry>,
    pub before: Option<Cursor>,
    pub after: Option<Cursor>,
}
/// A whole live envelope; no owner aggregate is split into per-kind counts.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum InboxDocument {
    Entry(Box<Inbox>),
    Owner(Box<OwnerInbox>),
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum InboxRow {
    Entry(Box<InboxEntry>),
    Owner(Box<OwnerInboxEntry>),
}
/// Raw scoped row identity, independent of row/read cursors or card identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InboxIdentity {
    Entry(ObjectKey),
    Owner { task: u64, object: OwnerInboxObject },
}
impl OwnerInboxEntry {
    #[must_use]
    pub const fn task_number(&self) -> u64 {
        match &self.task {
            OwnerTaskLink::Live(chip) | OwnerTaskLink::Retained { chip, .. } => chip.task,
            OwnerTaskLink::Stored { number, .. } => *number,
        }
    }
    #[must_use]
    pub const fn group(&self) -> InboxGroup {
        match self.object {
            OwnerInboxObject::Question { .. } => InboxGroup::Questions,
            OwnerInboxObject::Proposal { .. } => InboxGroup::Proposals,
            OwnerInboxObject::Escalation { .. } => InboxGroup::Escalations,
            OwnerInboxObject::PersonTask => InboxGroup::PersonTasks,
            OwnerInboxObject::Reply { .. } => InboxGroup::Replies,
            OwnerInboxObject::Result { .. } => InboxGroup::Results,
        }
    }
    #[must_use]
    pub const fn card_key(&self) -> ObjectKey {
        let task = self.task_number();
        match self.object {
            OwnerInboxObject::Question { message } => ObjectKey::Question { task, message },
            OwnerInboxObject::Proposal { proposer, number } => ObjectKey::Proposal(ProposalKey { proposer, number }),
            OwnerInboxObject::Escalation { .. } => ObjectKey::Escalation(task),
            OwnerInboxObject::PersonTask => ObjectKey::PersonTask(task),
            OwnerInboxObject::Reply { .. } => ObjectKey::Task(task),
            OwnerInboxObject::Result { .. } => ObjectKey::Result(task),
        }
    }
}
impl InboxRow {
    #[must_use]
    pub const fn identity(&self) -> InboxIdentity {
        match self {
            Self::Entry(value) => InboxIdentity::Entry(value.object),
            Self::Owner(value) => InboxIdentity::Owner { task: value.task_number(), object: value.object },
        }
    }
    #[must_use]
    pub const fn card_key(&self) -> ObjectKey {
        match self {
            Self::Entry(value) => value.object,
            Self::Owner(value) => value.card_key(),
        }
    }
    #[must_use]
    pub const fn group(&self) -> InboxGroup {
        match self {
            Self::Entry(value) => value.group,
            Self::Owner(value) => value.group(),
        }
    }
    #[must_use]
    pub const fn project(&self) -> Option<u32> {
        match self {
            Self::Entry(value) => Some(value.project),
            Self::Owner(value) => value.project,
        }
    }
    #[must_use]
    pub const fn at(&self) -> Option<Wall> {
        match self {
            Self::Entry(value) => Some(value.at),
            Self::Owner(value) => value.at,
        }
    }
    #[must_use]
    pub const fn goal(&self) -> Option<u64> {
        match self {
            Self::Entry(value) => value.goal,
            Self::Owner(_) => None,
        }
    }
    #[must_use]
    pub fn reply(&self) -> Option<&TaskLine> {
        match self {
            Self::Entry(value) => value.reply.as_deref(),
            Self::Owner(value) => value.reply.as_deref(),
        }
    }
    #[must_use]
    pub const fn turn_position(&self) -> Option<u64> {
        match self {
            Self::Entry(value) => match value.group {
                InboxGroup::Replies => Some(value.position),
                InboxGroup::Questions
                | InboxGroup::Proposals
                | InboxGroup::Escalations
                | InboxGroup::PersonTasks
                | InboxGroup::Results => None,
            },
            Self::Owner(value) => match value.object {
                OwnerInboxObject::Reply { message } => Some(message),
                OwnerInboxObject::Question { .. }
                | OwnerInboxObject::Proposal { .. }
                | OwnerInboxObject::Escalation { .. }
                | OwnerInboxObject::PersonTask
                | OwnerInboxObject::Result { .. } => None,
            },
        }
    }
    #[must_use]
    pub const fn result_position(&self) -> Option<u64> {
        match self {
            Self::Entry(value) => match value.group {
                InboxGroup::Results => Some(value.position),
                InboxGroup::Questions
                | InboxGroup::Proposals
                | InboxGroup::Escalations
                | InboxGroup::PersonTasks
                | InboxGroup::Replies => None,
            },
            Self::Owner(value) => match value.object {
                OwnerInboxObject::Result { position } => Some(position),
                OwnerInboxObject::Question { .. }
                | OwnerInboxObject::Proposal { .. }
                | OwnerInboxObject::Escalation { .. }
                | OwnerInboxObject::PersonTask
                | OwnerInboxObject::Reply { .. } => None,
            },
        }
    }
}
