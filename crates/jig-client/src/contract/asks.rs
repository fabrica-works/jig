//! Each visible act is one ask; client/domain/contract.md, 3.2 and 4.
//! Requests carry jig facts alone and their outcomes are durable.
//! Adoption uses the connector's application request (client/domain/pages.md, 6.5).
use super::{Actor, Authority, Decision, HeldRole, NoteScope, Part, Party, ProposalKey, Refusal, Revision, Text};
use alloc::boxed::Box;
use skein_lib::{List, Wall};
/// Executors the engine supports for a tracked goal; domain/people.md, 5.1.
/// Numeric identities stay whole; procedure names and parameters are not supplied.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GoalExecutor {
    Charter(u32),
    Procedure { connector: u16, code: u32 },
}

/// A person destination change names the independent person-task revision (domain/tasks.md, 7).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AddressChange {
    pub revision: Revision,
    pub address: super::PersonAddress,
}

/// A task change whose spec revision the request names.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Amendment {
    /// Retarget a person task, with admission decided by its engine document.
    pub addressed: Option<AddressChange>,
    pub spec: Option<super::Specification>,
    pub wake: Option<super::OwnerWakePolicy>,
    pub dependencies: Option<List<u64>>,
    pub authority: Option<super::OwnerAuthority>,
    pub instructions: Option<Text>,
}

/// The kinds of news that wake an agent.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct WakePolicy {
    pub words: bool,
    pub results: bool,
    pub topics: List<u16>,
    pub subscriptions: List<Part>,
}

/// A person task answer matching its contract.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PersonAnswer {
    /// An option and reason.
    Choice { option: u32, reason: Text },
    /// An approval naming the state seen.
    Approved { state: Box<[u8]>, words: Option<Text> },
    /// An answer in words.
    Words(Text),
    /// A failure with a reason.
    Cannot { reason: Text },
}

/// What becomes of one live chat delegate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Disposal {
    /// Cancel its subtree.
    Cancel { task: u64 },
    /// Make it the party's and move funding.
    TakeOver { task: u64 },
}

/// A goal and its new priority.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Priority {
    pub task: u64,
    pub priority: u32,
}

/// The read position moved by an explicit act.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Seen {
    /// Results seen through this position.
    Results { through: u64 },
    /// One chat replies seen through this position.
    Replies { chat: u64, through: u64 },
}

/// A party and its project role.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Holding {
    pub party: u64,
    pub role: HeldRole,
}

/// The project policy as facts.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Policy {
    pub ceiling: Authority,
    pub period: u64,
    pub spend: u64,
    pub roles: List<RolePolicy>,
    pub requirements: List<Part>,
    pub deciders: List<Decider>,
}

/// What a project role may give, allot and decide.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RolePolicy {
    pub number: u16,
    pub authority: Authority,
    pub allotment: u64,
    pub decisions: List<u16>,
}

/// The policy role for a kind of proposal past a tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Decider {
    pub kind: u16,
    pub role: u16,
}

/// Every jig request, without keys owned by the root table.
#[expect(
    clippy::large_enum_variant,
    reason = "whole amendments remain inline; the keyed payload envelope prices the largest variant"
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Ask {
    /// Start a chat.
    StartChat { project: u32, words: Text },
    /// Write words and optionally release a stopped chat.
    Write { project: u32, task: u64, words: Text, release: bool },
    /// Answer a named question.
    AnswerQuestion { project: u32, task: u64, message: u64, words: Text },
    /// Answer the person task revision seen.
    AnswerPerson { project: u32, task: u64, revision: Revision, answer: PersonAnswer },
    /// Take a role-addressed person task.
    Take { project: u32, task: u64 },
    /// Return a person task to its role.
    HandBack { project: u32, task: u64 },
    /// Decide an immutable proposal.
    DecideProposal { project: u32, proposal: ProposalKey, decision: Decision },
    /// Decide the hold revision seen.
    DecideEscalation { project: u32, task: u64, revision: Revision, decision: Decision },
    /// Release a hold, choosing whether to repeat an uncertain effect.
    Release { project: u32, task: u64, revision: Revision, again: Option<bool> },
    /// Amend and optionally release in one decision.
    Amend { project: u32, task: u64, revision: Revision, change: Amendment, release: Option<Revision> },
    /// Stop a run and hold its task.
    Stop { project: u32, task: u64 },
    /// Cancel a task and its subtree.
    Cancel { project: u32, task: u64, reason: Text },
    /// Close a chat after disposing of every live delegate.
    Close { project: u32, chat: u64, disposals: List<Disposal> },
    /// Become a task requester and fund it.
    TakeOver { project: u32, task: u64, reason: Text },
    /// Make or propose a goal.
    SetGoal { project: u32, spec: Text, executor: GoalExecutor, budget: u64, priority: u32, deadline: Option<Wall> },
    /// Change goal priorities together.
    Prioritise { project: u32, goals: List<Priority> },
    /// Move a read position explicitly.
    MarkSeen { seen: Seen },
    /// Correct the note revision seen.
    EditNote {
        project: u32,
        note: u64,
        scope: NoteScope,
        revision: Revision,
        description: Text,
        body: Text,
        references: List<u64>,
    },
    /// Delete the note revision seen.
    DeleteNote { project: u32, note: u64, scope: NoteScope, revision: Revision },
    /// Change a project policy.
    ChangePolicy { project: u32, edit: super::PolicyEdit },
    /// Replace project roles.
    SetRoles { project: u32, holdings: List<Holding> },
    /// Make a service with a project role.
    MakeService { project: u32, name: Text, role: HeldRole },
    /// Set a party pool preserving spend and reservations.
    SetPool { project: u32, party: u64, budget: u64 },
}

/// The durable result of one ask.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Outcome {
    /// Whole committed owner facts, without inferred names or explanations.
    Owner(super::OwnerOutcome),
    /// A named thing was made.
    Made { what: Made },
    /// The act became a proposal.
    Proposed { proposal: ProposalKey },
    /// A decision was made.
    Decided { by: Actor, how: Decision, at: Wall },
    /// Another party decided first.
    DecidedFirst { by: Party, how: Decision, at: Wall },
    /// The requested change was made.
    Changed,
}

/// What an ask made, named for its client.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Made {
    /// A chat task.
    Chat { task: u64 },
    /// Words in the task inbox.
    Words { task: u64, message: u64 },
    /// A goal task.
    Goal { task: u64 },
    /// Tasks accepted from a batch.
    Tasks { tasks: List<u64> },
    /// A service party.
    Service { party: u64 },
    /// A person task result.
    PersonResult { task: u64 },
    /// A note in its scope.
    Note { note: u64, scope: NoteScope },
}

/// The one terminal the request table hands to this domain.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Answer {
    /// A durable outcome.
    Done(Outcome),
    /// A durable refusal.
    Refused(Refusal),
}
