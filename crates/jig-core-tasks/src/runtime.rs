//! Durable lifecycle observations for chips (client/domain/objects.md, section 3).
//! Timestamps come from injected transition events; retries name the decision
//! that paused an activation, rather than a counter selected by a reader.
//! The row owns the attempt identity; a new claim clears both observations.
//! Its start report is root placement/adoption timing, not agent/tool timing.
use crate::{Active, Class, Phase, TaskRecord, Was};

/// The decision responsible for the most recent retry pause.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum RetryNotice {
    /// A classified attempt failure, with its counter and configured allowance.
    Failed { class: Class, count: u32, allowance: u32 },
    /// Preparation or placement refused before execution; no failure try spent.
    Refused { count: u32 },
}

pub(crate) fn valid(task: &TaskRecord) -> bool {
    if task.run_since.is_some() && task.attempt == 0 {
        return false;
    }
    let notice_valid = match task.retry_notice {
        Some(RetryNotice::Failed { count, allowance, .. }) => task.attempt != 0 && count != 0 && allowance != u32::MAX,
        Some(RetryNotice::Refused { count }) => count != 0,
        None => true,
    };
    if !notice_valid {
        return false;
    }
    match task.phase {
        Phase::Active(Active::Claimed { .. }) | Phase::Held { was: Was::Active(Active::Claimed { .. }), .. } => {
            task.run_since.is_none()
        }
        Phase::Active(Active::Running { .. }) | Phase::Held { was: Was::Active(Active::Running { .. }), .. } => {
            task.run_since.is_some()
        }
        Phase::Active(Active::BackingOff { .. }) | Phase::Held { was: Was::Active(Active::BackingOff { .. }), .. } => {
            match task.retry_notice {
                Some(RetryNotice::Failed { class, count, allowance }) => {
                    count == task.tries.of(class) && count <= allowance
                }
                Some(RetryNotice::Refused { count }) => count == task.refusals,
                None => false,
            }
        }
        Phase::Waiting | Phase::Active(_) | Phase::Closing(_) | Phase::Held { .. } | Phase::Ended(_) => true,
    }
}
