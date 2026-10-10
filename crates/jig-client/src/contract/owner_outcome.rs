//! Committed people request results (client/domain/contract.md, 4).
//! Keeps only the owner's supplied numbers, choices and clocks. Retry,
//! signed-out, refusal and connector answers are separate boundary routes.
use super::{ProposalKey, Revision, Seen};
use skein_lib::Wall;

/// The retained decision for a proposal, without an inferred explanation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OwnerProposalChoice {
    /// The proposal was accepted.
    Accepted,
    /// The proposal was rejected.
    Rejected,
    /// The proposal was passed to another holder.
    Passed,
    /// The proposal was withdrawn.
    Withdrawn,
    /// The proposal ceased to apply.
    Stale,
}

/// The retained decision for a held task's escalation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OwnerEscalationChoice {
    /// The task's hold was released.
    Released,
    /// The escalation was rejected and the task remains held.
    Rejected,
    /// The escalation was passed to another holder.
    Passed,
}

/// Facts returned by a committed jig request, including a historical winner.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum OwnerOutcome {
    /// A chat closed with the supplied counts of cancelled and kept delegates.
    Closing { chat: u64, cancelled: u32, kept: u32 },
    /// A results or chat-replies position was marked seen.
    MarkedSeen { seen: Seen },
    /// A note correction or deletion was committed.
    NoteEdited { name: u64 },
    /// A service party and its project role were made.
    ServiceMade { person: u64 },
    /// The owner committed a project's adoption.
    Adopted { project: u32 },
    /// A goal task was made.
    GoalStarted { task: u64 },
    /// A goal awaits a proposal decision; only the supplied number is known.
    GoalProposed { proposal: u64 },
    /// A task was stopped for a later release or decision.
    Stopped { task: u64 },
    /// A task and its descendants were cancelled.
    Cancelled { task: u64 },
    /// A task's hold was released.
    Released { task: u64 },
    /// A role-addressed task was taken.
    PersonTaken { task: u64 },
    /// The claimant handed the task back to its role.
    PersonHandedBack { task: u64 },
    /// A person's task answer was admitted.
    PersonAnswered { task: u64 },
    /// A task and its remaining funding moved to the requesting person.
    Moved { task: u64 },
    /// A proposal's current or historical decision names its maker and winner.
    ProposalDecided { proposal: ProposalKey, by: u64, at: Wall, choice: OwnerProposalChoice },
    /// Words were admitted under a message number, with the supplied release fact.
    Said { task: u64, message: u64, released: bool },
    /// A named question was answered by a message.
    QuestionAnswered { task: u64, question: u64, message: u64 },
    /// A project's goal priorities changed together.
    Prioritised { project: u32 },
    /// A task accepted an amendment, with the supplied release fact.
    Amended { task: u64, released: bool },
    /// An amendment awaits the supplied proposal number, without an inferred maker.
    AmendProposed { task: u64, proposal: u64, released: bool },
    /// A project's roster and affected waiting recipients changed together.
    RolesSet { project: u32 },
    /// A project's policy changed for later checks.
    PolicyChanged { project: u32 },
    /// A person's project pool was opened or resized.
    PoolSet { project: u32, person: u64 },
    /// A hold revision's current or historical decision names its winner.
    EscalationDecided { task: u64, revision: Revision, by: u64, at: Wall, choice: OwnerEscalationChoice },
    /// A chat task was started.
    Started { task: u64 },
}
