//! Ended goals use their stored subtree census (protocol/store.md, 7.3).
use super::PhaseCounts;
impl PhaseCounts {
    #[must_use]
    pub const fn from_endings(endings: jig_domain_tasks::Endings) -> Self {
        Self { done: endings.done, failed: endings.failed, cancelled: endings.cancelled, ..Self::ZERO }
    }
}
