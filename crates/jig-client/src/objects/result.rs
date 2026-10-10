//! Result facts are carried whole; this module supplies its typed projection.
//! Nothing computes authority or supplies words. See client/domain/objects.md, 8.
use super::{Body, Object};
/// Reads the document without copying the working set's one object.
#[must_use]
pub fn facts(object: &Object) -> Option<&crate::contract::Result> {
    match &object.body {
        Some(Body::Result(document)) => Some(document),
        Some(
            Body::Chip(_)
            | Body::Task(_)
            | Body::Proposal(_)
            | Body::OwnerProposal(_)
            | Body::Escalation(_)
            | Body::Question(_)
            | Body::PersonTask(_)
            | Body::Note { .. },
        )
        | None => None,
    }
}

/// A reply names its chat and the first unread reply, from its conversation line.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Reply {
    pub chat: u64,
    pub first_words: crate::contract::DocumentText,
    pub first_unread: u64,
}
impl Reply {
    /// Keeps whole words and checks the first unread position before ownership.
    #[must_use]
    pub fn from_line(line: crate::contract::ConversationLine) -> Option<Self> {
        if line.last_reply <= line.seen {
            return None;
        }
        Some(Self { chat: line.chip.task, first_words: line.first_words, first_unread: line.seen.checked_add(1)? })
    }
}
