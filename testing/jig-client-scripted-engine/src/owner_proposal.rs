//! Whole pending proposal admission (client/domain/contract.md, 5.3).
//! Every allocated list slot, box and opaque byte is priced before admission.
use crate::{Bounds, payload::Payload};
use core::mem::size_of;
use jig_client_domain::contract::{
    OwnerFundingOffer, OwnerHistoryHolder, OwnerProposal, OwnerProposalAction, OwnerProposalWaiting,
};
impl Payload for OwnerProposalWaiting {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        match self.holder {
            OwnerHistoryHolder::Task(_) | OwnerHistoryHolder::Person(_) | OwnerHistoryHolder::Policy { .. } => Some(0),
        }
    }
}
impl Payload for OwnerFundingOffer {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        self.source.heap(bounds)
    }
}
impl Payload for OwnerProposalAction {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Settings(value) => value.heap(bounds),
            Self::Batch(tasks) => tasks.heap(bounds),
            Self::Effect { part, authority, .. } | Self::ProcedureEffect { part, authority, .. } => {
                part.heap(bounds)?.checked_add(authority.heap(bounds)?)
            }
            Self::Amend { task, addressed, before, after, reason, release } => task
                .heap(bounds)?
                .checked_add(addressed.heap(bounds)?)?
                .checked_add(before.heap(bounds)?)?
                .checked_add(after.heap(bounds)?)?
                .checked_add(reason.heap(bounds)?)?
                .checked_add(release.heap(bounds)?),
            Self::Widen { task, before, after } => {
                task.heap(bounds)?.checked_add(before.heap(bounds)?)?.checked_add(after.heap(bounds)?)
            }
            Self::Release { task, hold } => task.heap(bounds)?.checked_add(hold.heap(bounds)?),
        }
    }
}
impl Payload for OwnerProposal {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        self.reason
            .heap(bounds)?
            .checked_add(self.action.heap(bounds)?)?
            .checked_add(self.waiting.heap(bounds)?)?
            .checked_add(self.funding.heap(bounds)?)?
            .checked_add(crate::offered::heap(&self.acts, bounds)?)
    }
}
/// Inline carrier plus all nested allocated capacity and supplied text bytes.
#[must_use]
pub fn bytes(value: &OwnerProposal, bounds: &Bounds) -> Option<u64> {
    u64::try_from(size_of::<OwnerProposal>()).ok()?.checked_add(value.heap(bounds)?)
}
/// Refuse the whole carrier without altering its source facts.
#[must_use]
pub fn fits(value: &OwnerProposal, bounds: &Bounds) -> bool {
    crate::payload::fits(value, bounds)
}
/// Finite receiving byte ceiling, including all recursively supplied lower values.
#[must_use]
pub fn worst_case(bounds: &Bounds) -> Option<u64> {
    if bounds.document_bytes < u64::try_from(size_of::<OwnerProposal>()).ok()? {
        None
    } else {
        Some(bounds.document_bytes)
    }
}
#[cfg(test)]
mod tests;
