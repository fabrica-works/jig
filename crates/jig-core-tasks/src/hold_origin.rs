//! Bounded references to connector explanations (`client/domain/objects.md`, 5).
use crate::{Hold, HoldSource, Limits};
pub(crate) fn valid(limits: &Limits, why: Hold, source: Option<&HoldSource>) -> bool {
    match source {
        None => true,
        Some(HoldSource::Resource(name)) => why == Hold::Drift && crate::holds::valid_name(limits, name),
        Some(HoldSource::Effect { entry, .. }) => {
            *entry != 0
                && match why {
                    Hold::EffectFailed | Hold::Effects => true,
                    Hold::Uncertain { entry: held } => held == *entry,
                    Hold::Failures(_)
                    | Hold::Stalled
                    | Hold::Dependency(_)
                    | Hold::Stopped
                    | Hold::StoppedBy { .. }
                    | Hold::Drift
                    | Hold::Budget
                    | Hold::Deadline
                    | Hold::HoldsWaited
                    | Hold::ResourceUnavailable
                    | Hold::ResourceAccess
                    | Hold::PoolLost { .. }
                    | Hold::Procedure => false,
                }
        }
    }
}
