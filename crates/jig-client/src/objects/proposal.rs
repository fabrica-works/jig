//! Proposal facts are carried whole; this module supplies its typed projection.
//! Nothing computes authority or supplies words. See client/domain/objects.md, 4.
use super::{Body, Object};
/// Reads the document without copying the working set's one object.
#[must_use]
pub fn facts(object: &Object) -> Option<&crate::contract::Proposal> {
    match &object.body {
        Some(Body::Proposal(document)) => Some(document),
        Some(
            Body::Chip(_)
            | Body::Task(_)
            | Body::Escalation(_)
            | Body::Question(_)
            | Body::PersonTask(_)
            | Body::Result(_)
            | Body::Note { .. },
        )
        | None => None,
    }
}
