//! Owned facts, identities and bounded text shared within the client vocabulary.
//! No engine or application type crosses here. Text is checked before ownership
//! moves in; list owners choose capacities from limits. See client/domain/contract.md, 5.
use alloc::boxed::Box;
use skein_lib::{List, Wall};
/// Owned text or a path, checked by its owner before it is kept.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Text {
    bytes: Box<[u8]>,
}
impl Text {
    /// Takes bytes only when their length fits the owner's bound.
    pub fn new(bytes: Box<[u8]>, limit: u32) -> Result<Self, Box<[u8]>> {
        if bytes.len() > usize::try_from(limit).expect("u32 fits usize") {
            return Err(bytes);
        }
        Ok(Self { bytes })
    }
    /// The opaque words, never parsed by the domain.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    /// Whether the already decoded text fits another owner's byte bound.
    #[must_use]
    pub fn fits(&self, limit: u32) -> bool {
        self.bytes.len() <= usize::try_from(limit).expect("u32 fits usize")
    }
    /// Moves the bytes to their next owner.
    #[must_use]
    pub fn into_bytes(self) -> Box<[u8]> {
        self.bytes
    }
}
/// The version of one independently decidable thing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Revision {
    pub value: u64,
}

/// A proposal named by its proposer and local number.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProposalKey {
    pub proposer: u64,
    pub number: u64,
}

/// One object in the working set.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ObjectKey {
    /// A task.
    Task(u64),
    /// A proposal.
    Proposal(ProposalKey),
    /// A held task.
    Escalation(u64),
    /// A question in a task inbox.
    Question { task: u64, message: u64 },
    /// A task carried out by a party.
    PersonTask(u64),
    /// A task result.
    Result(u64),
    /// A note in its project's page.
    Note { project: u32, note: u64 },
}

/// Whether a party has a person at its client.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PartyKind {
    /// A person.
    Person,
    /// A service with no view.
    Service,
}

/// Who acted, as display facts.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Party {
    pub number: u64,
    pub kind: PartyKind,
    pub name: Text,
}

/// Who made a recorded change.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Actor {
    /// A party.
    Party(u64),
    /// A task.
    Task(u64),
    /// The deployment.
    Deployment,
    /// A deployment decision scoped to this project.
    ProjectDeployment { project: u32 },
}

/// The kind of executor carrying out a task.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Executor {
    /// An agent and its charter.
    Agent { charter: u32 },
    /// A connector procedure.
    Procedure { connector: u16, kind: u32 },
    /// A named party.
    Party { party: u64 },
    /// Any party holding a role.
    Role { project: u32, role: u32 },
}

/// The task phase, without words from the view.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Phase {
    /// Waiting on dependencies or holds.
    Waiting,
    /// Active and waiting for news.
    Idle,
    /// Due to run.
    Due,
    /// Preparing a host.
    Preparing,
    /// Claimed by a host.
    Claimed,
    /// A run is live.
    Running,
    /// Waiting to retry.
    BackingOff,
    /// An executor is active.
    Active,
    /// Closing its children and effects.
    Closing,
    /// Held for a decision.
    Held,
    /// Ended successfully.
    Done,
    /// Ended with failure.
    Failed,
    /// Cancelled.
    Cancelled,
}

/// An application value named without carrying its type.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Part {
    /// The connector name, supplied by its own view words.
    Connector { connector: u16 },
    /// A resource in its connector.
    Resource { connector: u16, path: Text },
    /// An effect or read kind.
    Kind { connector: u16, kind: u16 },
    /// A task panel.
    Panel { connector: u16, task: u64 },
    /// A connector result.
    Result { connector: u16, task: u64 },
    /// A proposed effect.
    Effect { connector: u16, proposal: ProposalKey },
    /// An actual outbox entry in its supplied owner scope, independent of an effect proposal.
    Outbox { connector: u16, scope: PartScope, entry: u64 },
    /// The connector facts about a hold.
    Hold { connector: u16, task: u64 },
    /// A connector project summary.
    Project { connector: u16, project: u32 },
    /// A connector deployment summary.
    Deployment { connector: u16 },
}

/// Permission for a numbered kind on a resource pattern.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Grant {
    pub connector: u16,
    pub kind: u16,
    pub pattern: Text,
}

/// Authority facts to show or ask to amend, never evaluated here.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Authority {
    pub budget: u64,
    pub grants: List<Grant>,
    pub task_kinds: List<Executor>,
    pub depth: u32,
    pub tasks: u32,
    pub deadline: Option<Wall>,
    pub tools: List<u16>,
    pub note_scopes: List<NoteScope>,
}

/// The scope of a note.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum NoteScope {
    /// Deployment wide.
    Deployment,
    /// In a project.
    Project(u32),
    /// On a resource pattern.
    Resource { project: u32, connector: u16, pattern: Text },
    /// For one goal.
    Goal(u64),
}

/// How an object ended.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Ending {
    /// Completed.
    Done,
    /// Failed with a reason.
    Failed { reason: Option<Text> },
    /// Cancelled by its actor.
    Cancelled { by: Actor, reason: Option<Text>, at: Option<Wall> },
    /// Withdrawn by its proposer.
    Withdrawn { reason: Text },
    /// Decided by a party.
    Decided { by: u64, party: Option<Party>, how: Option<Decision>, at: Wall },
    /// A corrected or deleted object is no longer present.
    Deleted,
}

/// The decision made about a proposal or escalation.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Decision {
    /// Accepted.
    Accepted,
    /// Rejected with a reason.
    Rejected { reason: Text },
    /// Passed to a holder above.
    PassedUp,
    /// Released, with a choice for an uncertain effect.
    Released { again: Option<bool> },
    /// Left held with a reason.
    LeftHeld { reason: Text },
}

/// A turn named independently of its text offset.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TurnName {
    pub attempt: u32,
    pub number: u32,
}

/// A count whose upper bound may have been reached.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Count {
    pub value: u32,
    pub limited: bool,
}

/// The owner-supplied scope of a part's application document (contract.md, 7.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PartScope {
    Project(u32),
    Deployment,
}
