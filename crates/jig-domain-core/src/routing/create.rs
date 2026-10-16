//! Party roots checked whole, then made together (domain/people.md, 5.1).
use super::{CreationKind, parties::begin_creation, refusals::pool_refused};
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
    creation: Creation,
) {
    let request = creation.request;
    match checked(core, env, &creation) {
        Ok(members) => {
            begin_creation(core, env, out, work, request, CreationKind::Batch { person: creation.person }, members);
        }
        Err(why) => pool_refused(work, request, why),
    }
}
fn checked(core: &mut Core, env: &Env<Limits>, creation: &Creation) -> Result<Box<[tasks::New]>, people::Refusal> {
    let Creation { person, project, role, batch, .. } = creation;
    if let Some(at) = cycle(batch) {
        return Err(people::Refusal::NotWellFormed { at, why: people::Malformed::Cycle });
    }
    if !core.authority.escalations_held(*project, role.number()) {
        return Err(people::Refusal::NoEscalationHolder);
    }
    let mut delegates = List::with_capacity(env.limits.people.batch);
    let mut members = List::with_capacity(env.limits.people.batch);
    let tasks = core.counters.deployment().tasks;
    let first = tasks.checked_add(1).ok_or(people::Refusal::Limit { which: people::LimitKind::Arithmetic })?;
    for member in batch {
        let new = new_task(core, env, creation, member, first, members.len())?;
        let tasks::Executor::Agent { charter } = new.executor else { unreachable!("agent batch") };
        delegates
            .push(authority::Delegate {
                executor: authority::Executor::Charter(charter),
                authority: crate::translate::authority_value(&new.authority),
                symbolic: Box::new([]),
            })
            .expect("admitted batch");
        members.push(new).expect("admitted batch");
    }
    let (_, pool) = core.goal_pool(*project, *person);
    let ask = authority::PersonAsk {
        project: *project,
        role: role.number(),
        pool: crate::translate::authority_numbers(pool),
        tasks_left: env.limits.tasks.tree_tasks,
        request: authority::PersonRequest::Create(delegates.into_boxed()),
    };
    let mut findings = Queue::with_capacity(authority::max_out(core.authority.limits()).expect("authority output"));
    if authority::check_request(&core.authority, &ask, &mut findings).answer != authority::Answer::Allow {
        return Err(core.request_refusal(&ask, &mut findings));
    }
    let policy = core.authority.policy(*project).expect("checked project");
    if core.period_budget(*project) > policy.period_spend {
        return Err(people::Refusal::Authority {
            lacking: people::Lack::Budget { needs: core.period_budget(*project), left: policy.period_spend },
            proposable: None,
        });
    }
    for _ in &members {
        crate::fresh(&mut core.counters, crate::Family::Task)
            .ok_or(people::Refusal::Limit { which: people::LimitKind::Arithmetic })?;
    }
    Ok(members.into_boxed())
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
    let people::TaskExecutor::Agent { charter } = member.executor;
    if core.charter(charter).is_none() {
        return Err(people::Refusal::NotOffered);
    }
    let Some(given) = crate::amendments::granted(member.authority.clone(), &env.limits.tasks) else {
        return Err(people::Refusal::Limit { which: people::LimitKind::Authority });
    };
    let spec = crate::amendments::spec(
        people::Spec { words: member.words.clone(), parameters: member.parameters.clone(), inputs: Box::new([]) },
        &env.limits.tasks,
    )
    .ok_or(people::Refusal::Limit { which: people::LimitKind::Spec })?;
    let mut dependencies = List::with_capacity(env.limits.tasks.dependencies);
    for &at in &member.dependencies {
        if usize::try_from(at).expect("u32 fits usize") >= batch.len() {
            return Err(people::Refusal::Unknown);
        }
        if dependencies
            .push(
                first
                    .checked_add(u64::from(at))
                    .ok_or(people::Refusal::Limit { which: people::LimitKind::Arithmetic })?,
            )
            .is_err()
        {
            return Err(people::Refusal::Limit { which: people::LimitKind::Dependencies });
        }
    }
    Ok(tasks::New {
        number: first
            .checked_add(u64::from(at))
            .ok_or(people::Refusal::Limit { which: people::LimitKind::Arithmetic })?,
        project: *project,
        executor: tasks::Executor::Agent { charter },
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
                for &next in &member.dependencies {
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
