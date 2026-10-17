//! Engine-owned consequences of a decision (client/domain/contract.md, 5.4).
use crate::documents::PhaseCounts;
/// What lifting the task's hold does.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ReleaseEffect {
    Judged,
    RunAgain,
    Uncertain,
}
/// The live subtree and work a cancellation ends.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Impact {
    pub tasks: PhaseCounts,
    pub live_runs: u32,
    pub proposals: u32,
}
/// A consequence supplied beside the engine's admission.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Effect {
    Release(ReleaseEffect),
    Cancel(Impact),
    Move { reservation: u64 },
}
