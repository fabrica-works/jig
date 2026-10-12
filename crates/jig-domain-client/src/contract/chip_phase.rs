//! Exact chip lifecycle (domain/tasks.md, 5; client/domain/objects.md, 3).
//! Timing, observations and hold history remain in the separate mandatory metadata.
use super::{ClosingStage, HoldCategory, PersonAddress, Phase, RunStage};
use skein_lib::{List, Wall};

/// The whole producer phase, independent of observation and optional ending facts.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum OwnerChipPhase {
    Waiting { dependencies: List<u64>, more: bool, holds: bool },
    Active(RunStage),
    Person { address: PersonAddress, taken_by: Option<u64>, since: Option<Wall> },
    Closing(ClosingStage),
    Held { reason: HoldCategory, revision: u64 },
    Ended,
}
impl OwnerChipPhase {
    /// A compact filter value, without inventing an ended outcome.
    #[must_use]
    pub const fn compact(&self) -> Phase {
        match self {
            Self::Waiting { .. } => Phase::Waiting,
            Self::Active(stage) => match stage {
                RunStage::Idle => Phase::Idle,
                RunStage::Due => Phase::Due,
                RunStage::Preparing => Phase::Preparing,
                RunStage::Claimed { .. } => Phase::Claimed,
                RunStage::Running { .. } => Phase::Running,
                RunStage::BackingOff { .. } => Phase::BackingOff,
            },
            Self::Person { .. } => Phase::Active,
            Self::Closing(_) => Phase::Closing,
            Self::Held { .. } => Phase::Held,
            Self::Ended => Phase::Ended,
        }
    }
}
