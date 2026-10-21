//! One rule for dependencies and ending authority (domain/tasks.md, 5.7 and section 7).
use crate::domain::{Domain, record};
use crate::{Active, End, Executor, Phase, TaskResult, Was};

/// Whether an active task may go on, and why not (domain/tasks.md, 5.7).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Kept {
    /// A dependency an amendment added is pending.
    pub(crate) dependencies: bool,
    pub(crate) open: bool,
}

impl Domain {
    /// A waiting effect reads the task's current kept state (domain/tasks.md, 5.7).
    #[must_use]
    pub fn waiting_effect_problem(&self, task: u64) -> Option<crate::Problem> {
        let row = record(self, task)?;
        if for_record(row).dependencies {
            Some(crate::Problem::naming(Some(task), crate::Refusal::Kept, &row.waiting_on))
        } else {
            None
        }
    }
}

pub(crate) fn kept(domain: &Domain, task: u64) -> Kept {
    match record(domain, task) {
        Some(row) => for_record(row),
        None => pending(&[], crate::EndedBy::Result),
    }
}

/// Publication borrows its row mutably; closing keeps cancellation's partial result.
pub(crate) fn for_record(row: &crate::TaskRecord) -> Kept {
    match row.phase {
        Phase::Active(_) | Phase::Held { was: Was::Active(_), .. } => pending(&row.waiting_on, row.flags.ended_by),
        Phase::Waiting
        | Phase::Closing(_)
        | Phase::Held { was: Was::Waiting | Was::Closing(_), .. }
        | Phase::Ended(_) => pending(&[], crate::EndedBy::Result),
    }
}

/// The same facts for a compound request's projected dependency list.
pub(crate) fn pending(dependencies: &[u64], ended_by: crate::EndedBy) -> Kept {
    Kept { dependencies: !dependencies.is_empty(), open: ended_by == crate::EndedBy::Requester }
}

/// Each caller asks its own question; a new reason to keep a task answers every one.
impl Kept {
    /// The task starts no run: an agent is never prepared, claimed or Due (5.7).
    pub(crate) fn blocks_claims(self) -> bool {
        self.dependencies
    }

    /// A done end waits until every dependency is met: a person's answer is refused, a run's parks.
    pub(crate) fn refuses_done(self) -> bool {
        self.dependencies || self.open
    }

    /// No run of it may end it, so a finish grace without a live run cancels it at once.
    pub(crate) fn finishes_alone(self) -> bool {
        !self.dependencies && !self.open
    }

    /// Whether a wake may make this executor Due; a person or a procedure is never kept from its step.
    pub(crate) fn may_be_due(self, executor: Executor) -> bool {
        match executor {
            Executor::Agent { .. } => !self.blocks_claims(),
            Executor::Person(_) | Executor::Procedure { .. } => true,
        }
    }

    /// An amendment invalidates an agent's queued preparation; a committed claim stays live.
    pub(crate) fn amended(self, executor: Executor, phase: Phase) -> Phase {
        if self.may_be_due(executor) {
            return phase;
        }
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
    }

    /// Open refuses every raw finish before judgment. Dependency-only finishes
    /// keep the existing valid-done policy; failures and invalid results still end.
    pub(crate) fn parks(self, end: &End, contract: &crate::Contract, limits: &crate::Limits) -> bool {
        match end {
            End::Finished { result, .. } => {
                self.open
                    || self.dependencies && done(result) && super::invalid_result(contract, result, limits).is_none()
            }
            End::FinishedWithFollowups { result, followups } => {
                self.open
                    || self.dependencies
                        && done(result)
                        && super::invalid_followups(contract, result, followups, limits).is_none()
            }
            End::Parked | End::Failed(_) | End::Refused | End::Recheck => false,
        }
    }
}

fn done(result: &TaskResult) -> bool {
    match result {
        TaskResult::Failure { .. } => false,
        TaskResult::Approved { .. }
        | TaskResult::Report { .. }
        | TaskResult::Verdict { .. }
        | TaskResult::Change { .. } => true,
    }
}
