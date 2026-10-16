//! Question facts are carried whole; this module supplies its typed projection.
//! Nothing computes authority or supplies words. See client/domain/objects.md, 6.
use super::{Body, Object};
/// Reads the document without copying the working set's one object.
#[must_use]
pub fn facts(object: &Object) -> Option<&jig_client_domain::contract::Question> {
    match &object.body {
        Some(Body::Question(document)) => Some(document),
        Some(
            Body::Historical(_)
            | Body::Chip(_)
            | Body::Task(_)
            | Body::Proposal(_)
            | Body::OwnerProposal(_)
            | Body::Escalation(_)
            | Body::OwnerEscalation(_)
            | Body::PersonTask(_)
            | Body::OwnerPersonTask(_)
            | Body::Result(_)
            | Body::Note { .. },
        )
        | None => None,
    }
}
