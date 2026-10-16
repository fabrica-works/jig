//! Shared amendment payloads for the tests.
/// An amendment retaining every field, with the supplied explanation.
#[must_use]
pub(super) fn unchanged(reason: &[u8]) -> crate::Amendment {
    crate::Amendment {
        tracked: None,
        addressed: None,
        instructions: None,
        spec: None,
        wake: None,
        dependencies: None,
        authority: None,
        reason: reason.into(),
    }
}
