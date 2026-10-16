//! Ordered admission and whole application of a party's task operations (tasks.md, 7).
mod staged;
use crate::{Amendment, Domain, Funder, Limits, Numbers, Party, Problem, Refusal, Request, Word};
use alloc::boxed::Box;
use skein_lib::{Duration, Env, List, Queue, ReplyTo};

/// One authenticated party operation in a single project's whole request.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Operation {
    Cancel { task: u64, reason: Box<[u8]>, grace: Option<Duration>, message: Option<u64> },
    Move { task: u64, reason: Box<[u8]> },
    Amend { task: u64, message: u64, stop_run: bool, amendment: Box<Amendment> },
    Release { task: u64, again: Option<bool>, message: Option<u64> },
    Message { task: u64, word: Word, release: bool },
    Stop { task: u64 },
}
impl Operation {
    fn task(&self) -> u64 {
        match self {
            Self::Cancel { task, .. }
            | Self::Move { task, .. }
            | Self::Amend { task, .. }
            | Self::Release { task, .. }
            | Self::Message { task, .. }
            | Self::Stop { task } => *task,
        }
    }
}

pub(crate) enum Plan {
    Cancel(Box<crate::control::CancelPlan>),
    Move { plan: Box<crate::moving::Plan>, reason: Box<[u8]> },
    Amend(Box<crate::control::AmendPlan>),
    Release(Box<crate::control::ReleasePlan>),
    Message(Box<crate::inbox::MessagePlan>),
    Stop(Option<Box<crate::run::StopPlan>>),
}

pub(crate) struct Staged {
    pub(crate) closing: List<u64>,
    pub(crate) moved: List<u64>,
    pub(crate) reserved: u64,
    pub(crate) plans: List<Plan>,
    pub(crate) funding: List<(Funder, Numbers)>,
}
impl Staged {
    pub(crate) fn new(bound: u32) -> Self {
        Self {
            closing: List::with_capacity(bound),
            moved: List::with_capacity(bound),
            reserved: 0,
            plans: List::with_capacity(bound),
            funding: List::with_capacity(bound.saturating_mul(4)),
        }
    }
    pub(crate) fn after(&mut self, domain: &Domain, plan: &Plan) {
        match plan {
            Plan::Cancel(plan) => {
                let closing = match &plan.finish {
                    None => Some(plan.selected.as_ref()),
                    Some(crate::grace::FinishPlan::Cancel { selected, .. }) => Some(selected.as_ref()),
                    Some(
                        crate::grace::FinishPlan::Sooner { .. }
                        | crate::grace::FinishPlan::Word { .. }
                        | crate::grace::FinishPlan::Unchanged,
                    ) => None,
                };
                if let Some(closing) = closing {
                    for task in closing {
                        if !has(&self.closing, *task) {
                            self.closing.push(*task).expect("live tasks bound");
                        }
                    }
                }
            }
            Plan::Move { plan, .. } => {
                if !has(&self.moved, plan.number) {
                    self.moved.push(plan.number).expect("operation bound");
                }
                if self.numbers(domain, plan.destination).is_none() {
                    let mut period = match self.numbers(domain, plan.period_source) {
                        Some(numbers) => numbers,
                        None => Numbers { budget: plan.period_budget, spent: 0, spent_below: 0, reserved: 0 },
                    };
                    period.reserved = period.reserved.checked_add(plan.pool_budget).expect("admission checked");
                    self.funding.push((plan.period_source, period)).expect("period creation per move");
                }
                if plan.source != plan.destination {
                    let mut source = self.numbers(domain, plan.source).expect("checked source");
                    source.reserved = source.reserved.checked_sub(plan.budget).expect("source checked");
                    source.spent_below = source.spent_below.checked_add(plan.spent).expect("admission checked");
                    self.funding.push((plan.source, source)).expect("three ledgers per move");
                    let mut destination = match self.numbers(domain, plan.destination) {
                        Some(numbers) => numbers,
                        None => Numbers { budget: plan.pool_budget, spent: 0, spent_below: 0, reserved: 0 },
                    };
                    destination.reserved = destination.reserved.checked_add(plan.left).expect("admission checked");
                    self.funding.push((plan.destination, destination)).expect("three ledgers per move");
                    self.reserved = self.reserved.checked_add(plan.left).expect("admission checked");
                    let mut numbers = self.record(domain, plan.number).expect("moved task").numbers;
                    numbers.budget = plan.left;
                    numbers.spent = 0;
                    numbers.spent_below = 0;
                    self.funding.push((Funder::Task(plan.number), numbers)).expect("task normalization per move");
                }
            }
            Plan::Amend(plan) => {
                if let Some(authority) = &plan.amendment.authority {
                    let row = self.record(domain, plan.number).expect("checked task");
                    let funder = row.funder;
                    let budget = row.numbers.budget;
                    let row_numbers = row.numbers;
                    let mut numbers = self.numbers(domain, funder).expect("checked funder");
                    numbers.reserved = numbers
                        .reserved
                        .checked_sub(budget)
                        .expect("reservation checked")
                        .checked_add(authority.budget.spend)
                        .expect("resize checked");
                    self.funding.push((funder, numbers)).expect("one ledger per amendment");
                    let mut numbers = row_numbers;
                    numbers.budget = authority.budget.spend;
                    self.funding.push((Funder::Task(plan.number), numbers)).expect("task budget per amendment");
                }
            }
            Plan::Release(_) | Plan::Message(_) | Plan::Stop(_) => {}
        }
    }
}

pub(crate) struct Compound {
    pub reply_to: ReplyTo,
    pub person: u64,
    pub project: u32,
    pub operations: List<Operation>,
    pub period: u64,
    pub pool_budget: u64,
    pub period_budget: u64,
}

pub(crate) fn compound(domain: &mut Domain, env: &Env<Limits>, request: Compound, out: &mut Queue<Request>) {
    let mut staged = Staged::new(env.limits.tasks);
    let mut request = request;
    if request.operations.len() > env.limits.tasks {
        return out.push(Request::CompoundRefused {
            reply_to: request.reply_to,
            at: 0,
            problem: Problem::new(None, Refusal::Batch),
        });
    }
    let operations = core::mem::replace(&mut request.operations, List::with_capacity(0));
    for (at, operation) in operations.into_boxed().into_iter().enumerate() {
        let at = u32::try_from(at).expect("bounded operations");
        let number = operation.task();
        let checked = if request.person == 0 {
            Err(Box::new(Problem::new(Some(number), Refusal::Reference)))
        } else if !domain.ready() {
            Err(Box::new(Problem::new(Some(number), Refusal::NotReady)))
        } else if has(&staged.closing, number) {
            Err(Box::new(Problem::new(Some(number), Refusal::State)))
        } else {
            match staged.record(domain, number) {
                None => Err(Box::new(Problem::new(Some(number), Refusal::Unknown))),
                Some(row) if row.project != request.project => {
                    Err(Box::new(Problem::new(Some(number), Refusal::Project)))
                }
                Some(_) => check(domain, &staged, env, &request, operation),
            }
        };
        match checked {
            Ok(plan) => {
                staged.after(domain, &plan);
                assert!(staged.plans.push(plan).is_ok(), "operation bound");
            }
            Err(problem) => {
                return out.push(Request::CompoundRefused { reply_to: request.reply_to, at, problem: *problem });
            }
        }
    }
    let reroute = !staged.moved.is_empty();
    for plan in staged.plans.into_boxed() {
        apply(domain, env, request.person, plan, out);
    }
    if reroute {
        crate::moving::reroute(domain, env, out);
    }
    out.push(Request::Done { reply_to: request.reply_to });
}

fn check(
    domain: &Domain,
    staged: &Staged,
    env: &Env<Limits>,
    request: &Compound,
    operation: Operation,
) -> Result<Plan, Box<Problem>> {
    let by = Party::Person(request.person);
    match operation {
        Operation::Cancel { task, reason, grace, message } => Ok(Plan::Cancel(Box::new(crate::control::check_cancel(
            domain,
            staged,
            env,
            by,
            crate::control::Cancel { number: task, reason, grace, message },
        )?))),
        Operation::Amend { task, message, stop_run, amendment } => match crate::control::check_amend(
            domain,
            staged,
            env,
            by,
            crate::control::Amend { number: task, message, stop_run, amendment: *amendment },
        ) {
            Ok(plan) => Ok(Plan::Amend(Box::new(plan))),
            Err(why) => Err(Box::new(Problem::new(Some(task), why))),
        },
        Operation::Release { task, again, message } => {
            match crate::control::check_release(domain, staged, env, task, by, again, message) {
                Ok(plan) => Ok(Plan::Release(Box::new(plan))),
                Err(why) => Err(Box::new(Problem::new(Some(task), why))),
            }
        }
        Operation::Message { task, word, release } => {
            if word.from != by {
                return Err(Box::new(Problem::new(Some(task), Refusal::Reference)));
            }
            Ok(Plan::Message(Box::new(crate::inbox::check_message(
                domain,
                staged,
                env,
                crate::inbox::Message { project: request.project, number: task, word, release, authorized: true },
            )?)))
        }
        Operation::Stop { task } => {
            let plan = crate::run::check_stop(
                domain,
                staged,
                env,
                task,
                crate::Hold::StoppedBy { party: request.person },
                None,
            );
            match plan {
                Some(plan) => Ok(Plan::Stop(Some(Box::new(plan)))),
                None => Ok(Plan::Stop(None)),
            }
        }
        Operation::Move { task, reason } => match crate::moving::check(
            domain,
            staged,
            env,
            crate::moving::Move {
                number: task,
                person: request.person,
                period: request.period,
                pool_budget: request.pool_budget,
                period_budget: request.period_budget,
                reason: &reason,
            },
        ) {
            Ok(plan) => Ok(Plan::Move { plan: Box::new(plan), reason }),
            Err(problem) => Err(Box::new(problem)),
        },
    }
}

fn apply(domain: &mut Domain, env: &Env<Limits>, person: u64, plan: Plan, out: &mut Queue<Request>) {
    let by = Party::Person(person);
    match plan {
        Plan::Cancel(plan) => crate::control::apply_cancel(domain, env, by, *plan, out),
        Plan::Move { plan, reason } => crate::moving::make(domain, env, *plan, &reason, out),
        Plan::Amend(plan) => crate::control::apply_amend(domain, env, by, *plan, out),
        Plan::Release(plan) => crate::control::apply_release(domain, env, by, *plan, out),
        Plan::Message(plan) => {
            crate::inbox::apply_message(domain, env, *plan, out);
        }
        Plan::Stop(Some(plan)) => crate::run::apply_stop(domain, env, *plan, out),
        Plan::Stop(None) => {}
    }
}

pub(crate) fn has(list: &List<u64>, number: u64) -> bool {
    for entry in list {
        if *entry == number {
            return true;
        }
    }
    false
}

/// Input operations, owned plans and their bounded field projections.
pub(crate) fn worst_case(limits: &Limits, payload: u64) -> Option<u64> {
    let tasks = limits.tasks;
    List::<Plan>::worst_case(tasks)?
        .checked_add(List::<Operation>::worst_case(tasks)?)?
        .checked_add(List::<u64>::worst_case(tasks)?.checked_mul(2)?)?
        .checked_add(List::<(Funder, Numbers)>::worst_case(tasks.checked_mul(4)?)?)?
        .checked_add(
            u64::from(tasks).checked_mul(
                u64::try_from(size_of::<crate::control::AmendPlan>())
                    .ok()?
                    .checked_add(payload.checked_mul(4)?)?
                    .checked_add(List::<u64>::worst_case(tasks)?.checked_mul(2)?)?,
            )?,
        )?
        .checked_add(payload)
}
