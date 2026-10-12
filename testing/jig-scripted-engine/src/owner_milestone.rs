//! Whole milestone admission (client/domain/contract.md, 5.3).
//! Nested results retain all allocated capacity; the receiving byte bound
//! supplies the finite ceiling for recursively nested result values.
use crate::{Bounds, payload::Payload};
use core::mem::size_of;
use jig_domain_client::contract::{OwnerMilestoneLine, OwnerMilestonePhase};
impl Payload for OwnerMilestoneLine {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        self.phase.heap(bounds)
    }
}
impl Payload for OwnerMilestonePhase {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Waiting | Self::Active(_) => Some(0),
            Self::Closing { ending, .. } | Self::Ended(ending) => ending.heap(bounds),
            Self::Held { source, ending, .. } => source.heap(bounds)?.checked_add(ending.heap(bounds)?),
        }
    }
}
/// Inline milestone plus every recursive allocation, without shortening source.
#[must_use]
pub fn bytes(value: &OwnerMilestoneLine, bounds: &Bounds) -> Option<u64> {
    u64::try_from(size_of::<OwnerMilestoneLine>()).ok()?.checked_add(value.heap(bounds)?)
}
/// Admit the whole value at the receiving document byte boundary.
#[must_use]
pub fn fits(value: &OwnerMilestoneLine, bounds: &Bounds) -> bool {
    crate::payload::fits(value, bounds)
}
/// Finite ceiling for admitted whole values, including recursive result values.
/// Rows/text alone do not supply a recursive depth bound.
#[must_use]
pub fn worst_case(bounds: &Bounds) -> Option<u64> {
    if bounds.document_bytes < u64::try_from(size_of::<OwnerMilestoneLine>()).ok()? {
        None
    } else {
        Some(bounds.document_bytes)
    }
}
#[cfg(test)]
mod tests;
