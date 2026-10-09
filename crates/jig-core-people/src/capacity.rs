//! Content-free room-refusal facts for deployment capacity
//! (client/domain/pages.md, section 7; domain/people.md, section 5.1).
//! Permanent facts come from retained keyed answers; entrance pressure is
//! bounded live history, independent of diagnostics and replay decisions.

use crate::{Ask, LimitKind, Outcome, Refusal, RequestKey};
use skein_lib::Wall;

/// The refused operation's kind and existing object identity, without its payload.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum CapacityAction {
    Application { connector: u16 },
    Note { name: u64 },
    Service,
    Adoption,
    Roles,
    Policy,
    Pool { person: u64 },
    Chat,
    Goal,
    Escalation { task: u64 },
    Proposal { proposer: u64, number: u64 },
    Words { task: u64 },
    Question { task: u64, question: u64 },
    Priority,
    Amend { task: u64 },
    Move { task: u64 },
    TakePerson { task: u64 },
    HandBackPerson { task: u64 },
    AnswerPerson { task: u64 },
    Stop { task: u64 },
    Cancel { task: u64 },
    Release { task: u64 },
    MarkSeen { seen: crate::Seen },
}

/// Exact entrance pressure, or a retained permanent room-limit answer.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum CapacityCause {
    Requests { limit: u32 },
    Pending { limit: u32 },
    Waiters { limit: u32 },
    Limit { which: LimitKind },
}

/// One refusal at its actual decision wall time. The opaque request key identifies
/// the request; no words, provider identity, credential or reply right are copied.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CapacityRefusal {
    pub key: RequestKey,
    pub project: Option<u32>,
    pub action: CapacityAction,
    pub at: Wall,
    pub cause: CapacityCause,
}

pub(crate) fn action(ask: &Ask) -> CapacityAction {
    match ask {
        Ask::Application { connector, .. } => CapacityAction::Application { connector: *connector },
        Ask::EditNote { name, .. } => CapacityAction::Note { name: *name },
        Ask::MakeService { .. } => CapacityAction::Service,
        Ask::Adopt { .. } => CapacityAction::Adoption,
        Ask::SetRoles { .. } => CapacityAction::Roles,
        Ask::ChangePolicy { .. } => CapacityAction::Policy,
        Ask::SetPool { person, .. } => CapacityAction::Pool { person: *person },
        Ask::StartChat { .. } => CapacityAction::Chat,
        Ask::SetGoal { .. } => CapacityAction::Goal,
        Ask::DecideEscalation { task, .. } => CapacityAction::Escalation { task: *task },
        Ask::DecideProposal { proposer, proposal, .. } => {
            CapacityAction::Proposal { proposer: *proposer, number: *proposal }
        }
        Ask::Say { task, .. } => CapacityAction::Words { task: *task },
        Ask::AnswerQuestion { task, question, .. } => CapacityAction::Question { task: *task, question: *question },
        Ask::Prioritise { .. } => CapacityAction::Priority,
        Ask::Amend { task, .. } => CapacityAction::Amend { task: *task },
        Ask::Move { task, .. } => CapacityAction::Move { task: *task },
        Ask::TakePerson { task, .. } => CapacityAction::TakePerson { task: *task },
        Ask::HandBackPerson { task, .. } => CapacityAction::HandBackPerson { task: *task },
        Ask::AnswerPerson { task, .. } => CapacityAction::AnswerPerson { task: *task },
        Ask::Stop { task, .. } => CapacityAction::Stop { task: *task },
        Ask::Cancel { task, .. } => CapacityAction::Cancel { task: *task },
        Ask::Release { task, .. } => CapacityAction::Release { task: *task },
        Ask::MarkSeen { seen } => CapacityAction::MarkSeen { seen: *seen },
    }
}

pub(crate) fn project(ask: &Ask) -> Option<u32> {
    match ask {
        Ask::Application { project, .. }
        | Ask::EditNote { project, .. }
        | Ask::MakeService { project, .. }
        | Ask::Adopt { project, .. }
        | Ask::SetRoles { project, .. }
        | Ask::ChangePolicy { project, .. }
        | Ask::SetPool { project, .. }
        | Ask::StartChat { project, .. }
        | Ask::DecideEscalation { project, .. }
        | Ask::DecideProposal { project, .. }
        | Ask::Say { project, .. }
        | Ask::AnswerQuestion { project, .. }
        | Ask::Prioritise { project, .. }
        | Ask::Amend { project, .. }
        | Ask::Move { project, .. }
        | Ask::TakePerson { project, .. }
        | Ask::HandBackPerson { project, .. }
        | Ask::AnswerPerson { project, .. }
        | Ask::Stop { project, .. }
        | Ask::Cancel { project, .. }
        | Ask::Release { project, .. }
        | Ask::SetGoal { project, .. } => Some(*project),
        Ask::MarkSeen { .. } => None,
    }
}

pub(crate) fn permanent(outcome: Outcome) -> Option<CapacityCause> {
    match outcome {
        Outcome::Refused(refusal) => room(refusal),
        Outcome::Applied { .. }
        | Outcome::Again(_)
        | Outcome::SignedOut
        | Outcome::NoteEdited { .. }
        | Outcome::ServiceMade { .. }
        | Outcome::Adopted { .. }
        | Outcome::GoalStarted { .. }
        | Outcome::GoalProposed { .. }
        | Outcome::Stopped { .. }
        | Outcome::Cancelled { .. }
        | Outcome::Released { .. }
        | Outcome::PersonTaken { .. }
        | Outcome::PersonHandedBack { .. }
        | Outcome::PersonAnswered { .. }
        | Outcome::Moved { .. }
        | Outcome::ProposalDecided { .. }
        | Outcome::Said { .. }
        | Outcome::QuestionAnswered { .. }
        | Outcome::Prioritised { .. }
        | Outcome::Amended { .. }
        | Outcome::AmendProposed { .. }
        | Outcome::RolesSet { .. }
        | Outcome::PolicyChanged { .. }
        | Outcome::PoolSet { .. }
        | Outcome::EscalationDecided { .. }
        | Outcome::Started { .. }
        | Outcome::MarkedSeen { .. } => None,
    }
}

fn room(refusal: Refusal) -> Option<CapacityCause> {
    match refusal {
        Refusal::Limit { which } => match which {
            LimitKind::ProjectLiveTasks
            | LimitKind::LiveTasks
            | LimitKind::Tree
            | LimitKind::Delegates
            | LimitKind::LiveDelegates
            | LimitKind::Holds
            | LimitKind::InboxFull
            | LimitKind::Notes => Some(CapacityCause::Limit { which }),
            LimitKind::Words
            | LimitKind::Identity
            | LimitKind::Roles
            | LimitKind::Arithmetic
            | LimitKind::Duplicate
            | LimitKind::Empty
            | LimitKind::Batch
            | LimitKind::Depth
            | LimitKind::Subscription
            | LimitKind::Dependencies
            | LimitKind::Cycle
            | LimitKind::Spec
            | LimitKind::Contract
            | LimitKind::Authority
            | LimitKind::Inputs
            | LimitKind::Attempts
            | LimitKind::Restore
            | LimitKind::Read
            | LimitKind::Turn
            | LimitKind::HoldKind
            | LimitKind::HoldTaken
            | LimitKind::Revision
            | LimitKind::Question
            | LimitKind::Priority
            | LimitKind::Policy
            | LimitKind::Resource
            | LimitKind::Service => None,
        },
        Refusal::Role { .. }
        | Refusal::Authority { .. }
        | Refusal::Moved { .. }
        | Refusal::Taken { .. }
        | Refusal::Ended { .. }
        | Refusal::NotWaiting
        | Refusal::NoFurther
        | Refusal::Unknown
        | Refusal::NotOffered
        | Refusal::KeyConflict => None,
    }
}
