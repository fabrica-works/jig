//! Factual direct bounds and the policy holder for supported widening drafts.
use super::{Admission, Beyond, Bounds, Recipient, standing};
use crate::translate;
use crate::{Core, Limits};
use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{List, Queue, Wall};

pub(crate) fn recipient(core: &Core, project: u32, kind: authority::ProposalKind) -> Option<Recipient> {
    let policy = core.authority.policy(project)?;
    let mut roles = List::with_capacity(core.authority.limits().roles);
    for role in &policy.roles {
        if role.decides.allows(kind) {
            roles.push(role.number).expect("policy roles fit the admitted authority bound");
        }
    }
    if roles.is_empty() { None } else { Some(Recipient::Roles { project, roles }) }
}

pub(crate) fn goal(core: &Core, limits: &Limits, party: u64, project: u32, held: Option<people::Role>) -> Admission {
    let Some(held) = held else {
        return Admission::Refused(people::Refusal::Role { held });
    };
    let Some(role) = core.authority.role(project, held.number()) else {
        return Admission::Refused(people::Refusal::Role { held: Some(held) });
    };
    if !role.requests.allows(authority::RequestKind::Create) {
        return Admission::Refused(people::Refusal::Role { held: Some(held) });
    }
    let (_, pool) = core.goal_pool(project, party);
    let pool_left =
        pool.budget.saturating_sub(pool.spent.saturating_add(pool.spent_below).saturating_add(pool.reserved));
    let budget = pool_left.min(role.authority.budget.spend);
    let policy = core.authority.policy(project).expect("role belongs to policy");
    let maximum_budget = policy
        .period_spend
        .min(core.authority.rules().period_spend)
        .min(policy.ceiling.budget.spend)
        .min(core.authority.rules().ceiling.budget.spend);
    let maximum_deadline = earliest(policy.ceiling.budget.deadline, core.authority.rules().ceiling.budget.deadline);
    let mut charters = List::with_capacity(limits.documents.lines);
    let canonical_deadline = earliest(role.authority.budget.deadline, maximum_deadline);
    for charter in &core.settings.charters {
        if goal_charter(core, limits, party, project, held, charter.number) && charters.room() > 0 {
            charters.push(charter.number).expect("bounded offered charters");
        }
    }
    let mut procedures = List::with_capacity(limits.documents.lines);
    if core.projections.len() < limits.tasks.tasks {
        for kind in &role.authority.delegation.kinds {
            let authority::Executor::Procedure(procedure) = kind else { continue };
            for &connector in &core.connectors {
                let executor = people::GoalExecutor::Procedure { connector, code: *procedure };
                let supported =
                    goal_start(core, project, party, held, executor, 0, canonical_deadline, limits.tasks.tree_tasks)
                        .is_ok();
                if supported && procedures.push(super::GoalProcedure { connector, code: *procedure }).is_err() {
                    return Admission::Refused(people::Refusal::Limit { which: people::LimitKind::Authority });
                }
            }
        }
    }
    let beyond = match recipient(core, project, authority::ProposalKind::Batch) {
        Some(to) => Beyond::Proposed { to },
        None => Beyond::Refused(unproposable()),
    };
    Admission::Bounded {
        within: Bounds::Goal {
            pool_left,
            charters,
            procedures,
            latest: role.authority.budget.deadline,
            budget,
            maximum_budget,
            maximum_deadline,
        },
        beyond,
    }
}

pub(super) fn amend_preflight(
    core: &Core,
    limits: &Limits,
    party: u64,
    task: u64,
    held: Option<people::Role>,
) -> Result<(), people::Refusal> {
    amend_preflight_in(core, limits, party, task, held, &tasks::CompoundAdmission::new(&core.tasks, 0))
}
pub(crate) fn amend_preflight_in(
    core: &Core,
    limits: &Limits,
    party: u64,
    task: u64,
    held: Option<people::Role>,
    admission: &tasks::CompoundAdmission<'_>,
) -> Result<(), people::Refusal> {
    let row = admission.task(task).ok_or(people::Refusal::Unknown)?;
    let empty = tasks::Amendment {
        tracked: None,
        addressed: None,
        instructions: None,
        spec: None,
        wake: None,
        dependencies: None,
        authority: None,
        release: None,
        reason: Box::new([]),
    };
    amendment_admit_in(
        core,
        crate::compound::PartyTask { person: party, role: held, project: row.project, task },
        &empty,
        limits.tasks.depth,
        &limits.tasks,
        admission,
    )?;
    Ok(())
}

pub(super) fn amend(core: &Core, limits: &Limits, party: u64, task: u64, held: Option<people::Role>) -> Admission {
    let Some(row) = core.tasks.task(task) else {
        return Admission::Refused(people::Refusal::Unknown);
    };
    if let Err(why) = amend_preflight(core, limits, party, task, held) {
        return Admission::Refused(why);
    }
    let held = held.expect("amend admission found role");
    let role = core.authority.role(row.project, held.number()).expect("amend admission found role policy");
    // Amendment admission uses an already-opened pool; a configured future
    // pool is not available until a goal opens it.
    let source = tasks::Funder::Pool { project: row.project, person: party, period: core.period(row.project) };
    let pool = match core.tasks.funding(source) {
        Some(row) => row.numbers,
        None => tasks::Numbers { budget: 0, spent: 0, spent_below: 0, reserved: 0 },
    };
    let pool_left =
        pool.budget.saturating_sub(pool.spent.saturating_add(pool.spent_below).saturating_add(pool.reserved));
    let budget_left = role.authority.budget.spend.min(pool_left).saturating_sub(row.authority.budget.spend);
    let mut given = crate::task_authority(&role.authority);
    given.budget.spend = given.budget.spend.min(pool_left);
    given.delegation.tasks = given.delegation.tasks.min(core.tasks.delegation(task).expect("task context").tasks_left);
    let policy = core.authority.policy(row.project).expect("role belongs to policy");
    let mut hard = crate::task_authority(&policy.ceiling);
    hard.budget.spend = hard.budget.spend.min(pool_left);
    // Period permission is a fixed preflight, not an editable amount. Existing
    // authority may still narrow; no widening draft is supported here.
    if pool.budget > role.period_spend {
        given = row.authority.clone();
        hard = row.authority.clone();
    }
    let Ok(value) = crate::documents::act_authority(limits, &given) else {
        return Admission::Refused(people::Refusal::Limit { which: people::LimitKind::Authority });
    };
    let Ok(ceiling) = crate::documents::act_authority(limits, &hard) else {
        return Admission::Refused(people::Refusal::Limit { which: people::LimitKind::Authority });
    };
    let order = &core.authority.rules().implies;
    if order.len() > limits.documents.lines {
        return Admission::Refused(people::Refusal::Limit { which: people::LimitKind::Authority });
    }
    let mut implications = List::with_capacity(limits.documents.lines);
    for pair in order.entries() {
        implications.push(*pair).expect("complete configured relation fits receiving bound");
    }
    let beyond = match recipient(core, row.project, authority::ProposalKind::Amend) {
        Some(to) => Beyond::Proposed { to },
        None => Beyond::Refused(unproposable()),
    };
    Admission::Bounded {
        within: Bounds::Amend {
            budget_left,
            grants: value.grants.clone(),
            implications,
            authority: Box::new(value),
            ceiling: Box::new(ceiling),
        },
        beyond,
    }
}

fn earliest(left: Option<Wall>, right: Option<Wall>) -> Option<Wall> {
    match left {
        Some(left) => match right {
            Some(right) => Some(left.min(right)),
            None => Some(left),
        },
        None => right,
    }
}

/// Sample the same full goal judge for a configured charter's form fact.
pub(crate) fn goal_charter(
    core: &Core,
    limits: &Limits,
    party: u64,
    project: u32,
    held: people::Role,
    charter: u32,
) -> bool {
    if core.projections.len() >= limits.tasks.tasks {
        return false;
    }
    let Some(policy) = core.authority.policy(project) else { return false };
    let Some(role) = core.authority.role(project, held.number()) else { return false };
    let deadline = earliest(
        role.authority.budget.deadline,
        earliest(policy.ceiling.budget.deadline, core.authority.rules().ceiling.budget.deadline),
    );
    goal_start(core, project, party, held, people::GoalExecutor::Charter(charter), 0, deadline, limits.tasks.tree_tasks)
        .is_ok()
}

/// The goal's one full judge, also sampled for fixed supported form facts.
#[expect(clippy::too_many_arguments, reason = "a goal carries its requested authority and deadline")]
pub(crate) fn goal_start(
    core: &Core,
    project: u32,
    person: u64,
    role: people::Role,
    executor: people::GoalExecutor,
    budget: u64,
    deadline: Option<Wall>,
    tasks_left: u32,
) -> Result<crate::GoalStart, people::Refusal> {
    standing::role(core, project, Some(role), super::Act::SetGoal)?;
    if !core.authority.escalations_held(project, role.number()) {
        return Err(people::Refusal::NoEscalationHolder);
    }
    let Some(policy) = core.authority.policy(project) else { return Err(people::Refusal::Unknown) };
    let Some(role_policy) = core.authority.role(project, role.number()) else {
        return Err(people::Refusal::Role { held: Some(role) });
    };
    if !role_policy.requests.allows(authority::RequestKind::Create) {
        return Err(people::Refusal::Role { held: Some(role) });
    }
    let mut given = core.chat_authority(project, person, role, tasks_left);
    given.budget.spend = budget;
    given.budget.deadline = deadline;
    let executor = match executor {
        people::GoalExecutor::Charter(charter) => {
            if core.charter(charter).is_none() {
                return Err(people::Refusal::NotOffered);
            }
            authority::Executor::Charter(charter)
        }
        people::GoalExecutor::Procedure { connector, code } => {
            if code == 0 || !core.connectors.contains(&connector) {
                return Err(people::Refusal::NotOffered);
            }
            given.tools = authority::Tools(0);
            authority::Executor::Procedure(code)
        }
    };
    let period_cap = policy.period_spend.min(core.authority.rules().period_spend);
    if budget > period_cap {
        return Err(people::Refusal::Authority {
            proposable: None,
            lacking: people::Lack::Budget { needs: budget, left: period_cap },
        });
    }
    let hard = !authority::at_most(&given, &policy.ceiling, &core.authority.rules().implies);
    let (source, numbers) = core.goal_pool(project, person);
    let mut findings = Queue::with_capacity(authority::max_out(core.authority.limits()).expect("authority room"));
    let ask = authority::PersonAsk {
        project,
        role: role.number(),
        pool: translate::authority_numbers(numbers),
        tasks_left,
        request: authority::PersonRequest::Create(Box::new([authority::Delegate {
            executor,
            authority: given.clone(),
            symbolic: Box::new([]),
        }])),
    };
    let checked = authority::check_request(&core.authority, &ask, &mut findings);
    if hard || checked.answer != authority::Answer::Allow {
        return Err(core.request_refusal(&ask, checked.answer, &mut findings));
    }
    Ok(crate::GoalStart { source, authority: given })
}

/// The amendment's one full judge, used for admission and form standing.
#[expect(clippy::too_many_arguments, reason = "an amendment carries its current task, authority and child bounds")]
pub(crate) fn amendment_admit(
    core: &Core,
    person: u64,
    role: Option<people::Role>,
    project: u32,
    task: u64,
    amendment: &tasks::Amendment,
    depth: u32,
    limits: &tasks::Limits,
) -> Result<bool, people::Refusal> {
    amendment_admit_in(
        core,
        crate::compound::PartyTask { person, project, role, task },
        amendment,
        depth,
        limits,
        &tasks::CompoundAdmission::new(&core.tasks, 0),
    )
}
pub(crate) fn amendment_admit_in(
    core: &Core,
    party: crate::compound::PartyTask,
    amendment: &tasks::Amendment,
    depth: u32,
    limits: &tasks::Limits,
    admission: &tasks::CompoundAdmission<'_>,
) -> Result<bool, people::Refusal> {
    let crate::compound::PartyTask { person, project, role, task } = party;
    standing::role(core, project, role, super::Act::Amend)?;
    if amendment.tracked.is_some() {
        standing::role(core, project, role, super::Act::Prioritise)?;
    }
    let Some(context) = admission.task(task) else { return Err(people::Refusal::Unknown) };
    let Some(role) = role else { return Err(people::Refusal::Role { held: None }) };
    let standing = Core::person_tree_in(person, task, depth, admission);
    if context.project != project || !core.reaches_project(role, project) && !standing {
        return Err(people::Refusal::Unknown);
    }
    let Some(role_policy) = core.authority.role(project, role.number()) else {
        return Err(people::Refusal::Role { held: Some(role) });
    };
    if !role_policy.requests.allows(authority::RequestKind::Amend) && !standing {
        return Err(people::Refusal::Role { held: Some(role) });
    }
    let row = &context;
    if let Err(why) = admission.address_admit(task, &amendment.addressed) {
        let people::Outcome::Refused(why) = crate::routing::refusals::person_task_refusal(core, Some(task), why) else {
            unreachable!("address refusal is permanent");
        };
        return Err(why);
    }
    if let Err(why) = tasks::instruction_admit(limits, row.executor, &amendment.instructions) {
        let people::Outcome::Refused(why) = crate::routing::refusals::person_task_refusal(core, Some(task), why) else {
            unreachable!("instruction shape refusal is permanent");
        };
        return Err(why);
    }
    let Some(after) = &amendment.authority else { return Ok(false) };
    if !tasks::valid_authority(limits, after) {
        return Err(people::Refusal::Limit { which: people::LimitKind::Authority });
    }
    let before = translate::authority_value(context.authority);
    let after = translate::authority_value(after);
    let implies = &core.authority.rules().implies;
    let stop_run = !authority::at_most(&before, &after, implies);
    if authority::at_most(&after, &before, implies) {
        return Ok(stop_run);
    }
    let pool = tasks::Funder::Pool { project, person, period: core.period(project) };
    let numbers = match admission.funding(pool) {
        Some(numbers) => numbers,
        None => tasks::Numbers { budget: 0, spent: 0, spent_below: 0, reserved: 0 },
    };
    let mut findings = Queue::with_capacity(authority::max_out(core.authority.limits()).expect("authority bound"));
    let ask = authority::PersonAsk {
        project,
        role: role.number(),
        pool: translate::authority_numbers(numbers),
        tasks_left: context.tasks_left().ok_or(people::Refusal::Limit { which: people::LimitKind::Arithmetic })?,
        request: authority::PersonRequest::Amend(after),
    };
    let checked = authority::check_request_with_standing(&core.authority, &ask, standing, &mut findings);
    match checked.answer {
        authority::Answer::Allow => Ok(stop_run),
        authority::Answer::Propose => Err(core.request_refusal(&ask, checked.answer, &mut findings)),
        authority::Answer::Wait | authority::Answer::Refuse => {
            Err(core.request_refusal(&ask, checked.answer, &mut findings))
        }
    }
}

/// The same static absence of accepting authority is reported before a draft
/// is sent and when its admission would otherwise make a party proposal.
fn unproposable() -> people::Refusal {
    people::Refusal::Authority { lacking: people::Lack::Approver, proposable: Some(people::Accepter::Nobody) }
}

/// Only a newly admitted proposal needs an accepting policy role (tasks.md, 9).
pub(crate) fn proposal_has_recipient(core: &Core, holder: tasks::ProposalHolder) -> bool {
    let (project, kind) = match holder {
        tasks::ProposalHolder::Policy { project, kind } => (project, kind),
        tasks::ProposalHolder::Task(_) | tasks::ProposalHolder::Person(_) => return true,
    };
    let kind = match kind {
        tasks::ProposalKind::Batch => authority::ProposalKind::Batch,
        tasks::ProposalKind::Effect => authority::ProposalKind::Effect,
        tasks::ProposalKind::Amend => authority::ProposalKind::Amend,
        tasks::ProposalKind::Widen => authority::ProposalKind::Widen,
        tasks::ProposalKind::Release => authority::ProposalKind::Escalation,
    };
    recipient(core, project, kind).is_some()
}
