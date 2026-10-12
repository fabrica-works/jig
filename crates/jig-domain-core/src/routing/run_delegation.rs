//! Run delegation, procedure batches and recurring work routed through tasks
//! and numbered connector holdings (`domain/engine.md`, sections 4.4 and 7).

use super::HoldingsNeed;

use super::runs::{call_decision, named_propose, proposal_refused};
use super::{BatchKind, DelegateCreation, ProcedureCreation};
use crate::{
    Ask, CallKey, CallPart, Core, Delegate, Dependency, Event, Limits, Now, ProcedureAction, Request, Requests,
    ToolKind,
};
use alloc::boxed::Box;
use jig_domain_tasks as tasks;
use skein_lib::{Env, List, Queue, ReplyTo, Token};

pub(super) fn delegate_refused(
    core: &mut Core,
    to: ReplyTo,
    key: CallKey,
    task: Option<u64>,
    why: tasks::Refusal,
) -> Requests {
    let mut out = Queue::with_capacity(3);
    call_decision(
        core,
        &mut out,
        to,
        key,
        CallPart::DelegationRefused(tasks::Problem { task, why, blocked_by: None, capacity: None }),
    );
    out.push(Request::Decided);
    Requests::Out(out)
}

#[expect(
    clippy::too_many_lines,
    reason = "one bounded delegated batch receives task identities and connector handoffs"
)]
pub(super) fn delegate_validated(
    core: &mut Core,
    env: &Env<Limits>,
    to: ReplyTo,
    key: CallKey,
    batch: Box<[Delegate]>,
    stubs: Box<[tasks::Stub]>,
) -> Requests {
    if let Some(part) = core.authorize_tool(key, ToolKind::Delegate) {
        let mut out = Queue::with_capacity(3);
        call_decision(core, &mut out, to, key, part);
        out.push(Request::Decided);
        return Requests::Out(out);
    }
    if let Err(part) = core.delegate_preflight(&env.limits.tasks, key, &batch) {
        let mut out = Queue::with_capacity(3);
        call_decision(core, &mut out, to, key, part);
        out.push(Request::Decided);
        return Requests::Out(out);
    }
    let context = core.tasks.delegation(key.task).expect("preflighted delegator");
    let root = core.tasks.root(key.task).expect("delegator live");
    let mut numbers = List::with_capacity(env.limits.tasks.batch);
    for _ in &batch {
        let Some(number) = crate::fresh(&mut core.counters, crate::Family::Task) else {
            return delegate_refused(core, to, key, None, tasks::Refusal::Live);
        };
        numbers.push(number).expect("bounded delegation IDs");
    }
    let mut created = List::with_capacity(env.limits.tasks.batch);
    for (index, member) in batch.into_iter().enumerate() {
        let index = u32::try_from(index).expect("batch length fits u32");
        let mut dependencies = List::with_capacity(env.limits.tasks.dependencies);
        for dependency in member.dependencies {
            let number = match dependency {
                Dependency::Batch(at) => match numbers.get(at) {
                    Some(number) => *number,
                    None => {
                        return delegate_refused(
                            core,
                            to,
                            key,
                            numbers.get(index).copied(),
                            tasks::Refusal::Dependencies,
                        );
                    }
                },
                Dependency::Existing(number) => number,
            };
            if dependencies.push(number).is_err() {
                return delegate_refused(core, to, key, numbers.get(index).copied(), tasks::Refusal::Dependencies);
            }
        }
        let number = *numbers.get(index).expect("one ID per member");
        let Some(authority) = crate::resolved_delegate_authority(
            &member.authority,
            &member.symbolic_grants,
            number,
            core.authority.limits(),
        ) else {
            return delegate_refused(core, to, key, Some(number), tasks::Refusal::AuthorityShape);
        };
        created
            .push(tasks::New {
                number,
                project: context.project,
                executor: member.executor,
                spec: member.spec,
                contract: member.contract,
                authority,
                numbers: tasks::Numbers {
                    budget: member.authority.budget.spend,
                    spent: 0,
                    spent_below: 0,
                    reserved: 0,
                },
                funder: tasks::Funder::Task(key.task),
                dependencies: dependencies.into_boxed(),
                holdings: Box::new([]),
                wake: member.wake,
                recurring: None,
                tracked: None,
            })
            .expect("bounded delegation batch");
    }
    let request = to.into_token();
    let members = created.into_boxed();
    if !core.pending_calls.contains_key(&key) {
        assert!(core.pending_calls.insert(key, true) == Ok(None), "delegation call record room reserved");
    }
    let remaining = u32::try_from(core.connectors.len()).expect("configured connector count fits u32");
    assert!(remaining > 0, "delegation has a numbered connector");
    let mut asked = List::with_capacity(u32::try_from(members.len()).expect("bounded delegation batch"));
    for member in &members {
        asked
            .push(HoldingsNeed {
                project: member.project,
                root,
                number: member.number,
                executor: member.executor,
                spec: member.spec.clone(),
            })
            .expect("one holdings request per member");
    }
    assert!(
        core.creating_delegates
            .insert(
                request,
                DelegateCreation {
                    key,
                    kind: BatchKind::Delegation { stubs },
                    members,
                    remaining,
                    answered: List::with_capacity(remaining),
                    failed: false,
                }
            )
            .is_ok(),
        "one named delegation flight"
    );
    let mut out = Queue::with_capacity(remaining.checked_add(1).expect("connector handoff room"));
    for &connector in core.connectors.as_ref() {
        out.push(Request::Ask {
            connector,
            ask: Ask::DelegateHoldings { request, from: key.task, members: asked.as_slice().into() },
        });
    }
    out.push(Request::Decided);
    Requests::Out(out)
}

pub(super) fn delegate_holdings(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    request: Token,
    connector: u16,
    holdings: Option<Box<[Box<[tasks::Holding]>]>>,
) -> Requests {
    let Some(flight) = core.creating_delegates.get_mut(&request) else {
        let mut out = Queue::with_capacity(1);
        out.push(Request::Decided);
        return Requests::Out(out);
    };
    assert!(flight.remaining > 0, "one holdings answer per connector");
    assert!(
        core.connectors.contains(&connector) && !flight.answered.as_slice().contains(&connector),
        "one answer per connector"
    );
    flight.answered.push(connector).expect("configured connector bound");
    flight.remaining = flight.remaining.checked_sub(1).expect("one connector answer");
    match holdings {
        Some(holdings) if holdings.len() == flight.members.len() => {
            for (index, added) in holdings.into_iter().enumerate() {
                let member = flight.members.get_mut(index).expect("one answer per member");
                let mut whole = List::with_capacity(env.limits.tasks.holdings);
                for holding in member.holdings.iter().chain(added.iter()) {
                    if whole.push(holding.clone()).is_err() {
                        flight.failed = true;
                    }
                }
                member.holdings = whole.into_boxed();
            }
        }
        Some(_) | None => flight.failed = true,
    }
    if flight.remaining > 0 {
        let mut out = Queue::with_capacity(1);
        out.push(Request::Decided);
        return Requests::Out(out);
    }
    let flight = core.creating_delegates.remove(&request).expect("completed delegate holdings");
    if flight.failed {
        return match flight.kind {
            BatchKind::Delegation { .. } => {
                delegate_refused(core, ReplyTo::new(request), flight.key, None, tasks::Refusal::Holds)
            }
            BatchKind::Proposal { .. } => {
                proposal_refused(core, ReplyTo::new(request), flight.key, tasks::Refusal::Holds)
            }
        };
    }
    match flight.kind {
        BatchKind::Delegation { stubs } => {
            assert!(core.delegating.insert(request, (flight.key, stubs)) == Ok(None), "one pending delegated call");
            work.push(Event::Tasks(tasks::Event::Make {
                reply_to: ReplyTo::new(request),
                creator: tasks::Party::Task(flight.key.task),
                batch: flight.members,
            }));
            let mut out = Queue::with_capacity(1);
            out.push(Request::Decided);
            Requests::Out(out)
        }
        BatchKind::Proposal { reason, as_holder } => named_propose(
            core,
            env,
            work,
            ReplyTo::new(request),
            flight.key,
            tasks::ProposalAction::Batch(flight.members),
            reason,
            as_holder,
        ),
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "one proposal route carries its tracked priority and accepting-holder choice"
)]
pub(super) fn propose_batch(
    core: &mut Core,
    env: &Env<Limits>,
    to: ReplyTo,
    key: CallKey,
    batch: Box<[Delegate]>,
    reason: Box<[u8]>,
    as_holder: bool,
    tracked: Option<u32>,
) -> Requests {
    if reason.len() > usize::try_from(env.limits.tasks.message_bytes).expect("u32 fits usize") {
        return proposal_refused(core, to, key, tasks::Refusal::Read);
    }
    let Some(context) = core.tasks.delegation(key.task) else {
        return proposal_refused(core, to, key, tasks::Refusal::Unknown);
    };
    if batch.is_empty() || batch.len() > usize::try_from(env.limits.tasks.batch).expect("u32 fits usize") {
        return proposal_refused(core, to, key, tasks::Refusal::Batch);
    }
    let root = core.tasks.root(key.task).expect("proposer live");
    let mut numbers = List::with_capacity(env.limits.tasks.batch);
    for _ in &batch {
        let Some(number) = crate::fresh(&mut core.counters, crate::Family::Task) else {
            return proposal_refused(core, to, key, tasks::Refusal::Live);
        };
        numbers.push(number).expect("bounded proposal task IDs");
    }
    let mut created = List::with_capacity(env.limits.tasks.batch);
    for (index, member) in batch.into_iter().enumerate() {
        let mut dependencies = List::with_capacity(env.limits.tasks.dependencies);
        for dependency in member.dependencies {
            let number = match dependency {
                Dependency::Batch(at) => match numbers.get(at) {
                    Some(number) => *number,
                    None => return proposal_refused(core, to, key, tasks::Refusal::Dependencies),
                },
                Dependency::Existing(number) => number,
            };
            if dependencies.push(number).is_err() {
                return proposal_refused(core, to, key, tasks::Refusal::Dependencies);
            }
        }
        let number = *numbers.get(u32::try_from(index).expect("batch index")).expect("one ID per member");
        created
            .push(tasks::New {
                number,
                project: context.project,
                executor: member.executor,
                spec: member.spec,
                contract: member.contract,
                numbers: tasks::Numbers {
                    budget: member.authority.budget.spend,
                    spent: 0,
                    spent_below: 0,
                    reserved: 0,
                },
                authority: member.authority,
                funder: tasks::Funder::Task(key.task),
                dependencies: dependencies.into_boxed(),
                holdings: Box::new([]),
                wake: member.wake,
                recurring: None,
                tracked,
            })
            .expect("bounded proposal batch");
    }
    let request = to.into_token();
    let members = created.into_boxed();
    let remaining = u32::try_from(core.connectors.len()).expect("configured connector count fits u32");
    assert!(remaining > 0, "proposal has a numbered connector");
    let mut asked = List::with_capacity(u32::try_from(members.len()).expect("bounded proposal batch"));
    for member in &members {
        asked
            .push(HoldingsNeed {
                project: member.project,
                root,
                number: member.number,
                executor: member.executor,
                spec: member.spec.clone(),
            })
            .expect("one holdings request per member");
    }
    assert!(
        core.creating_delegates
            .insert(
                request,
                DelegateCreation {
                    key,
                    kind: BatchKind::Proposal { reason, as_holder },
                    members,
                    remaining,
                    answered: List::with_capacity(remaining),
                    failed: false,
                }
            )
            .is_ok(),
        "one proposal holdings flight"
    );
    let mut out = Queue::with_capacity(remaining.checked_add(1).expect("connector handoff room"));
    for &connector in core.connectors.as_ref() {
        out.push(Request::Ask {
            connector,
            ask: Ask::DelegateHoldings { request, from: key.task, members: asked.as_slice().into() },
        });
    }
    out.push(Request::Decided);
    Requests::Out(out)
}

pub(super) fn procedure_refused(task: u64, step: u64) -> Requests {
    let mut out = Queue::with_capacity(2);
    out.push(Request::Now(Box::new(Now::ProcedureDelegateOutcome { task, step, child: None })));
    out.push(Request::Decided);
    Requests::Out(out)
}

#[expect(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    reason = "one procedure batch carries its owner and bounded connector handoffs"
)]
pub(super) fn procedure_step(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    task: u64,
    step: u64,
    connector: u16,
    procedure_code: u32,
    action: ProcedureAction,
) -> Requests {
    let mut out = Queue::with_capacity(
        u32::try_from(core.connectors.len())
            .expect("bounded connectors")
            .checked_add(1)
            .expect("procedure handoff room"),
    );
    if core.tasks.procedure_due(task) != Some((connector, procedure_code, step)) {
        if match &action {
            ProcedureAction::Delegate(_) => true,
            ProcedureAction::Result(_) | ProcedureAction::Hold(_) | ProcedureAction::Wait => false,
        } {
            return procedure_refused(task, step);
        }
        out.push(Request::Decided);
        return Requests::Out(out);
    }
    match action {
        ProcedureAction::Delegate(batch) => {
            if !core.procedure_batch_admit(&env.limits.tasks, task, &batch) {
                return procedure_refused(task, step);
            }
            let context = core.tasks.delegation(task).expect("admitted procedure owner");
            let root = core.tasks.root(task).expect("procedure task live");
            let mut numbers = List::with_capacity(env.limits.tasks.batch);
            for _ in &batch {
                let Some(number) = crate::fresh(&mut core.counters, crate::Family::Task) else {
                    return procedure_refused(task, step);
                };
                numbers.push(number).expect("bounded procedure task IDs");
            }
            let mut created = List::with_capacity(env.limits.tasks.batch);
            for (index, member) in batch.into_iter().enumerate() {
                let at = u32::try_from(index).expect("bounded batch index");
                let mut dependencies = List::with_capacity(env.limits.tasks.dependencies);
                for dependency in member.dependencies {
                    let number = match dependency {
                        Dependency::Batch(index) => match numbers.get(index) {
                            Some(number) => *number,
                            None => {
                                return procedure_refused(task, step);
                            }
                        },
                        Dependency::Existing(number) => number,
                    };
                    if dependencies.push(number).is_err() {
                        return procedure_refused(task, step);
                    }
                }
                let number = *numbers.get(at).expect("one ID per member");
                let Some(authority) = crate::resolved_delegate_authority(
                    &member.authority,
                    &member.symbolic_grants,
                    number,
                    core.authority.limits(),
                ) else {
                    return procedure_refused(task, step);
                };
                created
                    .push(tasks::New {
                        number,
                        project: context.project,
                        executor: member.executor,
                        spec: member.spec,
                        contract: member.contract,
                        authority,
                        numbers: tasks::Numbers {
                            budget: member.authority.budget.spend,
                            spent: 0,
                            spent_below: 0,
                            reserved: 0,
                        },
                        funder: tasks::Funder::Task(task),
                        dependencies: dependencies.into_boxed(),
                        holdings: Box::new([]),
                        wake: member.wake,
                        recurring: None,
                        tracked: None,
                    })
                    .expect("bounded procedure batch");
            }
            let members = created.into_boxed();
            let remaining = u32::try_from(core.connectors.len()).expect("configured connector count fits u32");
            assert!(remaining > 0, "procedure has a numbered connector");
            let mut asked = List::with_capacity(u32::try_from(members.len()).expect("bounded procedure batch"));
            for member in &members {
                asked
                    .push(HoldingsNeed {
                        project: member.project,
                        root,
                        number: member.number,
                        executor: member.executor,
                        spec: member.spec.clone(),
                    })
                    .expect("one holdings request per member");
            }
            assert!(
                core.creating_procedures
                    .insert(
                        (task, step),
                        ProcedureCreation {
                            members,
                            remaining,
                            answered: List::with_capacity(remaining),
                            failed: false,
                        }
                    )
                    .is_ok(),
                "one procedure holdings flight"
            );
            for &connector in core.connectors.as_ref() {
                out.push(Request::Ask {
                    connector,
                    ask: Ask::ProcedureHoldings { task, step, members: asked.as_slice().into() },
                });
            }
        }
        ProcedureAction::Result(result) => work.push(Event::Tasks(tasks::Event::Procedure {
            reply_to: ReplyTo::new(Token::new(u64::MAX)),
            task,
            step,
            read: core.tasks.procedure_offered(task),
            decision: tasks::ProcedureDecision::Result(result),
        })),
        ProcedureAction::Hold(why) => work.push(Event::Tasks(tasks::Event::Procedure {
            reply_to: ReplyTo::new(Token::new(u64::MAX)),
            task,
            step,
            read: core.tasks.procedure_offered(task),
            decision: tasks::ProcedureDecision::Hold(why),
        })),
        ProcedureAction::Wait => work.push(Event::Tasks(tasks::Event::Procedure {
            reply_to: ReplyTo::new(Token::new(u64::MAX)),
            task,
            step,
            read: core.tasks.procedure_offered(task),
            decision: tasks::ProcedureDecision::Wait,
        })),
    }
    out.push(Request::Decided);
    Requests::Out(out)
}

#[expect(clippy::manual_map, reason = "step code does not use closures")]
pub(super) fn procedure_holdings(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    task: u64,
    step: u64,
    connector: u16,
    holdings: Option<Box<[Box<[tasks::Holding]>]>>,
) -> Requests {
    let Some(flight) = core.creating_procedures.get_mut(&(task, step)) else {
        let mut out = Queue::with_capacity(1);
        out.push(Request::Decided);
        return Requests::Out(out);
    };
    assert!(flight.remaining > 0, "one procedure answer per connector");
    assert!(
        core.connectors.contains(&connector) && !flight.answered.as_slice().contains(&connector),
        "one answer per connector"
    );
    flight.answered.push(connector).expect("configured connector bound");
    flight.remaining = flight.remaining.checked_sub(1).expect("one connector answer");
    match holdings {
        Some(holdings) if holdings.len() == flight.members.len() => {
            for (index, added) in holdings.into_iter().enumerate() {
                let member = flight.members.get_mut(index).expect("one answer per member");
                let mut whole = List::with_capacity(env.limits.tasks.holdings);
                for holding in member.holdings.iter().chain(added.iter()) {
                    if whole.push(holding.clone()).is_err() {
                        flight.failed = true;
                    }
                }
                member.holdings = whole.into_boxed();
            }
        }
        Some(_) | None => flight.failed = true,
    }
    let mut out = Queue::with_capacity(2);
    if flight.remaining == 0 {
        let flight = core.creating_procedures.remove(&(task, step)).expect("completed procedure holdings");
        let child = if flight.failed {
            None
        } else {
            match flight.members.first() {
                Some(member) => Some(member.number),
                None => None,
            }
        };
        out.push(Request::Now(Box::new(Now::ProcedureDelegateOutcome { task, step, child })));
        if !flight.failed {
            work.push(Event::Tasks(tasks::Event::Procedure {
                reply_to: ReplyTo::new(Token::new(u64::MAX)),
                task,
                step,
                read: core.tasks.procedure_offered(task),
                decision: tasks::ProcedureDecision::Delegate(flight.members),
            }));
        }
    }
    out.push(Request::Decided);
    Requests::Out(out)
}

pub(super) fn start_recurring(
    core: &mut Core,
    work: &mut Queue<Event>,
    project: u32,
    authority: tasks::Authority,
    template: tasks::RecurringTemplate,
) {
    if !core.recurring_admit(project, &authority, &template) {
        return;
    }
    let period = core.settings.period;
    let to = ReplyTo::new(Token::new(u64::MAX - 2));
    if core.tasks.funding(tasks::Funder::Period { project, period }).is_none() {
        work.push(Event::Tasks(tasks::Event::OpenPeriod {
            reply_to: ReplyTo::new(Token::new(u64::MAX - 2)),
            project,
            period,
            budget: core.settings.period_budget,
        }));
    }
    let Some(number) = crate::fresh(&mut core.counters, crate::Family::Task) else { return };
    work.push(Event::Tasks(tasks::Event::Make {
        reply_to: to,
        creator: tasks::Party::Deployment { project },
        batch: Box::new([tasks::New {
            number,
            project,
            executor: tasks::Executor::Procedure { connector: core.settings.recurring_connector, code: 1 },
            spec: tasks::Spec { words: b"recurring".as_slice().into(), parameters: Box::new([]), inputs: Box::new([]) },
            contract: tasks::Contract::Report { words: 0 },
            numbers: tasks::Numbers { budget: 0, spent: 0, spent_below: 0, reserved: 0 },
            authority,
            funder: tasks::Funder::Period { project, period },
            dependencies: Box::new([]),
            holdings: Box::new([]),
            wake: tasks::WakePolicy::DEFAULT,
            recurring: Some(Box::new(template)),
            tracked: None,
        }]),
    }));
    work.push(Event::Tasks(tasks::Event::TickRecurring { task: number, period }));
}

pub(super) fn route_period(
    core: &mut Core,
    work: &mut Queue<Event>,
    project: u32,
    period: u64,
    budget: u64,
    next: Option<skein_lib::Wall>,
) -> Requests {
    let allowed = match core.authority.policy(project) {
        Some(policy) => budget <= policy.period_spend,
        None => false,
    };
    if period > 0 && period >= core.settings.period && allowed {
        if period != core.settings.period {
            core.touch_frames();
        }
        core.settings.period = period;
        core.settings.period_budget = budget;
        if core.tasks.funding(tasks::Funder::Period { project, period }).is_none() {
            work.push(Event::Tasks(tasks::Event::OpenPeriod {
                reply_to: ReplyTo::new(Token::new(u64::MAX - 2)),
                project,
                period,
                budget,
            }));
        }
        work.push(Event::Tasks(tasks::Event::PeriodNext { project, period, next }));
        for task in core.tasks.recurring_tasks(project) {
            work.push(Event::Tasks(tasks::Event::TickRecurring { task, period }));
        }
        for task in core.tasks.standing_tasks(project) {
            work.push(Event::Tasks(tasks::Event::RenewStanding { task, period }));
        }
    }
    let mut out = Queue::with_capacity(1);
    out.push(Request::Decided);
    Requests::Out(out)
}

pub(super) fn route_start_recurring(
    core: &mut Core,
    work: &mut Queue<Event>,
    project: u32,
    authority: tasks::Authority,
    template: tasks::RecurringTemplate,
) -> Requests {
    start_recurring(core, work, project, authority, template);
    let mut out = Queue::with_capacity(1);
    out.push(Request::Decided);
    Requests::Out(out)
}

pub(super) fn delegate_input_failed(work: &mut Queue<Event>, to: ReplyTo, key: CallKey) -> Requests {
    work.push(Event::NamedAnswer {
        to,
        key,
        part: CallPart::DelegationRefused(tasks::Problem {
            task: Some(key.task),
            why: tasks::Refusal::Inputs,
            blocked_by: None,
            capacity: None,
        }),
    });
    let mut out = Queue::with_capacity(1);
    out.push(Request::Decided);
    Requests::Out(out)
}
