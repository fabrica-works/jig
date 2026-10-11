//! Whole immutable history admission (client/domain/contract.md, 5.3;
//! client/domain/objects.md, 11). No shortened history or batch is returned.
use crate::{Bounds, payload::Payload};
use core::mem::size_of;
use jig_client::contract::{
    FieldRule, OwnerAuthorityDocument, OwnerHistoryLine, OwnerPart, OwnerProposalHistory, OwnerProposalHistoryState,
    OwnerRetainedAction, OwnerRetainedAmendment, ParameterDocument, ProposedTask, VerdictChoice,
};
use skein_lib::List;

impl Payload for OwnerRetainedAmendment {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        self.addressed
            .heap(bounds)?
            .checked_add(self.instructions.heap(bounds)?)?
            .checked_add(self.release.heap(bounds)?)?
            .checked_add(self.spec.heap(bounds)?)?
            .checked_add(self.authority.heap(bounds)?)?
            .checked_add(self.wake.heap(bounds)?)?
            .checked_add(self.dependencies.heap(bounds)?)?
            .checked_add(self.reason.heap(bounds)?)
    }
}
impl Payload for OwnerRetainedAction {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Batch(tasks) => tasks.heap(bounds),
            Self::Effect { part, authority, .. } | Self::ProcedureEffect { part, authority, .. } => {
                part.heap(bounds)?.checked_add(authority.heap(bounds)?)
            }
            Self::Amend { change, .. } => change.heap(bounds),
            Self::Widen { authority, .. } => authority.heap(bounds),
            Self::Release { .. } => Some(0),
        }
    }
}
impl Payload for OwnerProposalHistoryState {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Rejected { reason, .. } => reason.heap(bounds),
            Self::Pending { .. } | Self::Accepted { .. } | Self::Withdrawn => Some(0),
        }
    }
}
impl Payload for OwnerProposalHistory {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        self.action.heap(bounds)?.checked_add(self.state.heap(bounds)?)
    }
}
impl Payload for OwnerHistoryLine {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        let snapshots = match &self.amendment {
            Some((before, after)) => before.heap(bounds)?.checked_add(after.heap(bounds)?)?,
            None => 0,
        };
        self.reason.heap(bounds)?.checked_add(self.proposal.heap(bounds)?)?.checked_add(snapshots)
    }
}
/// Inline history value plus every allocated list capacity, box and opaque byte.
#[must_use]
pub fn bytes(value: &OwnerHistoryLine, bounds: &Bounds) -> Option<u64> {
    value.heap(bounds)?.checked_add(u64::try_from(size_of::<OwnerHistoryLine>()).ok()?)
}
/// Refuse the whole history, preserving the supplied value unchanged.
#[must_use]
pub fn fits(value: &OwnerHistoryLine, bounds: &Bounds) -> bool {
    crate::payload::fits(value, bounds)
}
/// Finite maximum across all retained actions, states and two stored snapshots.
#[must_use]
pub fn worst_case(bounds: &Bounds) -> Option<u64> {
    let rows = u64::from(bounds.rows);
    let text = u64::from(bounds.text);
    let authority = crate::owner_authority::worst_case(bounds)?
        .checked_sub(u64::try_from(size_of::<OwnerAuthorityDocument>()).ok()?)?;
    let part = List::<Box<[u8]>>::worst_case(bounds.rows)?.checked_add(rows.checked_mul(text)?)?;
    let spec = List::<ParameterDocument>::worst_case(bounds.rows)?
        .checked_add(rows.checked_mul(text)?)?
        .checked_add(List::<u64>::worst_case(bounds.rows)?)?
        .checked_add(text)?;
    let verdict = List::<VerdictChoice>::worst_case(bounds.rows)?.checked_add(
        rows.checked_mul(List::<FieldRule>::worst_case(bounds.rows)?.checked_add(rows.checked_mul(text)?)?)?,
    )?;
    let contract = verdict.max(text.checked_add(part)?);
    let proposed = spec
        .checked_add(authority)?
        .checked_add(contract)?
        .checked_add(List::<u64>::worst_case(bounds.rows)?)?
        .checked_add(List::<OwnerPart>::worst_case(bounds.rows)?)?
        .checked_add(rows.checked_mul(part)?)?;
    let batch = List::<ProposedTask>::worst_case(bounds.rows)?.checked_add(rows.checked_mul(proposed)?)?;
    let amendment = u64::try_from(size_of::<OwnerRetainedAmendment>())
        .ok()?
        .checked_add(spec)?
        .checked_add(authority)?
        .checked_add(List::<u64>::worst_case(bounds.rows)?)?
        .checked_add(text.checked_mul(2)?)?;
    let action = batch.max(amendment).max(part.checked_add(authority)?).max(authority);
    u64::try_from(size_of::<OwnerHistoryLine>())
        .ok()?
        .checked_add(action)?
        .checked_add(text.checked_mul(2)?)?
        .checked_add(crate::owner_amendment::worst_case(bounds)?.checked_mul(2)?)
}
#[cfg(test)]
mod tests;
