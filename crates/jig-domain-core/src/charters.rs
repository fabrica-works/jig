//! Configured charters and readiness (domain/engine.md, 7.1 and 8).

use alloc::boxed::Box;
use jig_domain_fleet as fleet;
use jig_domain_tasks as tasks;
use skein_lib::{List, Queue};

use crate::{Core, Limits, RunPolicy, Settings, memory, run};

/// One numbered charter offered by the deployment (protocol/configuration.md, 3).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Charter {
    pub number: u32,
    pub policy: RunPolicy,
    pub hosts: fleet::Kinds,
    pub spend: u64,
}

impl Charter {
    /// The account shared by this charter's models (domain/engine.md, 7.1).
    #[must_use]
    pub const fn account(&self) -> u32 {
        self.policy.model.account
    }
}

pub(crate) fn assert_configured(settings: &Settings, limits: &Limits) {
    assert!(settings.charters.len() <= usize::try_from(limits.tasks.charters).expect("charter count"), "charter count");
    let mut seen = List::with_capacity(limits.tasks.charters);
    for charter in &settings.charters {
        assert!(charter.number != 0 && !seen.as_slice().contains(&charter.number), "unique nonzero charter number");
        seen.push(charter.number).expect("configured charter count");
        for model in &charter.policy.alternatives {
            assert_eq!(model.account, charter.account(), "one account per charter");
        }
        let policy = &charter.policy;
        assert!(
            policy.turns > policy.reserve_turns && policy.time > policy.reserve_time,
            "charter budget above reserve"
        );
        assert!(
            match run::reserve_floor(policy) {
                Some(floor) => charter.spend >= floor,
                None => false,
            },
            "charter default spend holds reserve floor"
        );
        assert!(
            match memory::policy_owned_bytes(policy) {
                Some(bytes) => bytes <= u64::from(limits.run_bytes),
                None => false,
            },
            "core prices its configured run policy"
        );
    }
}

pub(crate) fn numbers(settings: &Settings, limits: &Limits) -> Box<[u32]> {
    let mut numbers = List::with_capacity(limits.tasks.charters);
    for charter in &settings.charters {
        numbers.push(charter.number).expect("configured charter count");
    }
    numbers.into_boxed()
}

impl Core {
    pub(crate) fn claim_done(&mut self, task: u64, attempt: u64, out: &mut Queue<crate::Request>) {
        drop(self.replaced.remove(&task));
        let proof = self.proofs.get(&task).expect("claim proof pre-reserved");
        out.push(crate::Request::Write(crate::Write::Save(crate::Record::Core(crate::CoreRecord::RunProof(
            proof.clone(),
        )))));
        let kinds = match proof.host {
            fleet::HostKind::Engine => fleet::Kinds::Engine,
            fleet::HostKind::Worker => fleet::Kinds::Workers,
        };
        let charter = self.charter_of(task).number;
        for &connector in self.connectors.as_ref() {
            out.push(crate::Request::Ask { connector, ask: crate::Ask::ClaimDone { task, attempt, kinds, charter } });
        }
    }

    /// Look up a configured charter by number.
    #[must_use]
    #[expect(clippy::manual_find, reason = "the strict subset iterates without a closure")]
    pub fn charter(&self, number: u32) -> Option<&Charter> {
        for charter in &self.settings.charters {
            if charter.number == number {
                return Some(charter);
            }
        }
        None
    }

    /// An admitted live agent task's configured charter.
    #[must_use]
    pub fn charter_of(&self, task: u64) -> &Charter {
        let charter = match self.tasks.task(task).expect("live agent task").executor {
            tasks::Executor::Agent { charter } => self.charter(charter),
            tasks::Executor::Procedure { .. } | tasks::Executor::Person(_) => None,
        };
        charter.expect("agent task has configured charter")
    }

    fn context_ready(&self, context: &tasks::RunContext) -> bool {
        match context.executor {
            tasks::Executor::Agent { charter } => {
                self.accounts.usable(self.charter(charter).expect("task charter").account())
                    && self.credentials_ready(context.task)
            }
            tasks::Executor::Procedure { .. } => true,
            tasks::Executor::Person(_) => false,
        }
    }

    /// Take the first due task whose executor can start, keeping the others in order.
    pub fn take_due(&mut self) -> Option<Box<tasks::RunContext>> {
        let mut taken = None;
        for _ in 0..self.due.len() {
            let context = self.due.pop().expect("due count");
            if taken.is_none() && self.context_ready(&context) {
                taken = Some(context);
            } else {
                self.due.push(context);
            }
        }
        taken
    }

    /// Whether a due procedure or agent with a usable account can start.
    #[must_use]
    pub fn due_ready(&self) -> bool {
        for context in &self.due {
            if self.context_ready(context) {
                return true;
            }
        }
        false
    }
}
