//! The lifecycle's branch vocabulary, independent of drawn arrival/pricing times.

/// The retry's independently observed ending before its fresh claim.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RetryEnd {
    Refused,
    Lost,
    None,
}

/// The five lifecycle choices previously drawn from each sweep seed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Story {
    pub durable_turn: bool,
    pub retry: RetryEnd,
    pub restart_after_retry: bool,
    pub child_failure: bool,
    pub restart_before_close: bool,
}
impl Story {
    /// Enumerate every 2 × 3 × 2 × 2 × 2 branch combination, not a random sample.
    #[must_use]
    pub fn all() -> Vec<Self> {
        let mut stories = Vec::with_capacity(48);
        for durable_turn in [false, true] {
            for retry in [RetryEnd::Refused, RetryEnd::Lost, RetryEnd::None] {
                for restart_after_retry in [false, true] {
                    for child_failure in [false, true] {
                        for restart_before_close in [false, true] {
                            stories.push(Self {
                                durable_turn,
                                retry,
                                restart_after_retry,
                                child_failure,
                                restart_before_close,
                            });
                        }
                    }
                }
            }
        }
        stories
    }
}
