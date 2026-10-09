//! One-terminal reads, bounded pages and opaque cursors; client/domain/contract.md, 6.
//! A cursor is never decoded by the client. Ended work is read, live work watched.
use super::{
    ActivityLine, ConversationLine, Deployment, Escalation, Frame, GoalLine, HistoryLine, InboxEntry, InboxGroup, Note,
    NoteScope, PersonTask, Project, Proposal, ProposalKey, Question, Result, Task, Text,
};
use alloc::boxed::Box;
use skein_lib::List;
/// An opaque page continuation within its byte bound.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Cursor {
    pub value: Text,
}

/// A document or a bounded page requested without a key.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Read {
    /// The frame.
    Frame,
    /// One task.
    Task { task: u64 },
    /// One proposal.
    Proposal { proposal: ProposalKey },
    /// One escalation.
    Escalation { task: u64 },
    /// One question.
    Question { task: u64, message: u64 },
    /// One person task.
    PersonTask { task: u64 },
    /// One result.
    Result { task: u64 },
    /// Ended delegates.
    Ended { task: u64, after: Option<Cursor> },
    /// Task or subtree history.
    History { task: u64, subtree: bool, after: Option<Cursor> },
    /// Activity read from the end.
    Activity { task: u64, before: Option<Cursor> },
    /// A filtered inbox group.
    InboxGroup { group: InboxGroup, project: Option<u32>, after: Option<Cursor> },
    /// A filtered inbox window, including a goal subtree filter.
    Inbox { project: Option<u32>, kind: Option<InboxGroup>, goal: Option<u64>, after: Option<Cursor> },
    /// Party conversations.
    Conversations { project: Option<u32>, after: Option<Cursor> },
    /// Live or ended goals.
    Goals { project: u32, ended: bool, after: Option<Cursor> },
    /// Project spend by period, with an opaque continuation.
    Spend { project: u32, after: Option<Cursor> },
    /// One project.
    Project { project: u32 },
    /// Scoped notes.
    Notes { scope: NoteScope, description: Option<Text>, after: Option<Cursor> },
    /// Deployment state.
    Deployment,
}

/// A bounded page with opaque cursors at both ends.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ObjectPage {
    pub rows: List<Result>,
    pub before: Option<Cursor>,
    pub after: Option<Cursor>,
}

/// A bounded page with opaque cursors at both ends.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ConversationPage {
    pub rows: List<ConversationLine>,
    pub before: Option<Cursor>,
    pub after: Option<Cursor>,
}

/// A bounded page with opaque cursors at both ends.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ActivityPage {
    pub rows: List<ActivityLine>,
    pub before: Option<Cursor>,
    pub after: Option<Cursor>,
}

/// A bounded page with opaque cursors at both ends.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct HistoryPage {
    pub rows: List<HistoryLine>,
    pub before: Option<Cursor>,
    pub after: Option<Cursor>,
}

/// A bounded page with opaque cursors at both ends.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct InboxPage {
    pub rows: List<InboxEntry>,
    pub before: Option<Cursor>,
    pub after: Option<Cursor>,
}

/// A bounded page with opaque cursors at both ends.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GoalPage {
    pub rows: List<GoalLine>,
    pub before: Option<Cursor>,
    pub after: Option<Cursor>,
}

/// A bounded page with opaque cursors at both ends.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NotePage {
    pub rows: List<Note>,
    pub before: Option<Cursor>,
    pub after: Option<Cursor>,
}

/// A read document or page.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ReadValue {
    /// The frame.
    Frame(Box<Frame>),
    /// A task.
    Task(Box<Task>),
    /// A proposal.
    Proposal(Box<Proposal>),
    /// An escalation.
    Escalation(Box<Escalation>),
    /// A question.
    Question(Box<Question>),
    /// A person task.
    PersonTask(Box<PersonTask>),
    /// A result.
    Result(Box<Result>),
    /// Ended delegate results.
    Ended(Box<ObjectPage>),
    /// History lines.
    History(Box<HistoryPage>),
    /// Committed activity.
    Activity(Box<ActivityPage>),
    /// Inbox entries.
    Inbox(Box<InboxPage>),
    /// Conversations.
    Conversations(Box<ConversationPage>),
    /// Goals.
    Goals(Box<GoalPage>),
    /// Project spend periods.
    Spend(Box<SpendPage>),
    /// A project.
    Project(Box<Project>),
    /// Notes.
    Notes(Box<NotePage>),
    /// Deployment state.
    Deployment(Box<Deployment>),
}

/// Why one read has no value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ReadNone {
    /// No object the party may see.
    Unknown,
    /// Not admitted.
    Refused,
    /// No longer retained.
    Gone,
    /// The attempt lost its connection.
    Unreachable,
    /// The engine loads are full.
    Busy,
    /// The sign in ended.
    SignedOut,
}

/// The one terminal of a read.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ReadResult {
    /// The committed value.
    Value(ReadValue),
    /// Why no value came.
    None(ReadNone),
}

/// Bounded past spend periods with opaque page ends.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SpendPage {
    pub rows: List<super::SpendPeriod>,
    pub before: Option<Cursor>,
    pub after: Option<Cursor>,
}
