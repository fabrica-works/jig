//! Party roots checked whole, then made together (domain/people.md, 5.1).
use super::{party_inputs, refusals::pool_refused};
use crate::{Core, Event, Limits, Request};
use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Env, List, Queue, Token};
/// Authenticated creation envelope.
pub(super) struct Creation {
    pub request: Token,
    pub person: u64,
    pub project: u32,
    pub role: people::Role,
    pub batch: Box<[people::Member]>,
}
pub(super) fn create_request(
    core: &mut Core,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
    work: &mut Queue<Event>,
    mut creation: Creation,
) {
    let request = creation.request;
    if core.fill >= env.limits.fill_mark {
        super::refusals::pool_again(work, request, people::Again::Busy);
        return;
    }
    match checked(core, env, &mut creation, false) {
        Ok((members, given)) => {
            party_inputs::begin(
                core,
                env,
                out,
                work,
                party_inputs::Inputs {
                    person: creation.person,
                    project: creation.project,
                    role: creation.role,
                    members,
                    given,
                    proposal: None,
                    ids: Box::new([]),
                    at: 0,
                    row: None,
                    stubs: List::with_capacity(0),
                },
                request,
            );
        }
        Err(why) => pool_refused(work, request, why),
    }
}
pub(super) fn checked(
    core: &mut Core,
    env: &Env<Limits>,
    creation: &mut Creation,
    proposing: bool,
) -> Result<(Box<[tasks::New]>, Option<people::Given>), people::Refusal> {
    let allowed = if proposing { None } else { budget(core, creation)? };
    let Creation { person, project, role, batch, .. } = &*creation;
    well_formed(batch)?;
    if let Some(at) = cycle(batch) {
        return Err(people::Refusal::NotWellFormed { at, why: people::Malformed::Cycle });
    }
    if !proposing && !core.authority.escalations_held(*project, role.number()) {
        return Err(people::Refusal::NoEscalationHolder);
    }
    let mut delegates = List::with_capacity(env.limits.people.batch);
    let mut members = List::with_capacity(env.limits.people.batch);
    let tasks = core.counters.deployment().tasks;
    let first = tasks.checked_add(1).ok_or(people::Refusal::Limit { which: people::LimitKind::Arithmetic })?;
    for member in batch {
        let new = new_task(core, env, creation, member, first, members.len())?;
        let executor = core.delegation_executor(*project, new.executor).ok_or(people::Refusal::Unknown)?;
        delegates
            .push(authority::Delegate {
                executor,
                authority: crate::translate::authority_value(
                    &crate::amendments::granted(member.authority.clone(), &env.limits.tasks)
                        .expect("member authority checked"),
                ),
                symbolic: crate::symbolic_grants(&named(member, env)?, core.authority.limits().grants)
                    .ok_or(people::Refusal::Limit { which: people::LimitKind::Authority })?,
            })
            .expect("admitted batch");
        members.push(new).expect("admitted batch");
    }
    if !core.authority.escalations_held(*project, role.number()) {
        return Err(people::Refusal::NoEscalationHolder);
    }
    if proposing {
        super::propose::allowed(core, *project, delegates.into_boxed())?;
    } else {
        let (_, pool) = core.person_pool(*project, *person);
        let ask = authority::PersonAsk {
            project: *project,
            role: role.number(),
            pool: crate::translate::authority_numbers(pool),
            tasks_left: env.limits.tasks.tree_tasks,
            request: authority::PersonRequest::Create(delegates.into_boxed()),
        };
        let mut findings = Queue::with_capacity(authority::max_out(core.authority.limits()).expect("authority output"));
        let checked = authority::check_request(&core.authority, &ask, &mut findings);
        if checked.answer != authority::Answer::Allow {
            let mut refusal = core.request_refusal(&ask, checked.answer, &mut findings);
            if let people::Refusal::Authority { lacking: people::Lack::TaskKind { executor }, .. } = &mut refusal
                && let Some(policy) = core.authority.role(*project, role.number())
            {
                for (member, delegate) in batch.iter().zip(delegates_for(&ask)) {
                    if !policy.authority.delegation.kinds.contains(&delegate.executor) {
                        *executor = Some(member.executor);
                        break;
                    }
                }
            }
            return Err(refusal);
        }
        let policy = core.authority.policy(*project).expect("checked project");
        if core.period_budget(*project) > policy.period_spend {
            return Err(people::Refusal::Authority {
                lacking: people::Lack::Budget { needs: core.period_budget(*project), left: policy.period_spend },
                proposable: None,
            });
        }
    }
    for _ in &members {
        crate::fresh(&mut core.counters, crate::Family::Task)
            .ok_or(people::Refusal::Limit { which: people::LimitKind::Arithmetic })?;
    }
    let mut given = None;
    if let Some((at, amount)) = allowed {
        given = Some(people::Given { task: first.checked_add(u64::from(at)).expect("admitted task number"), amount });
    }
    Ok((members.into_boxed(), given))
}
fn new_task(
    core: &Core,
    env: &Env<Limits>,
    creation: &Creation,
    member: &people::Member,
    first: u64,
    at: u32,
) -> Result<tasks::New, people::Refusal> {
    let Creation { person, project, batch, .. } = creation;
    let executor = executor(core, member.executor)?;
    let Some(given) = crate::amendments::granted(member.authority.clone(), &env.limits.tasks) else {
        return Err(people::Refusal::Limit { which: people::LimitKind::Authority });
    };
    let spec = crate::amendments::spec(
        people::Spec {
            words: member.words.clone(),
            parameters: member.parameters.clone(),
            inputs: member.inputs.clone(),
        },
        &env.limits.tasks,
    )
    .ok_or(people::Refusal::Limit { which: people::LimitKind::Spec })?;
    let mut dependencies = List::with_capacity(env.limits.tasks.dependencies);
    for dependency in &member.dependencies {
        let number = match *dependency {
            people::Dependency::Batch(at) => {
                if usize::try_from(at).expect("u32 fits usize") >= batch.len() {
                    return Err(people::Refusal::Unknown);
                }
                first
                    .checked_add(u64::from(at))
                    .ok_or(people::Refusal::Limit { which: people::LimitKind::Arithmetic })?
            }
            people::Dependency::Existing(task) => {
                if !core.reaches(*person, creation.role, *project, task, env.limits.tasks.depth) {
                    return Err(people::Refusal::Unknown);
                }
                task
            }
        };
        if dependencies.push(number).is_err() {
            return Err(people::Refusal::Limit { which: people::LimitKind::Dependencies });
        }
    }
    let number =
        first.checked_add(u64::from(at)).ok_or(people::Refusal::Limit { which: people::LimitKind::Arithmetic })?;
    let given = crate::resolved_delegate_authority(&given, &named(member, env)?, number, core.authority.limits())
        .ok_or(people::Refusal::Limit { which: people::LimitKind::Authority })?;
    Ok(tasks::New {
        deadline: match member.deadline {
            people::Deadline::Named => tasks::Deadline::Named,
        },
        number,
        project: *project,
        executor,
        spec,
        contract: contract(&member.contract),
        authority: given,
        numbers: tasks::Numbers { budget: member.authority.spend, spent: 0, spent_below: 0, reserved: 0 },
        funder: tasks::Funder::Pool { project: *project, person: *person, period: core.period(*project) },
        dependencies: dependencies.into_boxed(),
        holdings: Box::new([]),
        wake: crate::amendments::wake(member.wake),
        recurring: None,
        tracked: member.tracked,
        label: member.label,
        flags: tasks::Flags {
            conversation: member.flags.conversation,
            listed: member.flags.listed,
            retention: match member.flags.retention {
                people::Retention::Kept => tasks::Retention::Kept,
                people::Retention::ForgottenOnceRead => tasks::Retention::ForgottenOnceRead,
            },
        },
    })
}

fn contract(value: &people::Contract) -> tasks::Contract {
    match value {
        people::Contract::Change { connector, kind, words } => {
            tasks::Contract::Change { connector: *connector, kind: *kind, words: *words }
        }
        people::Contract::Report { words } => tasks::Contract::Report { words: *words },
        people::Contract::Verdict { choices } => {
            let mut rows = List::with_capacity(u32::try_from(choices.len()).expect("bounded choices"));
            for choice in choices {
                let mut fields = List::with_capacity(u32::try_from(choice.fields.len()).expect("bounded fields"));
                for field in &choice.fields {
                    fields.push(tasks::FieldRule { name: field.name.clone(), max: field.max }).expect("field count");
                }
                rows.push(tasks::Verdict {
                    code: choice.code,
                    words: choice.words,
                    followups: choice.followups,
                    fields: fields.into_boxed(),
                })
                .expect("choice count");
            }
            tasks::Contract::Verdict { choices: rows.into_boxed() }
        }
    }
}
fn cycle(batch: &[people::Member]) -> Option<u32> {
    for (at, _) in batch.iter().enumerate() {
        let mut reached = List::with_capacity(u32::try_from(batch.len()).expect("bounded batch"));
        for _ in batch {
            reached.push(false).expect("one mark per member");
        }
        *reached.get_mut(u32::try_from(at).expect("bounded position")).expect("member mark") = true;
        for _ in batch {
            for (position, member) in batch.iter().enumerate() {
                if !*reached.get(u32::try_from(position).expect("bounded position")).expect("member mark") {
                    continue;
                }
                for dependency in &member.dependencies {
                    let people::Dependency::Batch(next) = *dependency else { continue };
                    if usize::try_from(next).expect("u32 fits usize") == at {
                        return Some(u32::try_from(at).expect("bounded position"));
                    }
                    if let Some(mark) = reached.get_mut(next) {
                        *mark = true;
                    }
                }
            }
        }
    }
    None
}

fn well_formed(batch: &[people::Member]) -> Result<(), people::Refusal> {
    let mut allowed = false;
    for (at, member) in batch.iter().enumerate() {
        let agent = match member.executor {
            people::TaskExecutor::Agent { .. } => true,
            people::TaskExecutor::Procedure { .. } | people::TaskExecutor::Person(_) => false,
        };
        let as_allowed = match member.budget {
            people::Budget::Named => false,
            people::Budget::AsAllowed { .. } => true,
        };
        let why = if member.flags.conversation && !agent {
            Some(people::Malformed::Conversation)
        } else if as_allowed {
            if allowed {
                Some(people::Malformed::AsAllowedTwice)
            } else {
                allowed = true;
                None
            }
        } else {
            None
        };
        if let Some(why) = why {
            return Err(people::Refusal::NotWellFormed { at: u32::try_from(at).expect("bounded member"), why });
        }
    }
    Ok(())
}
fn executor(core: &Core, executor: people::TaskExecutor) -> Result<tasks::Executor, people::Refusal> {
    Ok(match executor {
        people::TaskExecutor::Agent { charter } => {
            if core.charter(charter).is_none() {
                return Err(people::Refusal::NotOffered);
            }
            tasks::Executor::Agent { charter }
        }
        people::TaskExecutor::Procedure { connector, code } => {
            if code == 0 || !core.connectors.contains(&connector) {
                return Err(people::Refusal::NotOffered);
            }
            tasks::Executor::Procedure { connector, code }
        }
        people::TaskExecutor::Person(address) => tasks::Executor::Person(match address {
            people::PersonAddress::Person(person) => tasks::PersonAddress::Person(person),
            people::PersonAddress::Role(role) => tasks::PersonAddress::Role(role),
        }),
    })
}
fn named(member: &people::Member, env: &Env<Limits>) -> Result<Box<[tasks::Grant]>, people::Refusal> {
    let mut authority = member.authority.clone();
    authority.grants.clone_from(&member.named);
    match crate::amendments::granted(authority, &env.limits.tasks) {
        Some(value) => Ok(value.grants),
        None => Err(people::Refusal::Limit { which: people::LimitKind::Authority }),
    }
}
fn delegates_for(ask: &authority::PersonAsk) -> &[authority::Delegate] {
    let authority::PersonRequest::Create(batch) = &ask.request else { unreachable!("party creation") };
    batch
}

fn budget(core: &Core, creation: &mut Creation) -> Result<Option<(u32, u64)>, people::Refusal> {
    well_formed(&creation.batch)?;
    let mut named = 0_u64;
    for member in &creation.batch {
        if let people::Budget::Named = member.budget {
            named = named
                .checked_add(member.authority.spend)
                .ok_or(people::Refusal::Limit { which: people::LimitKind::Arithmetic })?;
        }
    }
    let (_, pool) = core.person_pool(creation.project, creation.person);
    let ask = authority::PersonAsk {
        project: creation.project,
        role: creation.role.number(),
        pool: crate::translate::authority_numbers(pool),
        tasks_left: 0,
        request: authority::PersonRequest::Create(Box::new([])),
    };
    for (at, member) in creation.batch.iter_mut().enumerate() {
        if let people::Budget::AsAllowed { cap } = member.budget {
            let amount = authority::as_allowed(&core.authority, &ask, named, cap);
            if amount == 0 {
                return Err(people::Refusal::Authority {
                    lacking: people::Lack::Budget {
                        needs: named.saturating_add(1),
                        left: authority::as_allowed(&core.authority, &ask, 0, None),
                    },
                    proposable: None,
                });
            }
            member.authority.spend = amount;
            return Ok(Some((u32::try_from(at).expect("bounded member"), amount)));
        }
    }
    Ok(None)
}
