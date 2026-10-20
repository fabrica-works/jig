//! A task's next-run wait, separate from its creator preference and its committed account
//! (client/domain/contract.md, 5.1; accounts.md, 4.3).
use crate::{Core, RunWait};
use jig_domain_accounts as accounts;
use jig_domain_tasks as tasks;
use skein_lib::Wall;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TaskRunWait {
    Account { account: Option<u32>, source: Option<u32>, why: TaskWaitWhy },
    Credential,
    Host,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TaskWaitWhy {
    Source(TaskSourceState),
    Refreshing,
    Host,
    Empty,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TaskSourceState {
    Asking,
    Granted,
    Later { until: Option<Wall> },
    Limited { until: Option<Wall> },
    Refused,
    NeedsSignIn,
    Misconfigured,
    Unloaded { at: accounts::Place, why: accounts::Unread },
    Cancelled,
    Closing,
}

pub(super) fn waiting(core: &Core, row: &tasks::TaskRecord) -> Option<TaskRunWait> {
    match row.executor {
        tasks::Executor::Procedure { .. } | tasks::Executor::Person(_) => return None,
        tasks::Executor::Agent { .. } => {}
    }
    match row.phase {
        tasks::Phase::Active(tasks::Active::Due | tasks::Active::Preparing) => {}
        tasks::Phase::Active(_)
        | tasks::Phase::Waiting
        | tasks::Phase::Held { .. }
        | tasks::Phase::Closing(_)
        | tasks::Phase::Ended(_) => return None,
    }
    match core.run_choice(row.number) {
        Ok(_) if !core.credentials_ready(row.number) => Some(TaskRunWait::Credential),
        Ok(_) => None,
        Err(RunWait::Credential) => Some(TaskRunWait::Credential),
        Err(RunWait::Host) => Some(TaskRunWait::Host),
        Err(RunWait::Account { account, source, why }) => Some(TaskRunWait::Account {
            account,
            source,
            why: match why {
                accounts::WaitWhy::Refreshing => TaskWaitWhy::Refreshing,
                accounts::WaitWhy::Host => TaskWaitWhy::Host,
                accounts::WaitWhy::Empty => TaskWaitWhy::Empty,
                accounts::WaitWhy::Source(state) => TaskWaitWhy::Source(source_state(core, state)),
            },
        }),
    }
}

fn source_state(core: &Core, state: accounts::State) -> TaskSourceState {
    let clock = Some(super::DocumentClock { now: core.run_now, wall: core.run_wall });
    match state {
        accounts::State::Asking => TaskSourceState::Asking,
        accounts::State::Granted => TaskSourceState::Granted,
        accounts::State::Later { until } => {
            TaskSourceState::Later { until: super::deployment::wall_deadline(clock, until) }
        }
        accounts::State::Limited { until } => TaskSourceState::Limited {
            until: match until {
                Some(until) => super::deployment::wall_deadline(clock, until),
                None => None,
            },
        },
        accounts::State::Refused => TaskSourceState::Refused,
        accounts::State::NeedsSignIn => TaskSourceState::NeedsSignIn,
        accounts::State::Misconfigured => TaskSourceState::Misconfigured,
        accounts::State::Unloaded { at, why } => TaskSourceState::Unloaded { at, why },
        accounts::State::Cancelled => TaskSourceState::Cancelled,
        accounts::State::Closing => TaskSourceState::Closing,
    }
}
