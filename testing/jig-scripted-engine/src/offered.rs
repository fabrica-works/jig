//! Whole closed offered-act allocation admission (client/domain/contract.md, 5.3).
//! Ordinary row bounds continue to apply to every nested supplied list.
use crate::{Bounds, payload::Payload};
use jig_domain_client::contract::{MAX_OFFERED_ACTS, Offered};
use skein_lib::List;

/// Price every reserved offer slot and every populated admission recursively.
/// The enclosing payload adds its inline bytes and checks `document_bytes`.
#[must_use]
pub fn heap(value: &List<Offered>, bounds: &Bounds) -> Option<u64> {
    if value.capacity() > MAX_OFFERED_ACTS {
        return None;
    }
    let mut total = List::<Offered>::worst_case(value.capacity())?;
    for offer in value {
        total = total.checked_add(offer.heap(bounds)?)?;
    }
    Some(total)
}
