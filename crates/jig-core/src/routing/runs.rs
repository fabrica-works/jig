//! Runs, named calls, delegation and preparation across the core's children
//! (`domain/engine.md`, sections 4.4 and 7).

use super::{
    Ask, BatchKind, DelegateCreation, EscalationChoice, Event, Held, HoldingsNeed, Limits, NamedAction, NoteRoute, Now,
    PayloadRefusal, ProcedureCreation, ProposalChoice, Request, Requests, ToolKind, Write, remember_due,
};
use crate::{
    CallKey, CallPart, CallRecord, Core, CoreKey, CoreRecord, Delegate, Dependency, Key, ProcedureAction, Record,
    RoutedCall, RunCharter, RunProof,
};
use skein_lib::{Env, List, Queue, ReplyTo, Token};

use alloc::boxed::Box;
use jig_core_authority as authority;
use jig_core_brief as brief;
use jig_core_fleet as fleet;
use jig_core_notes as notes;
use jig_core_tasks as tasks;

pub(super) fn call_decision(core: &mut Core, out: &mut Queue<Request>, to: ReplyTo, key: CallKey, part: CallPart) {
    if core.decide_named_call(key, part.clone()) {
        out.push(Request::Write(Write::Save(Record::Core(CoreRecord::Call(CallRecord {
            key,
            part: part.clone(),
            settled: core.call_settled.get(&key).cloned(),
        })))));
    }
    out.push(Request::Held(Box::new(Held::CallAnswer { to, key, part })));
}

pub(super) fn escalation_refused(
    core: &mut Core,
    to: ReplyTo,
    key: CallKey,
    task: u64,
    why: tasks::Refusal,
) -> Requests {
    let mut out = Queue::with_capacity(3);
    call_decision(
        core,
        &mut out,
        to,
        key,
        CallPart::EscalationRefused(tasks::Problem { task: Some(task), why, blocked_by: None }),
    );
    out.push(Request::Decided);
    Requests::Out(out)
}

pub(super) fn proposal_refused(core: &mut Core, to: ReplyTo, key: CallKey, why: tasks::Refusal) -> Requests {
    let mut out = Queue::with_capacity(3);
    call_decision(
        core,
        &mut out,
        to,
        key,
        CallPart::ProposalRefused(tasks::Problem { task: Some(key.task), why, blocked_by: None }),
    );
    out.push(Request::Decided);
    Requests::Out(out)
}

pub(super) fn delegate_refused(
    core: &mut Core,
    to: ReplyTo,
    key: CallKey,
    task: Option<u64>,
    why: tasks::Refusal,
) -> Requests {
    let mut out = Queue::with_capacity(3);
    call_decision(core, &mut out, to, key, CallPart::DelegationRefused(tasks::Problem { task, why, blocked_by: None }));
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

pub(super) fn brief_assembled(core: &mut Core, work: &mut Queue<Event>, task: u64, ready: bool) -> Requests {
    let mut out = Queue::with_capacity(2);
    if !ready {
        work.push(Event::PreparationFailed { task });
        out.push(Request::Now(Box::new(Now::RunPreparationFailed { task })));
        out.push(Request::Decided);
        return Requests::Out(out);
    }
    let Some(context) = core.contexts.get(&task).cloned() else {
        out.push(Request::Now(Box::new(Now::RunPreparationFailed { task })));
        out.push(Request::Decided);
        return Requests::Out(out);
    };
    let Some(attempt) = crate::fresh(&mut core.counters, crate::Family::Run) else {
        work.push(Event::Tasks(tasks::Event::PreparationFailed { task }));
        out.push(Request::Now(Box::new(Now::RunPreparationFailed { task })));
        out.push(Request::Decided);
        return Requests::Out(out);
    };
    assert!(core.workspace_pending.insert(task, attempt) == Ok(None), "one workspace flight per task");
    out.push(Request::Now(Box::new(Now::WorkspaceRequest { task, attempt, context })));
    out.push(Request::Decided);
    Requests::Out(out)
}

pub(super) fn workspace_prepared(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    task: u64,
    attempt: u64,
    writes: Option<Box<[authority::Write]>>,
) -> Requests {
    let mut out = Queue::with_capacity(2);
    if core.workspace_pending.remove(&task) != Some(attempt) {
        out.push(Request::Now(Box::new(Now::RunPreparationFailed { task })));
        out.push(Request::Decided);
        return Requests::Out(out);
    }
    let Some(context) = core.contexts.remove(&task) else {
        out.push(Request::Now(Box::new(Now::RunPreparationFailed { task })));
        out.push(Request::Decided);
        return Requests::Out(out);
    };
    drop(core.dependency_results.remove(&task));
    let transcript = core.transcripts.remove(&task).expect("rendered task owns loaded transcript");
    let Some(writes) = writes else {
        work.push(Event::Tasks(tasks::Event::Hold { task, why: tasks::Hold::Effects }));
        out.push(Request::Now(Box::new(Now::RunPreparationFailed { task })));
        out.push(Request::Decided);
        return Requests::Out(out);
    };
    let why = match core.run_admission(&context, env.wall, writes) {
        crate::RunAdmission::Allow => None,
        crate::RunAdmission::Account => Some(tasks::Hold::Effects),
        crate::RunAdmission::Hold(why) => Some(why),
    };
    if let Some(why) = why {
        work.push(Event::Tasks(tasks::Event::Hold { task, why }));
        out.push(Request::Now(Box::new(Now::RunPreparationFailed { task })));
        out.push(Request::Decided);
        return Requests::Out(out);
    }
    let Some(grant) = core.accounts.grant(core.settings.account, env.now) else {
        work.push(Event::Tasks(tasks::Event::PreparationFailed { task }));
        out.push(Request::Now(Box::new(Now::RunPreparationFailed { task })));
        out.push(Request::Decided);
        return Requests::Out(out);
    };
    if !core.proofs.contains_key(&task) && core.proofs.len() == core.proofs.capacity() {
        work.push(Event::Tasks(tasks::Event::PreparationFailed { task }));
        out.push(Request::Now(Box::new(Now::RunPreparationFailed { task })));
        out.push(Request::Decided);
        return Requests::Out(out);
    }
    let resume = core.settings.run.resume
        && context.tries.transient == 0
        && transcript.bytes <= u64::from(core.settings.resume_bytes);
    let transcript_from = if resume { transcript.from } else { attempt };
    let offered = if context.last_message == 0 { None } else { Some(context.last_message) };
    assert!(
        core.proofs.insert(task, RunProof::claimed(task, attempt, transcript_from, offered)).is_ok(),
        "claim proof reserved before child mutation"
    );
    let turns = if resume { core.resumed_turns(transcript) } else { Box::new([]) };
    let mut answered = List::with_capacity(env.limits.call_records);
    let mut after = None;
    for _ in 0..core.call_parts.len() {
        let mut next = None;
        for (&key, _) in &core.call_parts {
            if key.task == task
                && key.attempt < attempt
                && match after {
                    Some(previous) => key > previous,
                    None => true,
                }
                && match next {
                    Some(previous) => key < previous,
                    None => true,
                }
            {
                next = Some(key);
            }
        }
        let Some(key) = next else { break };
        answered.push(key).expect("retained call bound");
        after = Some(key);
    }
    let mut policy = core.settings.run.clone();
    policy.resume = resume;
    let run = RunCharter {
        policy,
        contract: context.contract,
        authority: context.authority,
        budget: authority::left(crate::translate::authority_numbers(context.numbers))
            .min(core.authority.rules().maximum_run_spend),
    };
    assert!(core.claiming.insert(task, attempt) == Ok(None), "one pending claim per task");
    out.push(Request::Now(Box::new(Now::RunPrepared {
        task,
        attempt,
        charter: core.settings.charter,
        run: Box::new(run),
        inbox: context.inbox,
        saved: context.saved,
        transcript: turns,
        answered: answered.into_boxed(),
        grant,
    })));
    out.push(Request::Decided);
    Requests::Out(out)
}

pub(super) fn start_brief(core: &mut Core, env: &Env<Limits>, work: &mut Queue<Event>, task: u64) -> Requests {
    let mut out = Queue::with_capacity(1);
    let Some(context) = core.contexts.get(&task) else {
        out.push(Request::Decided);
        return Requests::Out(out);
    };
    let Some(current) = core.tasks.task(task) else {
        out.push(Request::Decided);
        return Requests::Out(out);
    };
    if current.phase != tasks::Phase::Active(tasks::Active::Preparing) || current.last_message != context.last_message {
        out.push(Request::Decided);
        return Requests::Out(out);
    }
    if core.notes.busy() {
        core.pending_note_briefs.insert(task, true).expect("brief count bounds waiting notes");
        out.push(Request::Decided);
        return Requests::Out(out);
    }
    let mut scopes = List::with_capacity(env.limits.notes.scopes);
    scopes.push(notes::Scope::Goal { project: context.project, goal: current.root }).expect("goal scope room");
    scopes.push(notes::Scope::Project { project: context.project }).expect("project scope room");
    scopes.push(notes::Scope::Deployment).expect("deployment scope room");
    for resource in &context.authority.note_resources {
        let last = match &resource.pattern.last {
            tasks::Last::Exact(value) => notes::Last::Exact(value.clone()),
            tasks::Last::Open(value) => notes::Last::Open(value.clone()),
        };
        scopes
            .push(notes::Scope::Resources {
                project: context.project,
                connector: resource.connector,
                pattern: notes::Pattern { segments: resource.pattern.segments.clone(), last },
            })
            .expect("admitted note resource scope room");
    }
    let owner = core.begin_note_route(NoteRoute::Brief { task });
    work.push(Event::Notes(notes::Event::Index { owner, scopes, most: env.limits.notes.lines }));
    out.push(Request::Decided);
    Requests::Out(out)
}

pub(super) fn plan_brief(
    core: &Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    task: u64,
    lines: List<notes::Line>,
    more: u32,
) -> Requests {
    let mut out = Queue::with_capacity(2);
    let Some(context) = core.contexts.get(&task) else {
        out.push(Request::Decided);
        return Requests::Out(out);
    };
    let Some(current) = core.tasks.task(task) else {
        out.push(Request::Decided);
        return Requests::Out(out);
    };
    if current.phase != tasks::Phase::Active(tasks::Active::Preparing) || current.last_message != context.last_message {
        out.push(Request::Decided);
        return Requests::Out(out);
    }
    let text_limits = crate::BriefTextLimits {
        parts: env.limits.brief_parts,
        read_bytes: env.limits.brief.read_bytes,
        brief_bytes: env.limits.brief.brief_bytes,
    };
    let mut wanted = List::with_capacity(env.limits.brief.sections);
    let Some(task_text) = crate::read_brief_part(core, text_limits, task, crate::BriefPart::Spec) else {
        work.push(Event::Tasks(tasks::Event::PreparationFailed { task }));
        out.push(Request::Decided);
        return Requests::Out(out);
    };
    wanted
        .push(brief::Planned::Core {
            kind: brief::Core::Task,
            text: task_text,
            limit: env.limits.brief_core_budgets.task,
            priority: 0,
            required: true,
        })
        .expect("task brief room");
    if !context.dependencies.is_empty() || !context.spec.inputs.is_empty() {
        let Some(text) = crate::read_brief_part(core, text_limits, task, crate::BriefPart::Dependencies) else {
            work.push(Event::Tasks(tasks::Event::PreparationFailed { task }));
            out.push(Request::Decided);
            return Requests::Out(out);
        };
        wanted
            .push(brief::Planned::Core {
                kind: brief::Core::Results,
                text,
                limit: env.limits.brief_core_budgets.dependencies,
                priority: 1,
                required: true,
            })
            .expect("dependency result section room");
    }
    if core.transcript_oversized(task) || context.tries.transient != 0 || !core.settings.run.resume {
        let Some(text) = crate::read_brief_part(core, text_limits, task, crate::BriefPart::TranscriptTail) else {
            work.push(Event::Tasks(tasks::Event::PreparationFailed { task }));
            out.push(Request::Decided);
            return Requests::Out(out);
        };
        wanted
            .push(brief::Planned::Core {
                kind: brief::Core::TranscriptTail,
                text,
                limit: env.limits.brief_core_budgets.task,
                priority: 2,
                required: true,
            })
            .expect("tail brief room");
    }
    plan_note_index(&mut wanted, &env.limits, &lines, more);
    if !context.delegates.is_empty()
        && wanted.room() > 0
        && let Some(text) = crate::read_brief_part(core, text_limits, task, crate::BriefPart::Delegates)
    {
        wanted
            .push(brief::Planned::Core {
                kind: brief::Core::Plan,
                text,
                limit: env.limits.brief_core_budgets.plan,
                priority: 3,
                required: false,
            })
            .expect("delegate section room");
    }
    if context.tries != tasks::Tries::NONE
        && wanted.room() > 0
        && let Some(text) = crate::read_brief_part(core, text_limits, task, crate::BriefPart::Attempts)
    {
        wanted
            .push(brief::Planned::Core {
                kind: brief::Core::Attempts,
                text,
                limit: env.limits.brief_core_budgets.attempts,
                priority: 4,
                required: false,
            })
            .expect("attempt brief room");
    }
    let parent = brief_parent(context.requester);
    out.push(Request::Now(Box::new(Now::BriefCorePlanned { task, parent, sections: wanted })));
    out.push(Request::Decided);
    Requests::Out(out)
}

pub(super) fn brief_parent(requester: tasks::Party) -> Option<u64> {
    match requester {
        tasks::Party::Task(parent) => Some(parent),
        tasks::Party::Person(_) | tasks::Party::Deployment { .. } => None,
    }
}

pub(super) fn plan_note_index(
    wanted: &mut List<brief::Planned>,
    limits: &Limits,
    lines: &List<notes::Line>,
    more: u32,
) {
    if wanted.room() > 0 {
        wanted
            .push(brief::Planned::Core {
                kind: brief::Core::NotesIndex,
                text: crate::note_index_text(lines, more, limits.brief.read_bytes.min(limits.brief_core_budgets.notes)),
                limit: limits.brief_core_budgets.notes,
                priority: 4,
                required: false,
            })
            .expect("note index section room");
    }
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

#[expect(clippy::too_many_lines, reason = "one exhaustive authenticated task proposal dispatcher")]
pub(super) fn named_proposal_accept(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    to: ReplyTo,
    key: CallKey,
    proposal: tasks::Proposal,
) -> Requests {
    let result_followups = core.tasks.result_proposal(proposal.proposer, proposal.number);
    if let Err(why) =
        core.proposal_accept_allowed(&env.limits, proposal.proposer, key.task, proposal.project, &proposal.action)
    {
        return proposal_refused(core, to, key, why);
    }
    match proposal.action {
        tasks::ProposalAction::Effect { connector, .. } => {
            assert!(core.pending_calls.insert(key, true).is_ok(), "effect acceptance call room");
            let owner = to.into_token();
            return crate::effects::start(
                core,
                env,
                owner,
                connector,
                crate::EffectOrigin::Accept {
                    to: ReplyTo::new(owner),
                    key: Some(key),
                    proposal: Box::new(proposal),
                    by: tasks::Party::Task(key.task),
                },
            );
        }
        tasks::ProposalAction::Batch(_)
        | tasks::ProposalAction::Amend { .. }
        | tasks::ProposalAction::Widen { .. }
        | tasks::ProposalAction::Release { .. } => {}
    }
    let Some(message) = crate::fresh(&mut core.counters, crate::Family::Message) else {
        return proposal_refused(core, to, key, tasks::Refusal::Busy);
    };
    let token = to.into_token();
    let event = match proposal.action {
        tasks::ProposalAction::Effect { .. } => unreachable!("effect accepted through its connector"),
        tasks::ProposalAction::Batch(mut batch) => {
            let creator =
                if proposal.as_holder { tasks::Party::Task(key.task) } else { tasks::Party::Task(proposal.proposer) };
            for member in &mut batch {
                member.funder = tasks::Funder::Task(key.task);
            }
            if result_followups {
                tasks::Event::MakeResultFollowups {
                    reply_to: ReplyTo::new(token),
                    proposer: proposal.proposer,
                    proposal: proposal.number,
                    batch,
                }
            } else {
                tasks::Event::Make { reply_to: ReplyTo::new(token), creator, batch }
            }
        }
        tasks::ProposalAction::Amend { task, amendment } => {
            let Some(message) = crate::fresh(&mut core.counters, crate::Family::Message) else {
                return proposal_refused(core, ReplyTo::new(token), key, tasks::Refusal::Busy);
            };
            let Some(current) = core.tasks.delegation(task) else {
                return proposal_refused(core, ReplyTo::new(token), key, tasks::Refusal::Unknown);
            };
            let after =
                crate::translate::authority_value(amendment.authority.as_ref().expect("proposal amendment authority"));
            let before = crate::translate::authority_value(&current.authority);
            let stop_run = !authority::at_most(&before, &after, &core.authority.rules().implies);
            tasks::Event::Amend {
                reply_to: ReplyTo::new(token),
                by: tasks::Party::Task(key.task),
                task,
                message,
                stop_run,
                amendment,
            }
        }
        tasks::ProposalAction::Widen { task, authority } => {
            let Some(message) = crate::fresh(&mut core.counters, crate::Family::Message) else {
                return proposal_refused(core, ReplyTo::new(token), key, tasks::Refusal::Busy);
            };
            tasks::Event::Amend {
                reply_to: ReplyTo::new(token),
                by: tasks::Party::Task(key.task),
                task,
                message,
                stop_run: false,
                amendment: tasks::Amendment {
                    spec: None,
                    wake: None,
                    dependencies: None,
                    authority: Some(authority),
                    reason: proposal.reason,
                },
            }
        }
        tasks::ProposalAction::Release { task } => tasks::Event::Control {
            reply_to: ReplyTo::new(token),
            by: tasks::Party::Task(key.task),
            task,
            control: tasks::Control::Release,
        },
    };
    assert!(core.pending_calls.insert(key, true).is_ok(), "proposal acceptance room reserved");
    assert!(
        core.routing_calls.insert(
            token,
            RoutedCall::Accepting { key, proposer: proposal.proposer, proposal: proposal.number, message }
        ) == Ok(None),
        "one acceptance route"
    );
    work.push(Event::Tasks(event));
    let mut out = Queue::with_capacity(1);
    out.push(Request::Decided);
    Requests::Out(out)
}

#[expect(clippy::too_many_arguments, reason = "one named proposal decision carries its call and proposal identity")]
pub(super) fn named_proposal_decide(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    to: ReplyTo,
    key: CallKey,
    proposer: u64,
    proposal: u64,
    choice: ProposalChoice,
) -> Requests {
    let Some(pending) = core.tasks.proposal(proposer, proposal) else {
        return proposal_refused(core, to, key, tasks::Refusal::Unknown);
    };
    let current = match pending.state {
        tasks::ProposalState::Pending { holder, .. } => holder,
        tasks::ProposalState::Accepted { .. }
        | tasks::ProposalState::Rejected { .. }
        | tasks::ProposalState::Withdrawn => {
            return proposal_refused(core, to, key, tasks::Refusal::State);
        }
    };
    if current != tasks::ProposalHolder::Task(key.task) {
        return proposal_refused(core, to, key, tasks::Refusal::Reference);
    }
    let next = match choice {
        ProposalChoice::Accept => return named_proposal_accept(core, env, work, to, key, pending),
        ProposalChoice::Reject { reason } => tasks::ProposalDecision::Reject { reason },
        ProposalChoice::Pass => {
            let Some(holder) = core.proposal_holder(&env.limits, proposer, &pending.action, Some(current)) else {
                return proposal_refused(core, to, key, tasks::Refusal::Reference);
            };
            tasks::ProposalDecision::Pass { holder }
        }
    };
    let message = match next {
        tasks::ProposalDecision::Reject { .. } => match crate::fresh(&mut core.counters, crate::Family::Message) {
            Some(message) => Some(message),
            None => return proposal_refused(core, to, key, tasks::Refusal::Busy),
        },
        tasks::ProposalDecision::Pass { .. } => None,
        tasks::ProposalDecision::Accept => unreachable!("accept routed through action"),
    };
    let token = to.into_token();
    assert!(core.pending_calls.insert(key, true).is_ok(), "proposal decision room reserved");
    assert!(core.routing_calls.insert(token, RoutedCall::Decide { key, proposal }) == Ok(None), "one route");
    work.push(Event::Tasks(tasks::Event::DecideProposal {
        reply_to: ReplyTo::new(token),
        proposer,
        proposal,
        message,
        by: tasks::Party::Task(key.task),
        decision: next,
    }));
    let mut out = Queue::with_capacity(1);
    out.push(Request::Decided);
    Requests::Out(out)
}

#[expect(clippy::too_many_arguments, reason = "one named proposal retains its action and call correlation")]
pub(crate) fn named_propose(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    to: ReplyTo,
    key: CallKey,
    action: tasks::ProposalAction,
    reason: Box<[u8]>,
    as_holder: bool,
) -> Requests {
    if reason.len() > usize::try_from(env.limits.tasks.message_bytes).expect("u32 fits usize") {
        return proposal_refused(core, to, key, tasks::Refusal::Read);
    }
    let Some(context) = core.tasks.delegation(key.task) else {
        return proposal_refused(core, to, key, tasks::Refusal::Unknown);
    };
    if let tasks::ProposalAction::Amend { amendment, .. } = &action
        && amendment.authority.is_none()
    {
        return proposal_refused(core, to, key, tasks::Refusal::AuthorityShape);
    }
    if let Err(why) = core.proposal_admit_action(context.project, &action) {
        return proposal_refused(core, to, key, why);
    }
    let Some(holder) = core.proposal_holder(&env.limits, key.task, &action, None) else {
        return proposal_refused(core, to, key, tasks::Refusal::Reference);
    };
    let Some(number) = crate::fresh(&mut core.counters, crate::Family::Message) else {
        return proposal_refused(core, to, key, tasks::Refusal::Busy);
    };
    let token = to.into_token();
    assert!(core.pending_calls.insert(key, true).is_ok(), "proposal call room reserved");
    assert!(core.routing_calls.insert(token, RoutedCall::Propose { key, proposal: number }) == Ok(None), "one route");
    work.push(Event::Tasks(tasks::Event::Propose {
        reply_to: ReplyTo::new(token),
        proposal: tasks::Proposal {
            number,
            proposer: key.task,
            project: context.project,
            action,
            reason,
            as_holder,
            state: tasks::ProposalState::Pending { holder, since: env.wall },
        },
    }));
    let mut out = Queue::with_capacity(1);
    out.push(Request::Decided);
    Requests::Out(out)
}

#[expect(clippy::too_many_arguments, reason = "one named holder choice carries its call, task and revision")]
pub(super) fn named_escalation(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    to: ReplyTo,
    key: CallKey,
    task: u64,
    revision: u64,
    choice: EscalationChoice,
) -> Requests {
    let Some(context) = core.tasks.escalation(task) else {
        return escalation_refused(core, to, key, task, tasks::Refusal::Unknown);
    };
    let (current, holder) = match context.escalation {
        tasks::Escalation::Waiting { revision, holder, .. } => (revision, holder),
        tasks::Escalation::Unheld { .. } | tasks::Escalation::Routing { .. } | tasks::Escalation::Rejected { .. } => {
            return escalation_refused(core, to, key, task, tasks::Refusal::State);
        }
    };
    if current != revision || holder != tasks::EscalationHolder::Task(key.task) {
        return escalation_refused(core, to, key, task, tasks::Refusal::Reference);
    }
    let role = core.people.role(context.requester, context.project);
    let semantic = match choice {
        EscalationChoice::Release => {
            if core.escalation_recipient(&env.limits, &context, role) != Some(holder) {
                return escalation_refused(core, to, key, task, tasks::Refusal::Funding);
            }
            tasks::EscalationDecision::Release
        }
        EscalationChoice::Reject { reason } => tasks::EscalationDecision::Reject { reason },
        EscalationChoice::Pass => {
            let Some(next) = core.escalation_recipient_after(&env.limits, &context, role, Some(holder)) else {
                return escalation_refused(core, to, key, task, tasks::Refusal::Reference);
            };
            tasks::EscalationDecision::Pass { holder: next }
        }
    };
    let entry = match semantic {
        tasks::EscalationDecision::Pass { .. } => match crate::fresh(&mut core.counters, crate::Family::Message) {
            Some(entry) => Some(entry),
            None => return escalation_refused(core, to, key, task, tasks::Refusal::Busy),
        },
        tasks::EscalationDecision::Release | tasks::EscalationDecision::Reject { .. } => None,
    };
    let token = to.into_token();
    assert!(core.pending_calls.insert(key, true).is_ok(), "task escalation call room reserved");
    assert!(core.routing_calls.insert(token, RoutedCall::Escalation { key, task, revision }) == Ok(None), "one route");
    work.push(Event::TaskEscalation(tasks::Event::DecideEscalation {
        reply_to: ReplyTo::new(token),
        task,
        revision,
        by: key.task,
        entry,
        decision: semantic,
    }));
    let mut out = Queue::with_capacity(1);
    out.push(Request::Decided);
    Requests::Out(out)
}

#[expect(clippy::too_many_lines, reason = "one generic named-call vocabulary routes its finite task continuations")]
pub(super) fn named_action(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    to: ReplyTo,
    key: CallKey,
    action: NamedAction,
) -> Requests {
    assert!(core.current_proof(key.task, key.attempt), "named action belongs to current claim");
    if let Some(part) = core.authorize_tool(key, named_tool_kind(&action)) {
        let mut out = Queue::with_capacity(3);
        call_decision(core, &mut out, to, key, part);
        out.push(Request::Decided);
        return Requests::Out(out);
    }
    let token = to.into_token();
    let mut out = Queue::with_capacity(3);
    match action {
        NamedAction::Message { target, kind, words } => {
            let Some(context) = core.tasks.delegation(key.task) else {
                call_decision(
                    core,
                    &mut out,
                    ReplyTo::new(token),
                    key,
                    CallPart::MessageRefused(tasks::Problem {
                        task: Some(key.task),
                        why: tasks::Refusal::Unknown,
                        blocked_by: None,
                    }),
                );
                out.push(Request::Decided);
                return Requests::Out(out);
            };
            let project = context.project;
            let Some(number) = crate::fresh(&mut core.counters, crate::Family::Message) else {
                call_decision(
                    core,
                    &mut out,
                    ReplyTo::new(token),
                    key,
                    CallPart::MessageRefused(tasks::Problem {
                        task: Some(target),
                        why: tasks::Refusal::Busy,
                        blocked_by: None,
                    }),
                );
                out.push(Request::Decided);
                return Requests::Out(out);
            };
            assert!(core.pending_calls.insert(key, true).is_ok(), "call record room reserved");
            assert!(core.routing_calls.insert(token, RoutedCall::Message(key)) == Ok(None), "one live routed call");
            work.push(Event::Tasks(tasks::Event::Message {
                reply_to: ReplyTo::new(token),
                project,
                task: target,
                word: tasks::Word {
                    number,
                    from: tasks::Party::Task(key.task),
                    kind,
                    words,
                    at: env.wall,
                    hits: 1,
                    eligible: false,
                },
            }));
        }
        NamedAction::Introduce { left, right } => {
            assert!(core.pending_calls.insert(key, true).is_ok(), "call record room reserved");
            assert!(core.routing_calls.insert(token, RoutedCall::Introduce(key)) == Ok(None), "one live routed call");
            work.push(Event::Tasks(tasks::Event::Introduce {
                reply_to: ReplyTo::new(token),
                by: key.task,
                left,
                right,
            }));
        }
        NamedAction::Subscribe { kind } => {
            let Some(subscription) = crate::fresh(&mut core.counters, crate::Family::Message) else {
                call_decision(
                    core,
                    &mut out,
                    ReplyTo::new(token),
                    key,
                    CallPart::SubscriptionRefused(tasks::Problem {
                        task: Some(key.task),
                        why: tasks::Refusal::Busy,
                        blocked_by: None,
                    }),
                );
                out.push(Request::Decided);
                return Requests::Out(out);
            };
            assert!(core.pending_calls.insert(key, true).is_ok(), "call record room reserved");
            assert!(
                core.routing_calls.insert(token, RoutedCall::Subscribe { key, subscription }) == Ok(None),
                "one live routed call"
            );
            work.push(Event::Tasks(tasks::Event::Subscribe {
                reply_to: ReplyTo::new(token),
                task: key.task,
                subscription: tasks::Subscription { number: subscription, kind },
            }));
        }
        NamedAction::Unsubscribe { subscription } => {
            assert!(core.pending_calls.insert(key, true).is_ok(), "call record room reserved");
            assert!(core.routing_calls.insert(token, RoutedCall::Unsubscribe(key)) == Ok(None), "one live routed call");
            work.push(Event::Tasks(tasks::Event::Unsubscribe {
                reply_to: ReplyTo::new(token),
                task: key.task,
                subscription,
            }));
        }
        NamedAction::Control { target, control } => {
            assert!(core.pending_calls.insert(key, true).is_ok(), "call record room reserved");
            assert!(core.routing_calls.insert(token, RoutedCall::Control(key)) == Ok(None), "one live routed call");
            work.push(Event::Tasks(tasks::Event::Control {
                reply_to: ReplyTo::new(token),
                by: tasks::Party::Task(key.task),
                task: target,
                control,
            }));
        }
        NamedAction::Amend { target, amendment } => {
            let stop_run = match core.task_amend_admit(key.task, target, &amendment, env.limits.tasks.depth) {
                Ok(stop_run) => stop_run,
                Err(crate::TaskAmendDenied::Refused { task, why }) => {
                    call_decision(
                        core,
                        &mut out,
                        ReplyTo::new(token),
                        key,
                        CallPart::ControlRefused(tasks::Problem { task: Some(task), why, blocked_by: None }),
                    );
                    out.push(Request::Decided);
                    return Requests::Out(out);
                }
                Err(crate::TaskAmendDenied::Denied(answer)) => {
                    call_decision(core, &mut out, ReplyTo::new(token), key, CallPart::ControlDenied { answer });
                    out.push(Request::Decided);
                    return Requests::Out(out);
                }
            };
            let Some(message) = crate::fresh(&mut core.counters, crate::Family::Message) else {
                call_decision(
                    core,
                    &mut out,
                    ReplyTo::new(token),
                    key,
                    CallPart::ControlRefused(tasks::Problem {
                        task: Some(target),
                        why: tasks::Refusal::Busy,
                        blocked_by: None,
                    }),
                );
                out.push(Request::Decided);
                return Requests::Out(out);
            };
            assert!(core.pending_calls.insert(key, true).is_ok(), "call record room reserved");
            assert!(core.routing_calls.insert(token, RoutedCall::Control(key)) == Ok(None), "one amendment route");
            work.push(Event::Tasks(tasks::Event::Amend {
                reply_to: ReplyTo::new(token),
                by: tasks::Party::Task(key.task),
                task: target,
                message,
                stop_run,
                amendment,
            }));
        }
        NamedAction::DecideEscalation { task, revision, choice } => {
            return named_escalation(core, env, work, ReplyTo::new(token), key, task, revision, choice);
        }
        NamedAction::WithdrawProposal { proposal } => {
            if core.tasks.proposal(key.task, proposal).is_none() {
                return proposal_refused(core, ReplyTo::new(token), key, tasks::Refusal::Unknown);
            }
            assert!(core.pending_calls.insert(key, true).is_ok(), "proposal call room reserved");
            assert!(core.routing_calls.insert(token, RoutedCall::Withdraw { key, proposal }) == Ok(None), "one route");
            work.push(Event::Tasks(tasks::Event::WithdrawProposal {
                reply_to: ReplyTo::new(token),
                proposer: key.task,
                proposal,
            }));
        }
        NamedAction::DecideProposal { proposer, proposal, choice } => {
            return named_proposal_decide(core, env, work, ReplyTo::new(token), key, proposer, proposal, choice);
        }
        NamedAction::Propose { action, reason, as_holder } => {
            return named_propose(core, env, work, ReplyTo::new(token), key, action, reason, as_holder);
        }
        NamedAction::ProposeBatch { batch, reason, as_holder } => {
            return propose_batch(core, env, ReplyTo::new(token), key, batch, reason, as_holder, None);
        }
        NamedAction::ProposeGoal { goal, priority, reason } => {
            return propose_batch(core, env, ReplyTo::new(token), key, Box::new([*goal]), reason, true, Some(priority));
        }
        NamedAction::Note { mut entry, recalled } => {
            if recalled.is_none() && entry.name != 0 {
                call_decision(core, &mut out, ReplyTo::new(token), key, CallPart::NoteRefused(notes::Refusal::Exists));
                out.push(Request::Decided);
                return Requests::Out(out);
            }
            if recalled.is_some() && entry.name == 0 {
                call_decision(core, &mut out, ReplyTo::new(token), key, CallPart::NoteRefused(notes::Refusal::Missing));
                out.push(Request::Decided);
                return Requests::Out(out);
            }
            if entry.name == 0 {
                let Some(name) = crate::fresh(&mut core.counters, crate::Family::Message) else {
                    call_decision(
                        core,
                        &mut out,
                        ReplyTo::new(token),
                        key,
                        CallPart::NoteRefused(notes::Refusal::Full),
                    );
                    out.push(Request::Decided);
                    return Requests::Out(out);
                };
                entry.name = name;
            }
            entry.author = notes::Author::Task { task: key.task, attempt: key.attempt };
            assert!(core.pending_calls.insert(key, true) == Ok(None), "note call record room reserved");
            let owner = core.begin_note_route(NoteRoute::Call { to: ReplyTo::new(token), key });
            work.push(Event::Notes(notes::Event::Write { owner, entry, recalled }));
        }
        NamedAction::Recall { by, page } => {
            assert!(core.pending_calls.insert(key, true) == Ok(None), "recall call record room reserved");
            let owner = core.begin_note_route(NoteRoute::Call { to: ReplyTo::new(token), key });
            work.push(Event::Notes(notes::Event::Recall { owner, by, page }));
        }
    }
    out.push(Request::Decided);
    Requests::Out(out)
}

pub(super) fn named_tool_kind(action: &NamedAction) -> ToolKind {
    match action {
        NamedAction::Message { target, .. } => ToolKind::Message { target: *target },
        NamedAction::Introduce { left, .. } => ToolKind::Message { target: *left },
        NamedAction::Subscribe { .. } | NamedAction::Unsubscribe { .. } => ToolKind::Subscribe,
        NamedAction::Control { .. } | NamedAction::Amend { .. } => ToolKind::Control,
        NamedAction::DecideEscalation { .. }
        | NamedAction::WithdrawProposal { .. }
        | NamedAction::DecideProposal { .. } => ToolKind::Decide,
        NamedAction::Propose { .. } | NamedAction::ProposeBatch { .. } | NamedAction::ProposeGoal { .. } => {
            ToolKind::Propose
        }
        NamedAction::Note { entry, .. } => ToolKind::Note { scope: entry.scope.clone() },
        NamedAction::Recall { by, .. } => ToolKind::Recall { by: by.clone() },
    }
}

pub(super) fn route_named_answer(core: &mut Core, to: ReplyTo, key: CallKey, part: CallPart) -> Requests {
    assert!(core.current_proof(key.task, key.attempt), "named call belongs to current claim");
    let mut out = Queue::with_capacity(3);
    call_decision(core, &mut out, to, key, part);
    out.push(Request::Decided);
    Requests::Out(out)
}

pub(super) fn route_prepared_agent(
    core: &mut Core,
    context: Box<tasks::RunContext>,
    transcript_waiter: Option<Token>,
    busy: bool,
) -> Requests {
    let mut out = Queue::with_capacity(2);
    if busy {
        remember_due(core, context);
    } else {
        let number = context.task;
        let previous_attempt = context.previous_attempt;
        assert!(core.contexts.insert(number, context).is_ok(), "bounded activation context");
        core.begin_transcript(number, previous_attempt);
        out.push(Request::Now(Box::new(Now::StartPreparation { task: number, transcript_waiter })));
    }
    out.push(Request::Decided);
    Requests::Out(out)
}

pub(super) fn route_activate(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    context: Box<tasks::RunContext>,
    ready: bool,
) -> Requests {
    let number = context.task;
    let mut out = Queue::with_capacity(2);
    if !ready || core.restart_step().is_some() || core.restart_failure().is_some() {
        remember_due(core, context);
    } else if core.tasks.recurring_template(number).is_none() {
        match context.executor {
            tasks::Executor::Person(_) => {}
            tasks::Executor::Procedure { connector, .. } => {
                if let Some(step) = context.previous_attempt.checked_add(1) {
                    out.push(Request::Ask { connector, ask: Ask::StartProcedure { context, step } });
                } else {
                    work.push(Event::Tasks(tasks::Event::Hold { task: number, why: tasks::Hold::Effects }));
                }
            }
            tasks::Executor::Agent { .. } => match core.run_admission(&context, env.wall, Box::new([])) {
                crate::RunAdmission::Allow => out.push(Request::Now(Box::new(Now::PrepareAgent { context }))),
                crate::RunAdmission::Account => remember_due(core, context),
                crate::RunAdmission::Hold(why) => {
                    work.push(Event::Tasks(tasks::Event::Hold { task: number, why }));
                }
            },
        }
    }
    out.push(Request::Decided);
    Requests::Out(out)
}

pub(super) fn route_refused_payload(
    core: &mut Core,
    work: &mut Queue<Event>,
    request: Token,
    problem: tasks::Problem,
    payload: Option<PayloadRefusal>,
) -> Requests {
    match payload {
        Some(PayloadRefusal::Turn { task, attempt, turn }) if problem.task == Some(task) => {
            work.push(Event::Fleet(fleet::Event::TurnBusy {
                run: Token::new(task),
                attempt: Token::new(attempt),
                turn,
            }));
        }
        Some(PayloadRefusal::Answer { task, attempt })
            if problem.task == Some(task) && core.current_proof(task, attempt) =>
        {
            if core.committed_terminal(task, attempt).is_some() {
                let mut out = Queue::with_capacity(1);
                out.push(Request::Decided);
                return Requests::Out(out);
            }
            let proof = core.proofs.get_mut(&task).expect("answer proof reserved");
            proof.terminal = Some(crate::TerminalRecord {
                task,
                attempt,
                cumulative: match proof.turn {
                    Some(turn) => turn.cumulative,
                    None => 0,
                },
                end: tasks::End::Failed(tasks::Class::Invalid),
            });
            work.push(Event::Tasks(tasks::Event::Activation {
                reply_to: ReplyTo::new(request),
                task,
                attempt,
                end: tasks::End::Failed(tasks::Class::Invalid),
                saved: None,
                cause: tasks::Cause::Unpriced,
            }));
        }
        Some(PayloadRefusal::Turn { .. } | PayloadRefusal::Answer { .. }) | None => {}
    }
    let mut out = Queue::with_capacity(1);
    out.push(Request::Decided);
    Requests::Out(out)
}

#[expect(clippy::too_many_arguments, reason = "the route retains its event fields")]
pub(super) fn route_accepted_turn(
    core: &mut Core,
    env: &Env<Limits>,
    task: u64,
    attempt: u64,
    turn: u32,
    accepted: tasks::Accepted,
    cumulative: u64,
    read: Option<u64>,
    transcript: Box<[u8]>,
) -> Requests {
    let proof = core.proofs.get_mut(&task).expect("turn proof reserved before child mutation");
    assert!(proof.attempt == attempt, "turn callback retains its claim");
    let mut out = Queue::with_capacity(env.limits.call_records.checked_add(6).expect("turn route room"));
    match accepted {
        tasks::Accepted::New => {
            proof.turn = Some(crate::TurnProof { turn, cumulative, read });
            out.push(Request::Write(Write::Save(Record::Core(CoreRecord::RunProof(proof.clone())))));
            out.push(Request::Write(Write::Save(Record::Core(CoreRecord::Turn(crate::TurnRecord {
                task,
                attempt,
                turn,
                spent: cumulative,
                read,
                at: env.wall,
                transcript,
            })))));
            for key in core.retire_calls(task, attempt, turn, env.limits.call_records) {
                out.push(Request::Write(Write::Erase(Key::Core(CoreKey::Call(key)))));
            }
            out.push(Request::Held(Box::new(Held::ViewTurn { task, attempt, turn })));
        }
        tasks::Accepted::Already => {}
    }
    out.push(Request::Held(Box::new(Held::TaskTurnKept { task, attempt, turn })));
    out.push(Request::Decided);
    Requests::Out(out)
}

#[expect(clippy::too_many_arguments, reason = "the route retains its event fields")]
pub(super) fn route_answer_payload(
    core: &mut Core,
    work: &mut Queue<Event>,
    run: Token,
    attempt: Token,
    payload: Token,
    cumulative: u64,
    end: tasks::End,
    saved: Option<Box<[tasks::SavedResource]>>,
    invalid_saved: bool,
) -> Requests {
    assert!(core.current_proof(run.raw(), attempt.raw()), "current answer has reserved proof");
    let (cumulative, end, saved) = match core.committed_terminal(run.raw(), attempt.raw()) {
        Some(terminal) => (terminal.cumulative, terminal.end, None),
        None => {
            if invalid_saved {
                (cumulative, tasks::End::Failed(tasks::Class::Invalid), None)
            } else {
                (cumulative, end, saved)
            }
        }
    };
    let proof = core.proofs.get_mut(&run.raw()).expect("terminal proof pre-reserved");
    proof.terminal =
        Some(crate::TerminalRecord { task: run.raw(), attempt: attempt.raw(), cumulative, end: end.clone() });
    work.push(Event::Tasks(tasks::Event::Activation {
        reply_to: ReplyTo::new(payload),
        task: run.raw(),
        attempt: attempt.raw(),
        cause: tasks::Cause::Priced { cumulative },
        end,
        saved,
    }));
    let mut out = Queue::with_capacity(1);
    out.push(Request::Decided);
    Requests::Out(out)
}

#[expect(clippy::too_many_arguments, reason = "the route retains its event fields")]
pub(super) fn route_turn_payload(
    core: &mut Core,
    work: &mut Queue<Event>,
    run: Token,
    attempt: Token,
    turn: u32,
    body: Token,
    read: Option<u64>,
    cumulative: u64,
) -> Requests {
    assert!(core.current_proof(run.raw(), attempt.raw()), "current turn has reserved proof");
    let offered = core.proofs.get(&run.raw()).expect("current proof").offered;
    work.push(Event::Tasks(tasks::Event::Turn {
        reply_to: ReplyTo::new(body),
        task: run.raw(),
        attempt: attempt.raw(),
        turn,
        read,
        offered,
        cumulative,
    }));
    let mut out = Queue::with_capacity(1);
    out.push(Request::Decided);
    Requests::Out(out)
}

pub(super) fn route_preparation_failed(core: &mut Core, work: &mut Queue<Event>, task: u64) -> Requests {
    drop(core.contexts.remove(&task));
    drop(core.dependency_results.remove(&task));
    drop(core.transcripts.remove(&task));
    work.push(Event::Tasks(tasks::Event::PreparationFailed { task }));
    let mut out = Queue::with_capacity(1);
    out.push(Request::Decided);
    Requests::Out(out)
}

pub(super) fn route_period(
    core: &mut Core,
    work: &mut Queue<Event>,
    project: u32,
    period: u64,
    budget: u64,
) -> Requests {
    let allowed = match core.authority.policy(project) {
        Some(policy) => budget <= policy.period_spend,
        None => false,
    };
    if period > 0 && period >= core.settings.period && allowed {
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

pub(super) fn route_claim_refused(core: &mut Core, work: &mut Queue<Event>, task: u64) -> Requests {
    let mut out = Queue::with_capacity(2);
    if core.claiming.remove(&task).is_some() {
        drop(core.proofs.remove(&task));
        out.push(Request::Now(Box::new(Now::DropAssignment { task })));
        work.push(Event::Tasks(tasks::Event::PreparationFailed { task }));
    }
    out.push(Request::Decided);
    Requests::Out(out)
}

pub(super) fn route_claim_prepared(
    core: &mut Core,
    work: &mut Queue<Event>,
    task: u64,
    attempt: u64,
    budget: u64,
    writes: Box<[tasks::Name]>,
) -> Requests {
    if core.claiming.get(&task) == Some(&attempt) {
        work.push(Event::Tasks(tasks::Event::Claim {
            reply_to: ReplyTo::new(Token::new(task)),
            task,
            attempt,
            budget,
            writes,
        }));
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
        }),
    });
    let mut out = Queue::with_capacity(1);
    out.push(Request::Decided);
    Requests::Out(out)
}
