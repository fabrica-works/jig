//! Routing facts from retained outcomes (client/domain/contract.md, 4).
//! Classification never rewrites the whole durable answer or joins missing facts.
use jig_client::contract::{Made, Outcome, OwnerOutcome};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Route {
    Created { task: u64 },
    Chat { task: u64 },
    Goal { task: u64 },
    Words { task: u64, message: u64 },
    Proposed,
    Other,
}
pub(crate) fn route(outcome: &Outcome) -> Route {
    match outcome {
        Outcome::Owner(value) => match value {
            OwnerOutcome::Created { first, last, .. } => {
                if first == last {
                    Route::Created { task: *first }
                } else {
                    Route::Other
                }
            }
            OwnerOutcome::Started { task } => Route::Chat { task: *task },
            OwnerOutcome::GoalStarted { task } => Route::Goal { task: *task },
            OwnerOutcome::Said { task, message, .. } => Route::Words { task: *task, message: *message },
            OwnerOutcome::GoalProposed { .. } | OwnerOutcome::Proposed { .. } => Route::Proposed,
            OwnerOutcome::Withdrawn { .. }
            | OwnerOutcome::Compound { .. }
            | OwnerOutcome::Closing { .. }
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
            Made::Tasks { tasks } => {
                if tasks.len() == 1 {
                    Route::Created { task: *tasks.get(0).expect("one created task") }
                } else {
                    Route::Other
                }
            }
            Made::Service { .. } | Made::PersonResult { .. } | Made::Note { .. } => Route::Other,
        },
        Outcome::Proposed { .. } => Route::Proposed,
        Outcome::Decided { .. } | Outcome::DecidedFirst { .. } | Outcome::Changed => Route::Other,
    }
}
