//! Whole retained pages (client/domain/contract.md, 5–6).
//! Reserved rows, boxes, chip metadata/endings and opaque cursors are priced
//! without converting the owner's records into another document.
use crate::{Bounds, payload::Payload};
use core::mem::size_of;
use jig_client::contract::{OwnerEndedDelegate, OwnerEndedPage, OwnerFundingRecord, OwnerPeriodTimes, OwnerSpendPage};
impl Payload for OwnerEndedDelegate {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        self.chip.heap(bounds)?.checked_add(self.ended.heap(bounds)?)
    }
}
impl Payload for OwnerPeriodTimes {
    fn heap(&self, _: &Bounds) -> Option<u64> {
        Some(0)
    }
}
impl Payload for OwnerFundingRecord {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        self.period
            .heap(bounds)?
            .checked_add(self.funder.heap(bounds)?)?
            .checked_add(self.parent.heap(bounds)?)?
            .checked_add(self.numbers.heap(bounds)?)
    }
}
impl Payload for OwnerEndedPage {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        self.rows.heap(bounds)?.checked_add(self.before.heap(bounds)?)?.checked_add(self.after.heap(bounds)?)
    }
}
impl Payload for OwnerSpendPage {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        self.rows.heap(bounds)?.checked_add(self.before.heap(bounds)?)?.checked_add(self.after.heap(bounds)?)
    }
}
/// The full inline page plus each nested allocation.
#[must_use]
pub fn ended_bytes(value: &OwnerEndedPage, bounds: &Bounds) -> Option<u64> {
    u64::try_from(size_of::<OwnerEndedPage>()).ok()?.checked_add(value.heap(bounds)?)
}
/// The full inline page plus each reserved row and cursor byte.
#[must_use]
pub fn spend_bytes(value: &OwnerSpendPage, bounds: &Bounds) -> Option<u64> {
    u64::try_from(size_of::<OwnerSpendPage>()).ok()?.checked_add(value.heap(bounds)?)
}
#[must_use]
pub fn ended_fits(value: &OwnerEndedPage, bounds: &Bounds) -> bool {
    crate::payload::fits(value, bounds)
}
#[must_use]
pub fn spend_fits(value: &OwnerSpendPage, bounds: &Bounds) -> bool {
    crate::payload::fits(value, bounds)
}
/// Any admitted whole page consumes at most its byte ceiling.
#[must_use]
pub fn worst_case(bounds: &Bounds) -> Option<u64> {
    if bounds.document_bytes < u64::try_from(size_of::<OwnerEndedPage>().max(size_of::<OwnerSpendPage>())).ok()? {
        None
    } else {
        Some(bounds.document_bytes)
    }
}

/// Inline receiving vocabulary and boxed historical page storage together.
#[must_use]
pub fn read_bytes(value: &jig_client::contract::ReadValue, bounds: &Bounds) -> Option<u64> {
    u64::try_from(size_of::<jig_client::contract::ReadValue>()).ok()?.checked_add(value.heap(bounds)?)
}
#[must_use]
pub fn read_fits(value: &jig_client::contract::ReadValue, bounds: &Bounds) -> bool {
    crate::payload::fits(value, bounds)
}
#[cfg(test)]
mod tests;
