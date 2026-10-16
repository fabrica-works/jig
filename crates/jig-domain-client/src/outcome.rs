//! Routing facts from retained outcomes (client/domain/contract.md, 4).
//! Classification never rewrites the whole durable answer or joins missing facts.
use jig_client_domain::contract::{Made, Outcome, OwnerOutcome};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Route {
    Chat { task: u64 },
    Goal { task: u64 },
    Words { task: u64, message: u64 },
    Proposed,
    Other,
}
pub(crate) fn route(outcome: &Outcome) -> Route {
    match outcome {
        Outcome::Owner(value) => match value {
            OwnerOutcome::Started { task } => Route::Chat { task: *task },
            OwnerOutcome::GoalStarted { task } => Route::Goal { task: *task },
            OwnerOutcome::Said { task, message, .. } => Route::Words { task: *task, message: *message },
            OwnerOutcome::GoalProposed { .. } | OwnerOutcome::AmendProposed { .. } => Route::Proposed,
            OwnerOutcome::Closing { .. }
            | OwnerOutcome::MarkedSeen { .. }
            | OwnerOutcome::NoteEdited { .. }
            | OwnerOutcome::ServiceMade { .. }
            | OwnerOutcome::Adopted { .. }
            | OwnerOutcome::Stopped { .. }
            | OwnerOutcome::Cancelled { .. }
            | OwnerOutcome::Released { .. }
            | OwnerOutcome::PersonTaken { .. }
            | OwnerOutcome::PersonHandedBack { .. }
            | OwnerOutcome::PersonAnswered { .. }
            | OwnerOutcome::Moved { .. }
            | OwnerOutcome::ProposalDecided { .. }
            | OwnerOutcome::QuestionAnswered { .. }
            | OwnerOutcome::Prioritised { .. }
            | OwnerOutcome::Amended { .. }
            | OwnerOutcome::RolesSet { .. }
            | OwnerOutcome::PolicyChanged { .. }
            | OwnerOutcome::PoolSet { .. }
            | OwnerOutcome::EscalationDecided { .. } => Route::Other,
        },
        Outcome::Made { what } => match what {
            Made::Chat { task } => Route::Chat { task: *task },
            Made::Goal { task } => Route::Goal { task: *task },
            Made::Words { task, message } => Route::Words { task: *task, message: *message },
            Made::Tasks { .. } | Made::Service { .. } | Made::PersonResult { .. } | Made::Note { .. } => Route::Other,
        },
        Outcome::Proposed { .. } => Route::Proposed,
        Outcome::Decided { .. } | Outcome::DecidedFirst { .. } | Outcome::Changed => Route::Other,
    }
}
