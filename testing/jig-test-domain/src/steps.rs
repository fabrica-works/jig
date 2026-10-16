//! Reference connector procedure decisions translated into the core's atomic step.
use crate::{Domain, Work};
use alloc::boxed::Box;
use jig_domain_core as core;
use jig_domain_tasks as tasks;
use jig_test_connector as connector;
use skein_lib::{Decimal, List, Queue, Writer};

pub(crate) fn translate(
    domain: &mut Domain,
    task: u64,
    step: u64,
    number: u16,
    decision: connector::StepDecision,
    work: &mut Queue<Work>,
) -> Option<core::ProcedureStep> {
    let end = match decision {
        connector::StepDecision::Finish { result: connector::ProcedureResult::Succeeded } => {
            core::StepEnd::Result(tasks::TaskResult::Report { words: Box::new([]) })
        }
        connector::StepDecision::Finish { result: connector::ProcedureResult::Local { code } } => {
            core::StepEnd::Result(tasks::TaskResult::Change {
                connector: number,
                kind: code,
                resource: 0,
                words: Box::new([]),
            })
        }
        connector::StepDecision::Wait { .. } | connector::StepDecision::Stall => core::StepEnd::Wait,
        connector::StepDecision::Hold { .. } => core::StepEnd::Hold(tasks::Hold::Effects),
        connector::StepDecision::Propose { .. } => core::StepEnd::Hold(tasks::Hold::Procedure),
        connector::StepDecision::Delegate { kinds } => {
            let row = domain.core.tasks.task(task).expect("procedure task live");
            let capacity = u32::try_from(kinds.len()).expect("bounded program");
            let mut batch = List::with_capacity(capacity);
            let budget = row
                .numbers
                .budget
                .saturating_sub(row.numbers.spent)
                .saturating_sub(row.numbers.spent_below)
                .saturating_sub(row.numbers.reserved)
                .checked_div(u64::from(capacity))
                .unwrap_or(0);
            for kind in kinds {
                let mut authority = row.authority.clone();
                authority.budget.spend = budget;
                authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
                batch
                    .push(core::Delegate {
                        executor: tasks::Executor::Agent { charter: u32::from(kind) },
                        spec: tasks::Spec { words: words(task, step), parameters: Box::new([]), inputs: Box::new([]) },
                        contract: tasks::Contract::Report { words: 128 },
                        authority,
                        symbolic_grants: Box::new([]),
                        dependencies: Box::new([]),
                        wake: tasks::WakePolicy::DEFAULT,
                    })
                    .expect("one delegate per program kind");
            }
            return Some(core::ProcedureStep {
                batch: batch.into_boxed(),
                ..core::ProcedureStep::new(core::StepEnd::Wait)
            });
        }
        connector::StepDecision::Effect(effect) => {
            let owner = crate::effect_payload(domain, effect);
            domain.effect_procedures.insert(owner, task).expect("one procedure effect flight");
            work.push(Work::Core(core::Event::EffectStart {
                owner,
                connector: number,
                origin: core::EffectOrigin::Procedure { task, step, entry: None },
            }));
            return None;
        }
    };
    Some(core::ProcedureStep::new(end))
}

pub(crate) fn refused(number: u16, task: u64, step: u64, why: core::StepRefusal, work: &mut Queue<Work>) {
    let why = match why {
        core::StepRefusal::Moved { now: jig_domain_people::Moved::Spec { revision }, .. } => {
            connector::StepRefusal::Moved { revision }
        }
        core::StepRefusal::Moved { .. } | core::StepRefusal::Refused(_) | core::StepRefusal::Denied { .. } => {
            connector::StepRefusal::Refused
        }
    };
    work.push(Work::Connector { number, event: connector::Event::StepRefused { task, step, why } });
}

fn words(task: u64, step: u64) -> Box<[u8]> {
    let task = Decimal::of(task);
    let step = Decimal::of(step);
    let mut words = Writer::new(56);
    words.put(b"procedure ").expect("bounded label");
    words.put(task.as_bytes()).expect("bounded task number");
    words.put(b" step ").expect("bounded label");
    words.put(step.as_bytes()).expect("bounded step number");
    words.finish()
}
