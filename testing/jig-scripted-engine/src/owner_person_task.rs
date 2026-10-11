//! Whole pending person work admission (client/domain/contract.md, 5.3).
use crate::{Bounds, payload::Payload};
use core::mem::size_of;
use jig_client::contract::{OwnerPersonTask, OwnerShownResult};
impl Payload for OwnerShownResult {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        self.task.heap(bounds)?.checked_add(self.result.heap(bounds)?)
    }
}
impl Payload for OwnerPersonTask {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        self.task
            .heap(bounds)?
            .checked_add(self.spec.heap(bounds)?)?
            .checked_add(self.contract.heap(bounds)?)?
            .checked_add(self.results.heap(bounds)?)?
            .checked_add(crate::offered::heap(&self.acts, bounds)?)
    }
}
#[must_use]
pub fn bytes(value: &OwnerPersonTask, bounds: &Bounds) -> Option<u64> {
    u64::try_from(size_of::<OwnerPersonTask>()).ok()?.checked_add(value.heap(bounds)?)
}
#[must_use]
pub fn fits(value: &OwnerPersonTask, bounds: &Bounds) -> bool {
    crate::payload::fits(value, bounds)
}
#[must_use]
pub fn worst_case(bounds: &Bounds) -> Option<u64> {
    if bounds.document_bytes < u64::try_from(size_of::<OwnerPersonTask>()).ok()? {
        None
    } else {
        Some(bounds.document_bytes)
    }
}
#[cfg(test)]
pub(crate) mod tests;
