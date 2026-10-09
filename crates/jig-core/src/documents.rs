//! Typed live documents, made on demand from the core's committed vocabulary
//! (`client/domain/contract.md`, sections 5 and 7; `domain/engine.md`, 11).
//! No document is retained here, and this module neither delivers nor decides.
//! `make` authenticates visibility from a supplied party number and copies only
//! bounded text and lists. Its caller holds the answer until durable.
//!
//! Frame counts describe the people's bounded caches; cross-project exact needs
//! and read positions belong to the inbox work. Object acts belong to the acts
//! work. Activity positions belong to the activity work. Run writing status,
//! provisional text and calls belong to the watch stream; committed turns never
//! enter its text. Application subjects belong to their connectors.
//!
//! Store-only notes, ended rows and proposal routing trails use the public row
//! constructors. Notes lack changed wall times; history and milestones lack
//! wall times and amendment before snapshots until that work owns their records.
//! Person stall timing and result followup identities are not retained today;
//! their optional fields say so. Dependency results missing from live memory are
//! stored references for the reader. Effect pricing belongs to its connector;
//! a proposal's effect authority ceiling is never reported as its price.
//! Recurring wall schedules and past-period/root spending need store/period
//! owners; the project reports period numbers and current live-root spending.
//! Inbox result references may lack project and time, and reply read positions
//! are absent until inbox work. Lost-host identity and hello refusal history
//! are not retained; deployment preserves its adrift attempts instead. Use
//! `make_with_clock` for wall-time grace and account cooldowns; `make` leaves
//! those optional time fields absent when no clock was supplied.

mod inbox_deployment;
pub use inbox_deployment::{
    Account, Capacity, ConversationLine, Conversations, Deployment, DocumentClock, Host, HostRun, Inbox, InboxEntry,
    InboxObject, ProjectCapacity, RunLocation, make_with_clock,
};
mod objects;
mod project_note;
mod small;
pub use objects::{
    AmendmentChange, AmendmentState, EndedDelegate, Escalation, FundingOffer, HistoryLine, MilestoneLine,
    MilestonePhase, PersonTask, Proposal, ProposalAction, ProposalHistory, ProposalHistoryState, ProposalWaiting,
    ProposedTask, Question, ResultDoc, RetainedAction, ShownResult, SpecLine, ended_delegate_from_row,
    history_from_row, milestone_from_row, result_from_row, retained_action,
};
pub use project_note::{
    Ledger, Member, Note, NoteLocation, Policy, PolicyRole, Project, ProjectResource, Recurring, Requirement,
    ResourceRole, SpendLine, note_from_entry,
};
mod task_tree;
pub use task_tree::{
    AuthorityLine, BudgetLine, ContractLine, Ending, EscalationState, FoldedCount, GoalLine, Goals, GrantLine,
    HoldLine, NoteScope, Parameter, Part, PartKey, PartKind, PatternEnd, PhaseCounts, PriorPhase, ProposalRef,
    ResourceLine, ResultValue, Task, TaskLink, Tree, TreeRow,
};

use crate::{Core, Limits};
use alloc::boxed::Box;
use jig_core_people as people;
use jig_core_tasks as tasks;
use skein_lib::{Duration, List, Wall};

/// One bounded opaque text value, with loss stated separately.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Text {
    pub bytes: Box<[u8]>,
    pub cut: bool,
}

/// Versions of jig's contract served by this configured deployment.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Versions {
    pub minimum: u32,
    pub maximum: u32,
}

/// Bounds of one constructed document and its configured served versions.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct DocumentLimits {
    pub rows: u32,
    pub text_bytes: u32,
    pub chips: u32,
    pub lines: u32,
    pub versions: Versions,
}

/// Continuation position in a subject's stable numbered list.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Cursor {
    /// A task or goal number in the subject's ordered list.
    Number { number: u64 },
    /// Exact identity of one inbox row; number families may overlap.
    Inbox { task: u64, object: InboxObject, at: Option<Wall> },
}

/// The bounded slice requested by a watch or read.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Window {
    pub rows: u32,
    pub after: Option<Cursor>,
}

/// Identity of the immutable proposal's maker.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Proposer {
    /// A task's proposal.
    Task(u64),
    /// A party's proposed goal.
    Party(u64),
}

/// One subject whose current document a party may read or watch.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Subject {
    /// The authenticated party's frame.
    Frame { party: u64 },
    /// A task's overview and controls.
    Task { task: u64 },
    /// A task's live requester subtree.
    Tree { task: u64 },
    /// Each named task's compact live status.
    Chips { tasks: List<u64> },
    /// The party's waiting objects and unread updates.
    Inbox { party: u64 },
    /// A project's live goals.
    Goals { project: u32 },
    /// The party's live chats.
    Conversations { party: u64 },
    /// A task's current attempt.
    Run { task: u64 },
    /// Hosts, accounts and capacity.
    Deployment,
    /// An immutable pending proposal.
    Proposal { proposer: Proposer, number: u64 },
    /// A held task's escalation.
    Escalation { task: u64 },
    /// An unanswered question.
    Question { task: u64, message: u64 },
    /// A task addressed to a person or role.
    PersonTask { task: u64 },
    /// A result still named by the live engine.
    Result { task: u64 },
    /// A project's resources, policy and funding.
    Project { project: u32 },
    /// A note retained by its owning child.
    Note { note: u64 },
}

/// Why no live document is available.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Missing {
    /// No visible subject is known by this name.
    Unknown,
    /// The supplied party may not see this subject.
    NotAllowed,
    /// The subject belongs to stored history.
    Ended,
    /// The requested whole value exceeds construction bounds.
    TooLarge,
}

/// A bounded count, with saturation stated.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Count {
    /// Every currently known entry was counted.
    Known(u32),
    /// The bounded cache may omit older entries.
    AtLeast(u32),
}

/// A task's executor kind; its detail lives on the task document.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ExecutorKind {
    /// An agent with a configured charter.
    Agent,
    /// A connector-owned procedure.
    Procedure,
    /// A party or project role.
    Person,
}

/// A compact typed phase and its immediate waiting facts.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum PhaseLine {
    /// Waiting for live dependencies or resource holdings.
    Waiting { dependencies: List<u64>, more: bool, holds: bool },
    /// Current activation state of an agent or procedure.
    Active(tasks::Active),
    /// Waiting for the addressed party or role.
    Person { address: tasks::PersonAddress, taken_by: Option<u64>, since: Wall },
    /// Closing through its ordered obligations.
    Closing(tasks::Stage),
    /// Held for this reason at this independent revision.
    Held { reason: tasks::Hold, revision: u64 },
    /// Ended outside the live arena.
    Ended,
}

/// A task named in passing, with its bounded first-line title.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Chip {
    pub number: u64,
    pub title: Text,
    pub executor: ExecutorKind,
    pub phase: PhaseLine,
}

/// A project's charter numbers offered for chats and goals.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct OfferedCharter {
    pub project: u32,
    pub chat: u32,
    pub goals: List<u32>,
    pub cut: bool,
}

/// Party identity, current roles, bounded counts and contract configuration.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Frame {
    pub party: u64,
    pub kind: people::Kind,
    pub login: Text,
    pub name: Text,
    pub projects: List<people::ProjectRole>,
    pub needs: u32,
    pub updates: Count,
    pub retention: Duration,
    pub charters: List<OfferedCharter>,
    pub versions: Versions,
    pub cut: bool,
}

/// A live attempt's committed progress; provisional writing is supplied by watches.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Run {
    pub task: u64,
    pub attempt: u64,
    pub committed_turn: u32,
    pub writing: Option<bool>,
}

/// One whole typed document, within its configured limits.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Document {
    /// The party's frame.
    Frame(Frame),
    /// Compact status of each named task.
    Chips(List<Chip>),
    /// Current attempt and committed turn.
    Run(Run),
    /// One task page's current overview.
    Task(Box<Task>),
    /// A bounded live subtree with folds.
    Tree(Tree),
    /// Live goals in priority order.
    Goals(Goals),
    /// A pending immutable proposal.
    Proposal(Box<Proposal>),
    /// A held task and its preserved story.
    Escalation(Escalation),
    /// Whole unanswered question text.
    Question(Question),
    /// A current person task and its form.
    PersonTask(Box<PersonTask>),
    /// An outcome retained in a live closing row.
    Result(Box<ResultDoc>),
    /// Project resources, roles, policy and ledgers.
    Project(Box<Project>),
    /// A note loaded by its owning child.
    Note(Note),
    /// Party-only bounded waiting and unread groups.
    Inbox(Inbox),
    /// Party-only directly requested live chats.
    Conversations(Conversations),
    /// Fleet placement, accounts and finite capacities.
    Deployment(Box<Deployment>),
}

/// Make the current visible document without changing state or emitting effects.
pub fn make(core: &Core, limits: &Limits, party: u64, subject: &Subject, window: &Window) -> Result<Document, Missing> {
    if document_worst_case(&limits.documents).is_none() || window.rows > limits.documents.rows {
        return Err(Missing::TooLarge);
    }
    core.people.identity(party).ok_or(Missing::Unknown)?;
    match subject {
        Subject::Frame { party: owner } => {
            if *owner != party {
                return Err(Missing::NotAllowed);
            }
            let frame = small::frame(core, limits, party)?;
            Ok(Document::Frame(frame))
        }
        Subject::Chips { tasks: named } => {
            if named.len() > limits.documents.chips {
                return Err(Missing::TooLarge);
            }
            let mut chips = List::with_capacity(named.len());
            for number in named {
                chips.push(small::chip(core, limits, party, *number)?).expect("named chips fit");
            }
            Ok(Document::Chips(chips))
        }
        Subject::Run { task } => {
            let run = small::run(core, party, *task)?;
            Ok(Document::Run(run))
        }
        Subject::Task { task } => {
            let task = task_tree::task(core, limits, party, *task)?;
            Ok(Document::Task(Box::new(task)))
        }
        Subject::Tree { task } => {
            let tree = task_tree::tree(core, limits, party, *task, window)?;
            Ok(Document::Tree(tree))
        }
        Subject::Goals { project } => {
            let goals = task_tree::goals(core, limits, party, *project, window)?;
            Ok(Document::Goals(goals))
        }
        Subject::Inbox { party: owner } => {
            if *owner != party {
                return Err(Missing::NotAllowed);
            }
            Ok(Document::Inbox(inbox_deployment::inbox(core, limits, party, window)?))
        }
        Subject::Conversations { party: owner } => {
            if *owner != party {
                return Err(Missing::NotAllowed);
            }
            Ok(Document::Conversations(inbox_deployment::conversations(core, limits, party, window)?))
        }
        Subject::Deployment => {
            Ok(Document::Deployment(Box::new(inbox_deployment::deployment(core, limits, party, None)?)))
        }
        Subject::Proposal { proposer, number } => {
            Ok(Document::Proposal(Box::new(objects::proposal(core, limits, party, *proposer, *number)?)))
        }
        Subject::Escalation { task } => Ok(Document::Escalation(objects::escalation(core, limits, party, *task)?)),
        Subject::Question { task, message } => {
            Ok(Document::Question(objects::question(core, limits, party, *task, *message)?))
        }
        Subject::PersonTask { task } => {
            Ok(Document::PersonTask(Box::new(objects::person(core, limits, party, *task)?)))
        }
        Subject::Result { task } => Ok(Document::Result(Box::new(objects::result(core, limits, party, *task)?))),
        Subject::Project { project } => {
            Ok(Document::Project(Box::new(project_note::project(core, limits, party, *project)?)))
        }
        Subject::Note { .. } => Err(Missing::Ended),
    }
}

/// Checked allocation bound for one document; parents price each retained delivery separately.
#[must_use]
pub fn document_worst_case(limits: &DocumentLimits) -> Option<u64> {
    if limits.rows == 0
        || limits.text_bytes == 0
        || limits.chips == 0
        || limits.lines == 0
        || limits.versions.minimum == 0
        || limits.versions.minimum > limits.versions.maximum
    {
        return None;
    }
    let chip = u64::from(limits.text_bytes).checked_add(List::<u64>::worst_case(limits.lines)?)?;
    let chips = List::<Chip>::worst_case(limits.chips)?.checked_add(u64::from(limits.chips).checked_mul(chip)?)?;
    let charter = List::<u32>::worst_case(limits.lines)?;
    let frame = List::<people::ProjectRole>::worst_case(limits.lines)?
        .checked_add(List::<OfferedCharter>::worst_case(limits.lines)?)?
        .checked_add(u64::from(limits.lines).checked_mul(charter)?)?
        .checked_add(u64::from(limits.text_bytes).checked_mul(2)?)?;
    u64::try_from(size_of::<Document>()).ok()?.checked_add(
        chips
            .max(frame)
            .max(task_tree::worst_case(limits)?)
            .max(objects::worst_case(limits)?)
            .max(project_note::worst_case(limits)?)
            .max(inbox_deployment::worst_case(limits)?),
    )
}

/// Peak scratch allocated while constructing party/deployment pages. Child
/// accessors copy bounded shallow facts only; documents themselves are separate.
pub(crate) fn document_scratch_worst_case(limits: &Limits) -> Option<u64> {
    inbox_deployment::scratch_worst_case(limits)
}
