//! Immutable lifecycle milestones (client/domain/objects.md, 11; domain/tasks.md, 7).
//! Position is the supplied activity position, independent of semantic revisions.
use super::{ClosingStage, HoldCategory, OwnerChipEnding, OwnerPart, OwnerPriorPhase, RunStage};
use skein_lib::Wall;

/// One lifecycle change with its separately supplied wall clock.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OwnerMilestoneLine {
    pub task: u64,
    pub position: u64,
    pub phase: OwnerMilestonePhase,
    pub at: Option<Wall>,
}
/// The owner's lifecycle, without live chip observations or inferred prior clocks.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum OwnerMilestonePhase {
    /// Not yet activated.
    Waiting,
    /// A retained activation stage.
    Active(RunStage),
    /// Closing obligations and the whole retained ending.
    Closing { stage: ClosingStage, ending: OwnerChipEnding },
    /// Held prior lifecycle, reason, raw source and independently optional ending.
    Held { before: OwnerPriorPhase, why: HoldCategory, source: Option<OwnerPart>, ending: Option<OwnerChipEnding> },
    /// The whole terminal ending.
    Ended(OwnerChipEnding),
}
