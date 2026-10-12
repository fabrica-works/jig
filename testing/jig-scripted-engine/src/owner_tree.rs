//! Whole subtree admission (client/domain/contract.md, 5.3).
//! Every reserved list, chip, dependency and cursor allocation is charged.
use crate::{Bounds, payload::Payload};
use core::mem::size_of;
use jig_domain_client::contract::{OwnerTree, OwnerTreeFold, OwnerTreeRow};
impl Payload for OwnerTreeRow {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        self.chip.heap(bounds)?.checked_add(self.proposals.heap(bounds)?)?.checked_add(self.dependencies.heap(bounds)?)
    }
}
impl Payload for OwnerTreeFold {
    fn heap(&self, _: &Bounds) -> Option<u64> {
        Some(0)
    }
}
impl Payload for OwnerTree {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        self.root
            .heap(bounds)?
            .checked_add(self.rows.heap(bounds)?)?
            .checked_add(self.folded.heap(bounds)?)?
            .checked_add(self.next.heap(bounds)?)
    }
}
/// Inline envelope and all nested allocated capacity.
#[must_use]
pub fn bytes(value: &OwnerTree, bounds: &Bounds) -> Option<u64> {
    u64::try_from(size_of::<OwnerTree>()).ok()?.checked_add(value.heap(bounds)?)
}
/// Admit the complete envelope, never a shortened tree.
#[must_use]
pub fn fits(value: &OwnerTree, bounds: &Bounds) -> bool {
    crate::payload::fits(value, bounds)
}
/// Finite admitted ceiling; nested chip results share the document byte bound.
#[must_use]
pub fn worst_case(bounds: &Bounds) -> Option<u64> {
    if bounds.document_bytes < u64::try_from(size_of::<OwnerTree>()).ok()? { None } else { Some(bounds.document_bytes) }
}
#[cfg(test)]
mod tests;
