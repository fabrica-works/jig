//! Recurring template overlap facts (domain/tasks.md, section 10).
//! The engine supplies this policy; the client does not schedule batches.

/// What the template does when the preceding batch is still live.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RecurringOverlap {
    /// Let the due period pass without another batch.
    Skip,
    /// Make the latest pending period's batch after the old batch ends.
    Wait,
}
