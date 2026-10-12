//! Whole owner budget/ledger pricing (client/domain/contract.md, 5.3).
//! Inline accounting and source identities are retained as one value;
//! the shared Funder pricing belongs to the proposed-task vocabulary.
use crate::{Bounds, payload::Payload};
use core::mem::size_of;
use jig_domain_client::contract::{OwnerBudget, OwnerNumbers, PoolLedger};

impl Payload for OwnerNumbers {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        Some(0)
    }
}
impl Payload for OwnerBudget {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        self.numbers.heap(bounds)?.checked_add(self.funder.heap(bounds)?)
    }
}
impl Payload for PoolLedger {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        self.source.heap(bounds)?.checked_add(self.parent.heap(bounds)?)?.checked_add(self.numbers.heap(bounds)?)
    }
}

/// Complete retained budget bytes, including its supplied optional deadline.
#[must_use]
pub fn bytes(value: &OwnerBudget, bounds: &Bounds) -> Option<u64> {
    value.heap(bounds)?.checked_add(u64::try_from(size_of::<OwnerBudget>()).ok()?)
}
/// Complete retained ledger bytes, including all optional facts and both sources.
#[must_use]
pub fn ledger_bytes(value: &PoolLedger, bounds: &Bounds) -> Option<u64> {
    value.heap(bounds)?.checked_add(u64::try_from(size_of::<PoolLedger>()).ok()?)
}
/// Admit the whole supplied budget or refuse without shortening it.
#[must_use]
pub fn fits(value: &OwnerBudget, bounds: &Bounds) -> bool {
    crate::payload::fits(value, bounds)
}
/// Admit the whole supplied ledger or refuse without shortening it.
#[must_use]
pub fn ledger_fits(value: &PoolLedger, bounds: &Bounds) -> bool {
    crate::payload::fits(value, bounds)
}
/// Fixed whole budget footprint: its Funder has only inline scalar fields.
#[must_use]
pub fn budget_worst_case() -> Option<u64> {
    u64::try_from(size_of::<OwnerBudget>()).ok()
}
/// Fixed whole ledger footprint: both source identities have only inline fields.
#[must_use]
pub fn ledger_worst_case() -> Option<u64> {
    u64::try_from(size_of::<PoolLedger>()).ok()
}
