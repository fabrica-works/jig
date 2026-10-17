//! Durable task-held acceptance evidence shared by the world's store adapters.
use super::{r, source, tasks};

#[must_use]
pub fn task(row: &tasks::History) -> Option<(skein_lib::Wall, u64, r::Acceptance)> {
    if row.change == tasks::Change::ProposalAccepted
        && let Some(proposal) = &row.proposal
        && let tasks::ProposalAction::Amend { task, .. } | tasks::ProposalAction::Widen { task, .. } = proposal.action
    {
        return Some((row.at, task, r::Acceptance { proposal: proposal.number, by: source(row.by) }));
    }
    None
}
