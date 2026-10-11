//! Whole historical facts admission (client/domain/contract.md, 6).
//! Lists retain reserved capacity, full boxes are priced independently and no
//! historical source is shortened to another current document projection.
use crate::{Bounds, payload::Payload};
use core::mem::size_of;
use jig_client::contract::{
    OwnerEscalationDecision, OwnerEscalationDecisionRecord, OwnerGoalHolding, OwnerHistorical, OwnerHistoricalValue,
    OwnerPersonProposal, OwnerPersonProposalGoal, OwnerPersonProposalState, OwnerProposalDecision,
};
impl Payload for OwnerGoalHolding {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Write { path, .. } | Self::Slot { path, .. } => path.heap(bounds),
        }
    }
}
impl Payload for OwnerPersonProposalGoal {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        self.spec
            .heap(bounds)?
            .checked_add(self.contract.heap(bounds)?)?
            .checked_add(self.authority.heap(bounds)?)?
            .checked_add(self.dependencies.heap(bounds)?)?
            .checked_add(self.holdings.heap(bounds)?)
    }
}
impl Payload for OwnerPersonProposalState {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Pending { .. } | Self::Accepted { .. } => Some(0),
            Self::Rejected { reason, .. } => reason.heap(bounds),
        }
    }
}
impl Payload for OwnerPersonProposal {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        self.goal.heap(bounds)?.checked_add(self.state.heap(bounds)?)
    }
}
impl Payload for OwnerProposalDecision {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        Some(0)
    }
}
impl Payload for OwnerEscalationDecision {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Release | Self::Pass => Some(0),
            Self::Reject { reason } => reason.heap(bounds),
        }
    }
}
impl Payload for OwnerEscalationDecisionRecord {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        self.decision.heap(bounds)
    }
}
impl Payload for OwnerHistoricalValue {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Task { document, result } => document.heap(bounds)?.checked_add(result.heap(bounds)?),
            Self::Proposal(value) => value.heap(bounds),
            Self::PersonProposal { proposal, decision } => proposal.heap(bounds)?.checked_add(decision.heap(bounds)?),
            Self::Escalation { task, decision, hold } => {
                task.heap(bounds)?.checked_add(decision.heap(bounds)?)?.checked_add(hold.heap(bounds)?)
            }
            Self::Question { question, answer, ending } => {
                question.heap(bounds)?.checked_add(answer.heap(bounds)?)?.checked_add(ending.heap(bounds)?)
            }
            Self::PersonTask { document, result } => document.heap(bounds)?.checked_add(result.heap(bounds)?),
        }
    }
}
impl Payload for OwnerHistorical {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        self.value.heap(bounds)
    }
}
/// Inline envelope plus every independently owned nested allocation.
#[must_use]
pub fn bytes(value: &OwnerHistorical, bounds: &Bounds) -> Option<u64> {
    u64::try_from(size_of::<OwnerHistorical>()).ok()?.checked_add(value.heap(bounds)?)
}
/// Whole admission checks nested identities and all byte/list bounds.
#[must_use]
pub fn fits(value: &OwnerHistorical, bounds: &Bounds) -> bool {
    value.matches(value.key) && crate::payload::fits(value, bounds)
}
/// Existing recursive Result/Chip facts are limited by whole document admission,
/// not by an invented nesting-depth ceiling from rows and text alone.
#[must_use]
pub fn worst_case(bounds: &Bounds) -> Option<u64> {
    if bounds.document_bytes < u64::try_from(size_of::<OwnerHistorical>()).ok()? {
        None
    } else {
        Some(bounds.document_bytes)
    }
}
