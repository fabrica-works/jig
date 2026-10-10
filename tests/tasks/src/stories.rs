//! The lifecycle's choice vocabulary, independent of drawn arrival/pricing times.

/// The retry's independently observed ending before its fresh claim.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RetryEnd {
    Refused,
    Lost,
    None,
}

/// Whether the staged turn is committed before the mandatory restart.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TurnState {
    Staged,
    Committed,
}

/// An additional restart at one of the story's optional cuts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Restart {
    Skip,
    Restore,
}

/// The child result reported before its dependents settle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChildResult {
    Succeeded,
    Failed,
}

/// The five lifecycle choices previously drawn from each sweep seed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Story {
    pub turn: TurnState,
    pub retry: RetryEnd,
    pub restart_after_retry: Restart,
    pub child: ChildResult,
    pub restart_before_close: Restart,
}
impl Story {
    /// Enumerate every 2 × 3 × 2 × 2 × 2 choice combination, not a random sample.
    #[must_use]
    pub fn all() -> Vec<Self> {
        let mut stories = Vec::with_capacity(48);
        for turn in [TurnState::Staged, TurnState::Committed] {
            for retry in [RetryEnd::Refused, RetryEnd::Lost, RetryEnd::None] {
                for restart_after_retry in [Restart::Skip, Restart::Restore] {
                    for child in [ChildResult::Succeeded, ChildResult::Failed] {
                        for restart_before_close in [Restart::Skip, Restart::Restore] {
                            stories.push(Self { turn, retry, restart_after_retry, child, restart_before_close });
                        }
                    }
                }
            }
        }
        stories
    }
}
