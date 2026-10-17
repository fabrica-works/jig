//! Consequences supplied with each offered act (contract.md, 5.4).
use super::Impact;
/// What lifting the task's hold does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ReleaseEffect {
    Judged,
    RunAgain,
    Uncertain,
}
/// What the engine's current decision changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Effect {
    Release(ReleaseEffect),
    Cancel(Impact),
    Move { reservation: u64 },
}
