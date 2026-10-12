//! Whole owner amendment admission (client/domain/contract.md, 5.3;
//! domain/tasks.md, 7). No shortened value is returned on a bound failure.
use crate::{Bounds, payload::Payload};
use core::mem::size_of;
use jig_domain_client::contract::{OwnerAmendmentState, OwnerAuthorityDocument, ParameterDocument};
use skein_lib::List;

impl Payload for OwnerAmendmentState {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        let Self { addressed, taken_by, instructions, spec, authority, wake, dependencies, cut } = self;
        addressed
            .heap(bounds)?
            .checked_add(taken_by.heap(bounds)?)?
            .checked_add(instructions.heap(bounds)?)?
            .checked_add(spec.heap(bounds)?)?
            .checked_add(authority.heap(bounds)?)?
            .checked_add(wake.heap(bounds)?)?
            .checked_add(dependencies.heap(bounds)?)?
            .checked_add(cut.heap(bounds)?)
    }
}
/// Whole inline value plus all allocated list capacities and nested text bytes.
#[must_use]
pub fn bytes(value: &OwnerAmendmentState, bounds: &Bounds) -> Option<u64> {
    value.heap(bounds)?.checked_add(u64::try_from(size_of::<OwnerAmendmentState>()).ok()?)
}
/// Refuse the whole value if any text, allocated list capacity or byte total is too large.
#[must_use]
pub fn fits(value: &OwnerAmendmentState, bounds: &Bounds) -> bool {
    crate::payload::fits(value, bounds)
}
/// Maximum storage at the configured row and per-text bounds, including optional instructions.
#[must_use]
pub fn worst_case(bounds: &Bounds) -> Option<u64> {
    let authority = crate::owner_authority::worst_case(bounds)?
        .checked_sub(u64::try_from(size_of::<OwnerAuthorityDocument>()).ok()?)?;
    let text = u64::from(bounds.text);
    u64::try_from(size_of::<OwnerAmendmentState>())
        .ok()?
        .checked_add(authority)?
        .checked_add(List::<ParameterDocument>::worst_case(bounds.rows)?)?
        .checked_add(u64::from(bounds.rows).checked_mul(text)?)?
        .checked_add(List::<u64>::worst_case(bounds.rows)?.checked_mul(2)?)?
        .checked_add(text.checked_mul(2)?)
}

#[cfg(test)]
mod tests;
