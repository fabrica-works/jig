//! Party pools, task creation, adoption, service, policy and role requests
//! routed through authority, people, tasks and connectors
//! (`domain/people.md`, section 5.1). Controls and decisions are separate
//! families so each route file stays under 1,500 lines.

use super::refusals::pool_refused;
use super::{Creation, CreationKind};
use crate::{Ask, Core, Event, GoalRoute, Limits, PersonTaskRoute, Record, Request, Requests, Write};
use alloc::boxed::Box;
use jig_core_people as people;
use jig_core_tasks as tasks;
use skein_lib::{Env, List, Queue, ReplyTo, Token};

pub(super) fn route_holdings(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    request: Token,
    connector: u16,
    holdings: Option<Box<[tasks::Holding]>>,
) -> Requests {
    created_holdings(core, env, work, request, connector, holdings);
    let mut out = Queue::with_capacity(1);
    out.push(Request::Decided);
    Requests::Out(out)
}
#[expect(clippy::too_many_arguments, reason = "a keyed pool edit carries its owner, beneficiary and budget")]
pub(super) fn pool_route(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    request: Token,
    owner: u64,
    project: u32,
    person: u64,
    budget: u64,
) {
    if let Err(why) = core.role_allowed(owner, project, env.limits.tasks.tree_tasks) {
        pool_refused(work, request, why);
        return;
    }
    if core.person_tasks.len() >= env.limits.tasks.tasks || work.room() < 2 {
        super::refusals::pool_again(work, request, people::Again::Busy);
        return;
    }
    let Some(role) = core.people.role(person, project) else {
        pool_refused(work, request, people::Refusal::Unknown);
        return;
    };
    let Some(allowed) = core.authority.role(project, role.number()) else {
        pool_refused(work, request, people::Refusal::Unknown);
        return;
    };
    if budget > allowed.period_spend {
        pool_refused(
            work,
            request,
            people::Refusal::Authority {
                proposable: None,
                lacking: people::Lack::Budget { needs: budget, left: allowed.period_spend },
            },
        );
        return;
    }
    let Some(policy) = core.authority.policy(project) else {
        pool_refused(work, request, people::Refusal::Unknown);
        return;
    };
    if core.settings.period_budget > policy.period_spend {
        pool_refused(
            work,
            request,
            people::Refusal::Authority {
                proposable: None,
                lacking: people::Lack::Budget { needs: core.settings.period_budget, left: policy.period_spend },
            },
        );
        return;
    }
    let period = tasks::Funder::Period { project, period: core.settings.period };
    let funder = tasks::Funder::Pool { project, person, period: core.settings.period };
    let needed =
        u32::from(core.tasks.funding(period).is_none()).saturating_add(u32::from(core.tasks.funding(funder).is_none()));
    if core.tasks.funding_room() < needed {
        super::refusals::pool_again(work, request, people::Again::Busy);
        return;
    }
    if core.tasks.funding(period).is_none() && budget > core.settings.period_budget {
        pool_refused(
            work,
            request,
            people::Refusal::Authority {
                proposable: None,
                lacking: people::Lack::Budget { needs: budget, left: core.settings.period_budget },
            },
        );
        return;
    }
    if core.tasks.funding(period).is_none() {
        work.push(Event::Tasks(tasks::Event::OpenPeriod {
            reply_to: ReplyTo::new(Token::new(0)),
            project,
            period: core.settings.period,
            budget: core.settings.period_budget,
        }));
    }
    assert!(
        core.person_tasks.insert(request, PersonTaskRoute::PoolSet { project, person }) == Ok(None),
        "one admitted pool route"
    );
    if core.tasks.funding(funder).is_some() {
        work.push(Event::Tasks(tasks::Event::ResizePool {
            reply_to: ReplyTo::new(request),
            project,
            person,
            period: core.settings.period,
            budget,
        }));
    } else {
        work.push(Event::Tasks(tasks::Event::CarvePool {
            reply_to: ReplyTo::new(request),
            project,
            person,
            period: core.settings.period,
            budget,
        }));
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "one authenticated goal carries its task specification and funding request"
)]
pub(super) fn begin_goal_creation(
    core: &mut Core,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
    work: &mut Queue<Event>,
    request: Token,
    person: u64,
    project: u32,
    role: people::Role,
    spec: Box<[u8]>,
    charter: u32,
    budget: u64,
    priority: u32,
    deadline: Option<skein_lib::Wall>,
) {
    if core.projections.len() >= env.limits.tasks.tasks {
        pool_refused(work, request, people::Refusal::Limit { which: people::LimitKind::LiveTasks });
        return;
    }
    let admitted = match core.goal_start(project, person, role, charter, budget, deadline, env.limits.tasks.tree_tasks)
    {
        Ok(admitted) => admitted,
        Err(why) => {
            pool_refused(work, request, why);
            return;
        }
    };
    let Some(number) = crate::fresh(&mut core.counters, crate::Family::Task) else {
        pool_refused(work, request, people::Refusal::Limit { which: people::LimitKind::Arithmetic });
        return;
    };
    let new = tasks::New {
        number,
        project,
        executor: tasks::Executor::Agent { charter },
        spec: tasks::Spec { words: spec, parameters: Box::new([]), inputs: Box::new([]) },
        contract: tasks::Contract::Report { words: env.limits.tasks.result_bytes },
        authority: crate::translate::task_authority(&admitted.authority),
        numbers: tasks::Numbers { budget, spent: 0, spent_below: 0, reserved: 0 },
        funder: admitted.source,
        dependencies: Box::new([]),
        holdings: Box::new([]),
        wake: tasks::WakePolicy::DEFAULT,
        recurring: None,
        tracked: Some(priority),
    };
    if !tasks::valid_spec(&env.limits.tasks, &new.spec) || !tasks::valid_authority(&env.limits.tasks, &new.authority) {
        pool_refused(work, request, people::Refusal::Limit { which: people::LimitKind::Words });
        return;
    }
    begin_creation(core, env, out, work, request, CreationKind::Goal { person, direct: admitted.direct }, new);
}

#[expect(clippy::too_many_arguments, reason = "one authenticated chat carries its membership and opening words")]
pub(super) fn begin_chat_creation(
    core: &mut Core,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
    work: &mut Queue<Event>,
    request: Token,
    person: u64,
    project: u32,
    role: people::Role,
    words: Box<[u8]>,
) {
    if let Err(why) = core.chat_admit(project, person, role, env.limits.tasks.tree_tasks) {
        pool_refused(work, request, why);
        return;
    }
    let (period, pool) = core.goal_open_pool(project, person);
    if let Some(event) = period {
        work.push(Event::Tasks(event));
    }
    if let Some(event) = pool {
        work.push(Event::Tasks(event));
    }
    let Some(number) = crate::fresh(&mut core.counters, crate::Family::Task) else {
        pool_refused(work, request, people::Refusal::Limit { which: people::LimitKind::Arithmetic });
        return;
    };
    let new = tasks::New {
        number,
        project,
        executor: tasks::Executor::Agent { charter: core.settings.charter },
        spec: tasks::Spec { words, parameters: Box::new([]), inputs: Box::new([]) },
        contract: tasks::Contract::Report { words: env.limits.tasks.result_bytes },
        authority: crate::translate::task_authority(&core.settings.chat_authority),
        numbers: tasks::Numbers {
            budget: core.settings.chat_authority.budget.spend,
            spent: 0,
            spent_below: 0,
            reserved: 0,
        },
        funder: tasks::Funder::Pool { project, person, period: core.settings.period },
        dependencies: Box::new([]),
        holdings: Box::new([]),
        wake: tasks::WakePolicy::DEFAULT,
        recurring: None,
        tracked: None,
    };
    begin_creation(core, env, out, work, request, CreationKind::Chat { person }, new);
}

pub(super) fn begin_creation(
    core: &mut Core,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
    work: &mut Queue<Event>,
    request: Token,
    kind: CreationKind,
    new: tasks::New,
) {
    if core.creating.len() == core.creating.capacity() {
        super::refusals::pool_again(work, request, people::Again::Busy);
        return;
    }
    let remaining = u32::try_from(core.connectors.len()).expect("configured connector count fits u32");
    assert!(remaining > 0, "task creation has a numbered connector");
    for &connector in core.connectors.as_ref() {
        out.push(Request::Ask {
            connector,
            ask: Ask::TaskHoldings {
                request,
                project: new.project,
                root: new.number,
                number: new.number,
                executor: new.executor,
                spec: new.spec.clone(),
            },
        });
    }
    let inserted = core.creating.insert(
        request,
        Creation {
            kind,
            new,
            remaining,
            holdings: List::with_capacity(env.limits.tasks.holdings),
            answered: List::with_capacity(remaining),
            failed: false,
        },
    );
    match inserted {
        Ok(None) => {}
        Ok(Some(_)) | Err(_) => unreachable!("one keyed task creation"),
    }
}

pub(super) fn created_holdings(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    request: Token,
    connector: u16,
    holdings: Option<Box<[tasks::Holding]>>,
) {
    let Some(creating) = core.creating.get_mut(&request) else { return };
    assert!(creating.remaining > 0, "one holdings answer per connector");
    assert!(
        core.connectors.contains(&connector) && !creating.answered.as_slice().contains(&connector),
        "one answer from each numbered connector"
    );
    creating.answered.push(connector).expect("one answer per configured connector");
    creating.remaining = creating.remaining.checked_sub(1).expect("one holdings answer per connector");
    match holdings {
        Some(holdings) => {
            for holding in holdings {
                if creating.holdings.push(holding).is_err() {
                    creating.failed = true;
                }
            }
        }
        None => creating.failed = true,
    }
    if creating.remaining != 0 {
        return;
    }
    let creating = core.creating.remove(&request).expect("completed creation flight");
    if creating.failed {
        pool_refused(work, request, people::Refusal::Limit { which: people::LimitKind::Arithmetic });
        return;
    }
    let mut new = creating.new;
    new.holdings = creating.holdings.into_boxed();
    match creating.kind {
        CreationKind::Goal { person, direct: true } => {
            let (period, pool) = core.goal_open_pool(new.project, person);
            if let Some(event) = period {
                work.push(Event::Tasks(event));
            }
            if let Some(event) = pool {
                work.push(Event::Tasks(event));
            }
            assert!(core.made.insert(request, (new.number, true)) == Ok(None), "one goal make flight");
            work.push(Event::Tasks(tasks::Event::Make {
                reply_to: ReplyTo::new(request),
                creator: tasks::Party::Person(person),
                batch: Box::new([new]),
            }));
        }
        CreationKind::Goal { person, direct: false } => {
            let Some(proposal) = crate::fresh(&mut core.counters, crate::Family::Message) else {
                pool_refused(work, request, people::Refusal::Limit { which: people::LimitKind::Arithmetic });
                return;
            };
            assert!(
                core.goal_routes.insert(request, GoalRoute::Proposing { proposal }) == Ok(None),
                "one goal proposal flight"
            );
            work.push(Event::Tasks(tasks::Event::ProposePerson {
                reply_to: ReplyTo::new(request),
                proposal: tasks::PersonProposal {
                    number: proposal,
                    proposer: person,
                    project: new.project,
                    goal: new,
                    state: tasks::PersonProposalState::Pending { since: env.wall },
                },
            }));
        }
        CreationKind::Chat { person } => {
            assert!(core.made.insert(request, (new.number, false)) == Ok(None), "one chat make flight");
            work.push(Event::Tasks(tasks::Event::Make {
                reply_to: ReplyTo::new(request),
                creator: tasks::Party::Person(person),
                batch: Box::new([new]),
            }));
        }
    }
}

#[expect(clippy::too_many_arguments, reason = "one keyed role edit carries the candidate roster and its owner")]
pub(super) fn roles_route(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    out: &mut Queue<Request>,
    request: Token,
    person: u64,
    project: u32,
    holdings: Box<[people::Holding]>,
) {
    if let Err(why) = core.role_allowed(person, project, env.limits.tasks.tree_tasks) {
        pool_refused(work, request, why);
        return;
    }
    for holding in &holdings {
        match holding.role {
            people::Role::Policy { role } => {
                if core.authority.role(project, role).is_none() {
                    pool_refused(work, request, people::Refusal::Unknown);
                    return;
                }
            }
            people::Role::Owner | people::Role::Maintainer | people::Role::Member | people::Role::Observer => {}
        }
    }
    let contexts = match core.tasks.project_escalations(&env.limits.tasks, project) {
        Ok(contexts) => contexts,
        Err(refusal) => {
            assert!(refusal == tasks::Refusal::NotReady, "project inspection refusal is readiness");
            super::refusals::pool_again(work, request, people::Again::NotReady);
            return;
        }
    };
    let changed = match core.roles_preflight(&env.limits, &contexts, &holdings) {
        Ok(changed) => changed,
        Err(why) => {
            pool_refused(work, request, why);
            return;
        }
    };
    let handoffs = u32::try_from(contexts.len()).expect("bounded Waiting count").checked_add(1).expect("recheck room");
    drop(contexts);
    drop(holdings);
    if work.room() < handoffs.checked_add(env.limits.tasks.tasks).expect("validated recheck bound")
        || out.room() < changed.checked_add(2).expect("validated role cohort bound")
    {
        super::refusals::pool_again(work, request, people::Again::Busy);
        return;
    }
    if let Err(why) = core.role_allowed(person, project, env.limits.tasks.tree_tasks) {
        pool_refused(work, request, why);
        return;
    }
    core.touch_project_parties(project);
    let mut applied = Queue::with_capacity(people::max_out(&env.limits.people));
    people::step(
        &mut core.people,
        &Env { now: env.now, wall: env.wall, limits: env.limits.people },
        people::Event::ApplyRoles { reply_to: ReplyTo::new(request), request },
        &mut applied,
    );
    let mut terminal = None;
    for _ in 0..applied.len() {
        match applied.pop().expect("role application output count") {
            people::Request::Save { record } => {
                super::view_routes::touch_people_row(core, &record);
                out.push(Request::Write(Write::Save(Record::People(record))));
            }
            people::Request::RolesApplied { reply_to, request: actual, result } => {
                assert!(
                    reply_to.into_token() == request && actual == request && terminal.is_none(),
                    "one role terminal"
                );
                terminal = Some(result);
            }
            people::Request::Erase { .. }
            | people::Request::Route { .. }
            | people::Request::Reply { .. }
            | people::Request::RolesRefused { .. }
            | people::Request::ServiceMade { .. }
            | people::Request::RestoreRefused { .. } => unreachable!("role application has one save and terminal"),
        }
    }
    if let Err(why) = terminal.expect("role application terminal") {
        work.push(Event::People(people::Event::Decided { request, outcome: why }));
        return;
    }
    work.push(Event::Tasks(tasks::Event::RecheckEscalations { reply_to: ReplyTo::new(request), project }));
    let mut reroutes = core.proposal_recheck_project(&env.limits, project);
    for _ in 0..reroutes.len() {
        work.push(Event::Tasks(reroutes.pop().expect("bounded proposal recheck")));
    }
    work.push(Event::People(people::Event::Decided { request, outcome: people::Outcome::RolesSet { project } }));
}

pub(super) fn make_service_request(
    core: &mut Core,
    env: &Env<Limits>,
    request: Token,
    person: u64,
    project: u32,
    pending: &mut Queue<people::Request>,
    service_role: people::Role,
) {
    let outcome = match core.role_allowed(person, project, env.limits.tasks.tree_tasks) {
        Ok(()) if core.authority.role(project, service_role.number()).is_some() => {
            match crate::fresh(&mut core.counters, crate::Family::Person) {
                Some(candidate) => {
                    let mut follow = Queue::with_capacity(people::max_out(&env.limits.people));
                    people::step(
                        &mut core.people,
                        &Env { now: env.now, wall: env.wall, limits: env.limits.people },
                        people::Event::MakeService { request, person: candidate },
                        &mut follow,
                    );
                    for _ in 0..follow.len() {
                        pending.push(follow.pop().expect("service output count"));
                    }
                    None
                }
                None => Some(people::Outcome::Refused(people::Refusal::Limit { which: people::LimitKind::Arithmetic })),
            }
        }
        Ok(()) => Some(people::Outcome::Refused(people::Refusal::Unknown)),
        Err(refusal) => Some(people::Outcome::Refused(refusal)),
    };
    if let Some(outcome) = outcome {
        let mut follow = Queue::with_capacity(people::max_out(&env.limits.people));
        people::step(
            &mut core.people,
            &Env { now: env.now, wall: env.wall, limits: env.limits.people },
            people::Event::Decided { request, outcome },
            &mut follow,
        );
        for _ in 0..follow.len() {
            pending.push(follow.pop().expect("service decision count"));
        }
    }
}

#[expect(clippy::too_many_arguments, reason = "the route retains the admitted party and request fields")]
pub(super) fn adopt_request(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    request: Token,
    person: u64,
    project: u32,
    out: &mut Queue<Request>,
    adoption: people::Adoption,
) {
    match core.role_allowed(person, project, env.limits.tasks.tree_tasks) {
        Ok(()) => {
            out.push(Request::Ask {
                connector: adoption.resource.connector,
                ask: Ask::Adopt { request, project, adoption },
            });
        }
        Err(why) => {
            work.push(Event::People(people::Event::Decided { request, outcome: people::Outcome::Refused(why) }));
        }
    }
}

#[expect(clippy::too_many_arguments, reason = "the route retains the admitted party and request fields")]
pub(super) fn change_policy_request(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    request: Token,
    person: u64,
    project: u32,
    out: &mut Queue<Request>,
    change: people::PolicyChange,
) {
    if work.room() < env.limits.tasks.tasks.checked_add(1).expect("task recheck bound") || out.room() < 2 {
        super::refusals::pool_again(work, request, people::Again::Busy);
        return;
    }
    match core.change_policy(&env.limits, person, project, change) {
        Ok(snapshot) => {
            core.touch_project_objects(project);
            out.push(Request::Write(Write::Save(Record::People(people::Stored::Policy { project, value: snapshot }))));
            let mut reroutes = core.proposal_recheck_project(&env.limits, project);
            for _ in 0..reroutes.len() {
                work.push(Event::Tasks(reroutes.pop().expect("policy holder recheck")));
            }
            work.push(Event::People(people::Event::Decided {
                request,
                outcome: people::Outcome::PolicyChanged { project },
            }));
        }
        Err(why) => pool_refused(work, request, why),
    }
}
