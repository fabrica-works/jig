//! Exact counts from the owner (client/domain/contract.md, 5.1).
/// Ten independent supplied lifecycle counters; local folded chips use PhaseCounts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OwnerPhaseCounts {
    pub waiting: u32,
    pub idle: u32,
    pub starting: u32,
    pub running: u32,
    pub backing_off: u32,
    pub closing: u32,
    pub held: u32,
    pub done: u32,
    pub failed: u32,
    pub cancelled: u32,
}
impl OwnerPhaseCounts {
    /// Explicitly supplied zero counts for synthetic fixtures.
    pub const ZERO: Self = Self {
        waiting: 0,
        idle: 0,
        starting: 0,
        running: 0,
        backing_off: 0,
        closing: 0,
        held: 0,
        done: 0,
        failed: 0,
        cancelled: 0,
    };
}
