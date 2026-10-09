//! Person task facts are carried whole; this module supplies its typed projection.
//! Nothing computes authority or supplies words. See client/domain/objects.md, 7.
use super::{Body, Object};
/// Reads the document without copying the working set's one object.
#[must_use]
pub fn facts(object: &Object) -> Option<&crate::contract::PersonTask> {
    match &object.body {
        Some(Body::PersonTask(document)) => Some(document),
        Some(
            Body::Chip(_)
            | Body::Task(_)
            | Body::Proposal(_)
            | Body::Escalation(_)
            | Body::Question(_)
            | Body::Result(_)
            | Body::Note { .. },
        )
        | None => None,
    }
}
/// Names only the independently decidable facts on this card.
#[must_use]
pub fn revision(object: &Object) -> Option<crate::contract::Revision> {
    let document = facts(object)?;
    Some(document.revision)
}
