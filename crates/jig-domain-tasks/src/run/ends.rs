//! One rule for going on after an amendment (domain/tasks.md, 5.7 and section 7).
use crate::domain::{Domain, record};
use crate::{Active, End, Executor, Phase, TaskResult, Was};

/// Whether an active task may go on, and why not (domain/tasks.md, 5.7).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Kept {
    /// A dependency an amendment added is pending.
    pub(crate) dependencies: bool,
}

pub(crate) fn kept(domain: &Domain, task: u64) -> Kept {
    match record(domain, task) {
        Some(row) => for_record(row),
        None => pending(&[]),
    }
}

/// Publication borrows its row mutably; closing keeps cancellation's partial result.
pub(crate) fn for_record(row: &crate::TaskRecord) -> Kept {
    match row.phase {
        Phase::Active(_) | Phase::Held { was: Was::Active(_), .. } => pending(&row.waiting_on),
        Phase::Waiting
        | Phase::Closing(_)
        | Phase::Held { was: Was::Waiting | Was::Closing(_), .. }
        | Phase::Ended(_) => pending(&[]),
    }
}

/// The same facts for a compound request's projected dependency list.
pub(crate) fn pending(dependencies: &[u64]) -> Kept {
    Kept { dependencies: !dependencies.is_empty() }
}

impl Kept {
    pub(crate) fn blocks_claims(self) -> bool {
        self.dependencies
    }

    /// An amendment invalidates an agent's queued preparation; a committed claim stays live.
    pub(crate) fn amended(self, executor: Executor, phase: Phase) -> Phase {
        let agent = match executor {
            Executor::Agent { .. } => true,
            Executor::Person(_) | Executor::Procedure { .. } => false,
        };
        if self.blocks_claims() && agent {
            match phase {
                Phase::Active(Active::Due | Active::Preparing) => Phase::Active(Active::Idle),
                Phase::Waiting
                | Phase::Active(
                    Active::Idle | Active::Claimed { .. } | Active::Running { .. } | Active::BackingOff { .. },
                )
                | Phase::Held { .. }
                | Phase::Closing(_)
                | Phase::Ended(_) => phase,
            }
        } else {
            phase
        }
    }

    /// A failed result still ends; a done result keeps the committed conversation.
    pub(crate) fn parks(self, end: &End) -> bool {
        self.dependencies
            && match end {
                End::Finished { result, .. } | End::FinishedWithFollowups { result, .. } => match result {
                    TaskResult::Failure { .. } => false,
                    TaskResult::Approved { .. }
                    | TaskResult::Report { .. }
                    | TaskResult::Verdict { .. }
                    | TaskResult::Change { .. } => true,
                },
                End::Parked | End::Failed(_) | End::Refused | End::Recheck => false,
            }
    }
}
