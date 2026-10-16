//! Whole authority payload pricing (client/domain/contract.md, 5.3).
//! Includes every list's allocated capacity and every opaque segment/end byte.
use crate::{Bounds, payload::Payload};
use core::mem::size_of;
use jig_client_domain::contract::{
    OwnerAuthority, OwnerAuthorityDocument, OwnerAuthorityExecutor, OwnerDelegation, OwnerGrant, OwnerPattern,
    OwnerPatternEnd, OwnerResourceScope,
};
use skein_lib::List;

impl Payload for OwnerPatternEnd {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Exact(bytes) | Self::Open(bytes) => bytes.heap(bounds),
        }
    }
}
impl Payload for OwnerPattern {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        self.segments.heap(bounds)?.checked_add(self.last.heap(bounds)?)
    }
}
impl Payload for OwnerGrant {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        self.pattern.heap(bounds)
    }
}
impl Payload for OwnerResourceScope {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        self.pattern.heap(bounds)
    }
}
impl Payload for OwnerAuthorityExecutor {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        Some(0)
    }
}
impl Payload for OwnerDelegation {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        self.kinds.heap(bounds)
    }
}
impl Payload for OwnerAuthority {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        self.grants
            .heap(bounds)?
            .checked_add(self.delegation.heap(bounds)?)?
            .checked_add(self.note_resources.heap(bounds)?)
    }
}
impl Payload for OwnerAuthorityDocument {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        self.authority.heap(bounds)
    }
}
/// Measured whole payload bytes, or an explicit nested bound/overflow failure.
#[must_use]
pub fn bytes(value: &OwnerAuthority, bounds: &Bounds) -> Option<u64> {
    value.heap(bounds)?.checked_add(u64::try_from(size_of::<OwnerAuthority>()).ok()?)
}
/// Measured whole document bytes, including its cut metadata wrapper.
#[must_use]
pub fn document_bytes(value: &OwnerAuthorityDocument, bounds: &Bounds) -> Option<u64> {
    value.heap(bounds)?.checked_add(u64::try_from(size_of::<OwnerAuthorityDocument>()).ok()?)
}
/// Admit the whole authority; a failure never returns a shortened value.
#[must_use]
pub fn fits(value: &OwnerAuthority, bounds: &Bounds) -> bool {
    crate::payload::fits(value, bounds)
}
/// Admit the whole document, including its completeness metadata.
#[must_use]
pub fn document_fits(value: &OwnerAuthorityDocument, bounds: &Bounds) -> bool {
    crate::payload::fits(value, bounds)
}
/// Maximum whole document allocation at the configured row and byte bounds.
/// An unrepresentable total is rejected explicitly.
#[must_use]
pub fn worst_case(bounds: &Bounds) -> Option<u64> {
    let rows = u64::from(bounds.rows);
    let pattern = List::<Box<[u8]>>::worst_case(bounds.rows)?
        .checked_add(rows.checked_mul(u64::from(bounds.text))?)?
        .checked_add(u64::from(bounds.text))?;
    let nested = pattern.checked_mul(rows)?.checked_mul(2)?;
    List::<OwnerGrant>::worst_case(bounds.rows)?
        .checked_add(List::<OwnerResourceScope>::worst_case(bounds.rows)?)?
        .checked_add(List::<OwnerAuthorityExecutor>::worst_case(bounds.rows)?)?
        .checked_add(nested)?
        .checked_add(u64::try_from(size_of::<OwnerAuthorityDocument>()).ok()?)
}
