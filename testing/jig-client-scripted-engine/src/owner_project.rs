//! Whole allocated Project admission; client/domain/contract.md, 5.3.
use crate::{Bounds, payload::Payload};
use core::mem::size_of;
use jig_client_domain::contract::{
    OwnerProject, OwnerProjectMember, OwnerProjectPolicy, OwnerProjectPolicyRole, OwnerProjectResource,
    OwnerProjectResourceRole, OwnerProjectSpend,
};
impl Payload for OwnerProjectResourceRole {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Owned | Self::Participant | Self::Unavailable | Self::Context => Some(0),
        }
    }
}
impl Payload for OwnerProjectResource {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        self.part.heap(bounds)
    }
}
impl Payload for OwnerProjectPolicyRole {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        self.authority.heap(bounds)
    }
}
impl Payload for OwnerProjectPolicy {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        self.ceiling
            .heap(bounds)?
            .checked_add(self.projections.heap(bounds)?)?
            .checked_add(self.roles.heap(bounds)?)?
            .checked_add(self.requirements.heap(bounds)?)?
            .checked_add(self.permissions.heap(bounds)?)?
            .checked_add(self.templates.heap(bounds)?)
    }
}
impl Payload for OwnerProjectMember {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        self.login.heap(bounds)?.checked_add(self.name.heap(bounds)?)
    }
}
impl Payload for OwnerProjectSpend {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        self.chip.heap(bounds)
    }
}
impl Payload for OwnerProject {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        crate::offered::heap(&self.acts, bounds)?
            .checked_add(self.resources.heap(bounds)?)?
            .checked_add(self.policy.heap(bounds)?)?
            .checked_add(self.members.heap(bounds)?)?
            .checked_add(self.current_period.heap(bounds)?)?
            .checked_add(self.pools.heap(bounds)?)?
            .checked_add(self.configured_pools.heap(bounds)?)?
            .checked_add(self.spend.heap(bounds)?)?
            .checked_add(self.recurring.heap(bounds)?)
    }
}
/// Inline carrier and every allocated list, box, text and nested value.
#[must_use]
pub fn bytes(value: &OwnerProject, bounds: &Bounds) -> Option<u64> {
    u64::try_from(size_of::<OwnerProject>()).ok()?.checked_add(value.heap(bounds)?)
}
/// Refusal leaves the supplied whole value unchanged.
#[must_use]
pub fn fits(value: &OwnerProject, bounds: &Bounds) -> bool {
    crate::payload::fits(value, bounds)
}
/// All admitted recursive values remain below this exact byte ceiling.
#[must_use]
pub fn worst_case(bounds: &Bounds) -> Option<u64> {
    if bounds.document_bytes < u64::try_from(size_of::<OwnerProject>()).ok()? {
        None
    } else {
        Some(bounds.document_bytes)
    }
}
