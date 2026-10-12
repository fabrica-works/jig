//! Read pages and the owner-keyed store transport (`contract.md`, section 6).
//! Cursors name a query and an owner key. The client only echoes them.
use crate::documents::{self, Document, Proposer};
use crate::{Core, CoreRecord, Key, Record};
use alloc::boxed::Box;
use core::mem::{size_of, size_of_val};
use jig_domain_notes as notes;
use jig_domain_tasks as tasks;
use skein_lib::{List, Wall};

/// Which inbox group the page selects.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Group {
    /// Objects awaiting the party, oldest first.
    Needs,
    /// Unread outcomes and replies, newest first.
    Updates,
    /// Pending proposals, oldest first.
    Proposals,
    /// Pending escalations, oldest first.
    Escalations,
    /// Unanswered questions, oldest first.
    Questions,
    /// Active person tasks, oldest first.
    PersonTasks,
    /// Unread results, newest first.
    Results,
    /// Unread chat replies, newest first.
    Replies,
}

impl Group {
    pub(crate) fn updates(self) -> bool {
        match self {
            Self::Updates | Self::Results | Self::Replies => true,
            Self::Needs | Self::Proposals | Self::Escalations | Self::Questions | Self::PersonTasks => false,
        }
    }

    pub(crate) fn includes(self, object: documents::InboxObject) -> bool {
        match object {
            documents::InboxObject::Proposal { .. } => self == Self::Needs || self == Self::Proposals,
            documents::InboxObject::Escalation { .. } => self == Self::Needs || self == Self::Escalations,
            documents::InboxObject::Question { .. } => self == Self::Needs || self == Self::Questions,
            documents::InboxObject::PersonTask => self == Self::Needs || self == Self::PersonTasks,
            documents::InboxObject::Result { .. } => self == Self::Updates || self == Self::Results,
            documents::InboxObject::Reply { .. } => self == Self::Updates || self == Self::Replies,
        }
    }
}

/// Optional filters on the party's cross-project inbox.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Filter {
    pub project: Option<u32>,
    pub goal: Option<u64>,
}

/// Identity of a store query, independent of its continuation.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Scope {
    /// One retained task.
    Task { task: u64 },
    /// One immutable proposal and its final routing.
    Proposal { proposer: Proposer, proposal: u64 },
    /// Latest kept decision for one escalation.
    Escalation { task: u64 },
    /// Original question and any retained answer.
    Question { task: u64, message: u64 },
    /// One retained person task.
    PersonTask { task: u64 },
    /// One retained result.
    Result { task: u64 },
    /// Exact outward activity named by an inbox reply.
    Reply { task: u64, activity: u64 },
    /// An immutable opaque turn archive.
    Turn { task: u64, attempt: u64, turn: u32 },
    /// Ended direct delegates in task-key order.
    EndedDelegates { task: u64 },
    /// Semantic changes and lifecycle milestones in owner-key order.
    History { task: u64, subtree: bool },
    /// Immutable task activity, newest position first.
    Activity { task: u64 },
    /// One filtered inbox group in its semantic order.
    Inbox { group: Group, filter: Filter },
    /// Ended conversations, newest task first after all live conversations.
    Conversations,
    /// Ended goals after all live goals.
    Goals { project: u32 },
    /// Notes selected by their owning project.
    Notes { project: u32 },
    /// Exact notes owner scope and optional byte substring of its descriptions.
    ScopedNotes { scope: notes::Scope, description: Option<Box<[u8]>> },
    /// Historical project period ledgers.
    Spend { project: u32 },
}

/// Owner key in this query's store order. Inbox indices preserve exact identity
/// and wall ordering across projects without a second live cache.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Position {
    /// An owner's physical record key.
    Owner(Key),
    /// A virtual owner index for an exact inbox object.
    Inbox { at: Option<Wall>, task: u64, object: documents::InboxObject, key: Key },
}

/// Opaque continuation of this exact query.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Cursor {
    /// Remaining live rows, before entering the store range.
    Live { scope: Scope, after: documents::Cursor, store_after: Option<Position> },
    /// Exclusive owner key; `None` starts the stored portion after live rows.
    Store { scope: Scope, after: Option<Position> },
    /// Preceding stored conversations, before this examined owner key.
    ConversationsBefore { before: Position },
    /// Earlier live conversations; absent position starts from the live tail.
    ConversationsLiveBefore { before: Option<documents::Cursor> },
    /// The original conversation list, beginning with its live rows.
    ConversationsStart,
}

/// One document or bounded page, without a request key.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Read {
    /// A connector-owned document within the client's stated role scope.
    Application { scope: documents::ApplicationScope, connector: u16, read: skein_lib::Token },
    /// The authenticated party's frame.
    Frame,
    /// A task overview, live or ended.
    Task { task: u64 },
    /// An immutable proposal.
    Proposal { proposer: Proposer, proposal: u64 },
    /// A held task and its kept decision.
    Escalation { task: u64 },
    /// Whole original question and any kept answer.
    Question { task: u64, message: u64 },
    /// A person form, live or ended.
    PersonTask { task: u64 },
    /// One result.
    Result { task: u64 },
    /// Whole outward words at the exact child activity key.
    Reply { task: u64, activity: u64 },
    /// One committed opaque turn.
    Turn { task: u64, attempt: u64, turn: u32 },
    /// Ended direct delegates.
    EndedDelegates { task: u64, from: Option<Cursor> },
    /// Task or subtree history.
    History { task: u64, subtree: bool, from: Option<Cursor> },
    /// Activity from its end, in decreasing committed position.
    Activity { task: u64, before: Option<Cursor> },
    /// One filtered inbox group.
    Inbox { group: Group, filter: Filter, from: Option<Cursor> },
    /// Live conversations followed by retained ended ones.
    Conversations { from: Option<Cursor> },
    /// Live project goals followed by retained ended ones.
    Goals { project: u32, from: Option<Cursor> },
    /// A project's current resources, policy and funding.
    Project { project: u32 },
    /// Project notes from their owning child's rows.
    Notes { project: u32, from: Option<Cursor> },
    /// Notes in one exact owner scope, optionally searched by description.
    ScopedNotes { scope: notes::Scope, description: Option<Box<[u8]>>, from: Option<Cursor> },
    /// Historical period spending.
    Spend { project: u32, from: Option<Cursor> },
    /// Deployment state.
    Deployment,
}

/// One store request. The transport selects the owner's range, never a cache.
/// `after` is exclusive in the stated order. Historical rows carry their owner
/// and bounded durable requester ancestry for visibility and subtree checks.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Range {
    pub party: u64,
    pub scope: Scope,
    pub after: Option<Position>,
    pub backwards: bool,
    pub rows: u32,
    pub bytes: u64,
}

/// A raw owner row with the facts needed to authenticate it without restoration.
#[derive(PartialEq, Eq, Debug)]
pub struct Row {
    pub value: Record,
    pub owner: Option<Box<tasks::TaskRecord>>,
    /// Requester ancestors nearest first, copied from durable task records.
    pub ancestors: Box<[u64]>,
    /// Related immutable evidence, such as a decision or answered question.
    pub related: Option<Record>,
    /// Existing ancestor, dependency and ended-subtree rows requested by this query.
    pub linked: Box<[Box<tasks::TaskRecord>]>,
    /// The selected reference when an owner row has several waiting objects.
    pub inbox: Option<documents::Cursor>,
}

impl Row {
    /// Complete ownership of this transport row and its requested evidence.
    #[must_use]
    pub fn owned_bytes(&self) -> Option<u64> {
        row_bytes(self)
    }
}

/// The one terminal event of a store load.
#[derive(Debug)]
pub enum Loaded {
    /// Rows within the range's row and byte caps; continuation is exclusive.
    Page { rows: Box<[Row]>, next: Option<Position> },
    /// Transport admission was full and retained nothing.
    Busy,
    /// This range is no longer available.
    Gone,
}

/// An ended object's document and its durable ending facts.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Historical {
    pub value: Past,
    pub at: Option<Wall>,
    pub by: Option<tasks::Party>,
}

/// Typed historical values, made by the same owners as live documents.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Past {
    /// Full ended task overview and retained outcome.
    Task { document: Box<documents::Task>, result: Box<documents::ResultDoc> },
    /// Immutable task proposal and final routing state.
    Proposal(documents::ProposalHistory),
    /// A retained person-origin proposal and its first decision evidence.
    PersonProposal { proposal: Box<tasks::PersonProposal>, decision: Option<crate::ProposalDecisionRecord> },
    /// Kept escalation decision, its task and prior lifecycle story.
    Escalation {
        task: Box<documents::Task>,
        decision: crate::EscalationDecisionRecord,
        hold: Option<documents::MilestoneLine>,
    },
    /// Original question and any retained answer or task ending.
    Question {
        question: documents::Question,
        answer: Option<tasks::ActivityLine>,
        ending: Option<Box<documents::ResultDoc>>,
    },
    /// Retained person form and its outcome.
    PersonTask { document: Box<documents::PersonTask>, result: Box<documents::ResultDoc> },
    /// Whole outward words retained by their producing task.
    Reply(tasks::Word),
    /// Immutable opaque transcript bytes, not decoded by the core.
    Turn(crate::TurnRecord),
}

/// One semantic row of a read page.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum PageRow {
    /// An ended delegate's status and outcome.
    EndedDelegate(documents::EndedDelegate),
    /// A semantic task change.
    History(documents::HistoryLine),
    /// A lifecycle transition.
    Milestone(documents::MilestoneLine),
    /// Whole original words, run boundaries and exact archive references.
    Activity(tasks::ActivityLine),
    /// One waiting object or unread update.
    Inbox(documents::InboxEntry),
    /// One directly requested conversation.
    Conversation(documents::ConversationLine),
    /// One tracked goal.
    Goal(documents::GoalLine),
    /// One loaded note.
    Note(documents::Note),
    /// Historical period accounting, never inferred from live roots.
    Spend(tasks::FundingRecord),
}

/// One bounded list page; no continuation means the range ended.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Page {
    pub rows: List<PageRow>,
    /// Conversation prefix continuation; other scopes supply none.
    pub before: Option<Cursor>,
    pub next: Option<Cursor>,
    pub cut: bool,
    /// Complete original membership contexts for Inbox rows only, when supplied.
    pub inbox_selection: Option<List<documents::InboxSelection>>,
}

/// Exactly one terminal, held until its preceding decisions are durable.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Answer {
    /// A current document.
    Document(Box<Document>),
    /// An ended or decided object, with kept actor and wall time.
    Historical(Box<Historical>),
    /// A bounded list page.
    Page(Box<Page>),
    /// No value is available.
    None(Why),
}

/// Refusal facts which reveal no invisible object identity.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Why {
    /// Missing, or invisible to the party.
    Unknown,
    /// A construction or load bound prevents admission.
    Busy,
    /// Required historical rows are no longer retained.
    Gone,
    /// The read's sign-in is unknown to a ready engine, ended or expired.
    SignedOut,
}

pub(crate) fn row_owner(row: &Row) -> Option<&tasks::TaskRecord> {
    match &row.value {
        Record::Tasks(tasks::Stored::Live(task) | tasks::Stored::Ended(task)) => Some(task),
        Record::Tasks(
            tasks::Stored::Activity(_)
            | tasks::Stored::PersonProposal(_)
            | tasks::Stored::History(_)
            | tasks::Stored::Milestone(_)
            | tasks::Stored::Stub(_)
            | tasks::Stored::Ledger(_)
            | tasks::Stored::Writer(_)
            | tasks::Stored::Result(_)
            | tasks::Stored::ConversationNewest(_)
            | tasks::Stored::Goal(_)
            | tasks::Stored::EndedDelegate(_)
            | tasks::Stored::Delegate(_)
            | tasks::Stored::ConversationOldest(_)
            | tasks::Stored::Update(_)
            | tasks::Stored::Pool(_)
            | tasks::Stored::ClosedLedger(_)
            | tasks::Stored::DecidedProposal(_),
        )
        | Record::Core(_)
        | Record::People(_)
        | Record::Notes(_) => row.owner.as_deref(),
    }
}

pub(crate) fn row_visible(core: &Core, party: u64, row: &Row) -> bool {
    match &row.value {
        Record::Tasks(tasks::Stored::PersonProposal(proposal) | tasks::Stored::DecidedProposal(proposal)) => {
            core.people.role(party, proposal.project).is_some()
        }
        Record::Tasks(tasks::Stored::Ledger(ledger) | tasks::Stored::ClosedLedger(ledger)) => match ledger.funder {
            tasks::Funder::Period { project, .. } => core.people.role(party, project).is_some(),
            tasks::Funder::Pool { project, person, .. } => {
                person == party && core.people.role(party, project).is_some()
            }
            tasks::Funder::Task(_) | tasks::Funder::Recurring { .. } => false,
        },
        Record::Notes(notes::Record::Entry(entry)) => match &entry.scope {
            notes::Scope::Deployment => core.people.role(party, 0).is_some(),
            notes::Scope::Project { project }
            | notes::Scope::Goal { project, .. }
            | notes::Scope::Resources { project, .. } => core.people.role(party, *project).is_some(),
        },
        Record::Core(_)
        | Record::Tasks(
            tasks::Stored::Live(_)
            | tasks::Stored::Ended(_)
            | tasks::Stored::Activity(_)
            | tasks::Stored::History(_)
            | tasks::Stored::Milestone(_)
            | tasks::Stored::Stub(_)
            | tasks::Stored::Writer(_)
            | tasks::Stored::Result(_)
            | tasks::Stored::ConversationNewest(_)
            | tasks::Stored::Goal(_)
            | tasks::Stored::EndedDelegate(_)
            | tasks::Stored::Delegate(_)
            | tasks::Stored::ConversationOldest(_)
            | tasks::Stored::Update(_)
            | tasks::Stored::Pool(_),
        ) => match row_owner(row) {
            Some(owner) => core.people.role(party, owner.project).is_some(),
            None => false,
        },
        Record::People(_) | Record::Notes(notes::Record::Line(_)) => false,
    }
}

pub(crate) fn record_bytes(value: &Record) -> Option<u64> {
    let carrier = u64::try_from(size_of::<Record>()).ok()?;
    let owned = match value {
        Record::Tasks(value) => tasks::stored_bytes(value)?,
        Record::Core(CoreRecord::Turn(value)) => u64::try_from(value.transcript.len()).ok()?,
        Record::Core(CoreRecord::Terminal(value)) => tasks::terminal_bytes(&value.end)?,
        Record::Core(CoreRecord::EscalationDecision(value)) => match &value.decision {
            jig_domain_people::EscalationDecision::Reject { reason } => u64::try_from(reason.len()).ok()?,
            jig_domain_people::EscalationDecision::Release | jig_domain_people::EscalationDecision::Pass => 0,
        },
        Record::Core(CoreRecord::ProposalDecision(_)) => 0,
        Record::Notes(notes::Record::Entry(entry)) => u64::try_from(entry.body.len())
            .ok()?
            .checked_add(u64::try_from(entry.description.len()).ok()?)?
            .checked_add(List::<u64>::worst_case(entry.references.len())?)?
            .checked_add(match &entry.scope {
                notes::Scope::Deployment | notes::Scope::Project { .. } | notes::Scope::Goal { .. } => 0,
                notes::Scope::Resources { pattern, .. } => {
                    let mut total = u64::try_from(size_of_val(pattern)).ok()?;
                    total = total.checked_add(u64::try_from(size_of_val(pattern.segments.as_ref())).ok()?)?;
                    total = total.checked_add(match &pattern.last {
                        notes::Last::Exact(bytes) | notes::Last::Open(bytes) => u64::try_from(bytes.len()).ok()?,
                    })?;
                    for segment in &pattern.segments {
                        total = total.checked_add(u64::try_from(segment.len()).ok()?)?;
                    }
                    total
                }
            })?,
        Record::Core(
            CoreRecord::Deployment(_) | CoreRecord::RunProof(_) | CoreRecord::Call(_) | CoreRecord::Projection(_),
        )
        | Record::People(_)
        | Record::Notes(notes::Record::Line(_)) => return None,
    };
    carrier.checked_add(owned)
}

pub(crate) fn row_bytes(row: &Row) -> Option<u64> {
    let mut total = u64::try_from(size_of::<Row>())
        .ok()?
        .checked_add(record_bytes(&row.value)?)?
        .checked_add(match &row.owner {
            Some(owner) => tasks::task_bytes(owner)?,
            None => 0,
        })?
        .checked_add(u64::try_from(size_of_val(row.ancestors.as_ref())).ok()?)?
        .checked_add(match &row.related {
            Some(value) => record_bytes(value)?,
            None => 0,
        })?
        .checked_add(u64::try_from(size_of_val(row.linked.as_ref())).ok()?)?;
    for linked in &row.linked {
        total = total.checked_add(tasks::task_bytes(linked)?)?;
    }
    Some(total)
}
