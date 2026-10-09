//! One whole chat close and its durable finish grace (domain/people.md, section 7;
//! domain/tasks.md, section 7). Every disposal and reservation is checked before
//! mutation. The task alarm reprojects the saved wall deadline after restart.
use crate::domain::{Domain, entrance, record, refused, task_mut};
use crate::{Change, Executor, Funder, Limits, Party, Phase, Refusal, Request, Was};
use skein_lib::{Env, List, Queue, ReplyTo, Wall};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Keep {
    Cancel,
    MakeMine,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Disposal {
    pub task: u64,
    pub keep: Keep,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Grace {
    pub by: u64,
    pub until: Wall,
}

pub(crate) fn mutable(phase: &Phase) -> bool {
    match phase {
        Phase::Waiting | Phase::Active(_) | Phase::Held { was: Was::Waiting | Was::Active(_), .. } => true,
        Phase::Closing(_) | Phase::Held { was: Was::Closing(_), .. } | Phase::Ended(_) => false,
    }
}

const FINISH: &[u8] = b"Finish this chat.";
const EXPIRED: &[u8] = b"Chat finish grace expired.";
const CANCEL: &[u8] = b"Chat delegate cancelled.";

#[expect(clippy::too_many_arguments, reason = "one whole chat close carries its funding and durable finish request")]
#[expect(clippy::too_many_lines, reason = "all cohort admission precedes any close mutation")]
pub(crate) fn close(
    domain: &mut Domain,
    env: &Env<Limits>,
    to: ReplyTo,
    number: u64,
    person: u64,
    message: u64,
    until: Wall,
    disposals: List<Disposal>,
    period: u64,
    pool_budget: u64,
    period_budget: u64,
    out: &mut Queue<Request>,
) {
    let to = match entrance(domain, to, number) {
        Ok(to) => to,
        Err((to, why)) => return refused(to, Some(number), why, out),
    };
    let old = record(domain, number).expect("live chat");
    let agent = match old.executor {
        Executor::Agent { .. } => true,
        Executor::Person(_) | Executor::Procedure { .. } => false,
    };
    if !agent
        || old.requester != Party::Person(person)
        || person == 0
        || !mutable(&old.phase)
        || old.chat_close.is_some()
        || until <= env.wall
        || message == 0
        || message <= old.last_message
        || FINISH.len() > usize::try_from(env.limits.message_bytes).expect("message bound")
        || CANCEL.len() > usize::try_from(env.limits.result_bytes).expect("reason bound")
        || EXPIRED.len() > usize::try_from(env.limits.result_bytes).expect("reason bound")
    {
        return refused(to, Some(number), Refusal::State, out);
    }
    if usize::try_from(disposals.len()).expect("bounded disposals") != old.delegates.len()
        || disposals.len() > env.limits.delegates
    {
        return refused(to, Some(number), Refusal::Reference, out);
    }
    let project = old.project;
    let mut plans = List::with_capacity(env.limits.delegates);
    let mut reserved = 0_u64;
    let mut parent_debit = 0_u64;
    let mut parent_spent = 0_u64;
    let mut references = old.references.len();
    for (at, disposal) in disposals.iter().enumerate() {
        if !old.delegates.contains(&disposal.task) {
            return refused(to, Some(disposal.task), Refusal::Reference, out);
        }
        for previous in disposals.iter().take(at) {
            if previous.task == disposal.task {
                return refused(to, Some(disposal.task), Refusal::Reference, out);
            }
        }
        match disposal.keep {
            Keep::Cancel => {}
            Keep::MakeMine => {
                let plan = match crate::moving::check(
                    domain,
                    env,
                    disposal.task,
                    person,
                    period,
                    pool_budget,
                    period_budget,
                    FINISH,
                ) {
                    Ok(plan) => plan,
                    Err(problem) => return crate::moving::refuse(domain, env, to, problem, out),
                };
                let child = record(domain, disposal.task).expect("named delegate");
                if !old.references.contains(&disposal.task) {
                    references = references.checked_add(1).expect("delegate count bound");
                }
                let left = crate::funders::remaining(child.numbers).expect("move checked remaining");
                let spent = crate::funders::total(child.numbers).expect("move checked spend");
                if child.funder != (Funder::Pool { project, person, period }) {
                    reserved = match reserved.checked_add(left) {
                        Some(value) => value,
                        None => return refused(to, Some(disposal.task), Refusal::Funding, out),
                    };
                }
                if child.funder == Funder::Task(number) {
                    parent_debit = match parent_debit.checked_add(child.numbers.budget) {
                        Some(value) => value,
                        None => return refused(to, Some(number), Refusal::Funding, out),
                    };
                    parent_spent = match parent_spent.checked_add(spent) {
                        Some(value) => value,
                        None => return refused(to, Some(number), Refusal::Funding, out),
                    };
                }
                plans.push(plan).expect("one plan per direct delegate");
            }
        }
    }
    if references > usize::try_from(env.limits.references).expect("reference bound")
        || old.numbers.reserved.checked_sub(parent_debit).is_none()
        || old.numbers.spent_below.checked_add(parent_spent).is_none()
    {
        return refused(to, Some(number), Refusal::Reference, out);
    }
    let pool = Funder::Pool { project, person, period };
    let available = match domain.funding.get(&pool) {
        Some(row) => crate::funders::available(row.numbers),
        None => Some(pool_budget),
    };
    if match available {
        Some(left) => reserved > left,
        None => true,
    } {
        return refused(to, Some(number), Refusal::Funding, out);
    }
    for (task, _) in &domain.names {
        let row = record(domain, *task).expect("live task");
        // One disposal history, a possible proposal reroute, and the finish
        // history for the chat must all remain representable.
        if row.revision.checked_add(2).is_none() {
            return refused(to, Some(*task), Refusal::State, out);
        }
    }
    for plan in plans.into_boxed() {
        crate::moving::make(domain, env, plan, FINISH, out);
    }
    for disposal in disposals.into_boxed() {
        match disposal.keep {
            Keep::MakeMine => {}
            Keep::Cancel => cancel(domain, env, disposal.task, person, CANCEL, out),
        }
    }
    task_mut(domain, number).expect("live chat").record.chat_close = Some(Grace { by: person, until });
    crate::control::amend(
        domain,
        env,
        to,
        Party::Person(person),
        number,
        message,
        false,
        crate::Amendment {
            release: None,
            instructions: None,
            spec: None,
            wake: None,
            dependencies: None,
            authority: None,
            reason: FINISH.into(),
        },
        out,
    );
    crate::moving::reroute(domain, env, out);
}

fn cancel(domain: &mut Domain, env: &Env<Limits>, number: u64, person: u64, reason: &[u8], out: &mut Queue<Request>) {
    let mut selected = List::with_capacity(env.limits.tasks);
    for (task, _) in &domain.names {
        if crate::moving::descendant(domain, *task, number, env.limits.tasks) {
            selected.push(*task).expect("live cohort bound");
        }
    }
    for task in selected.into_boxed() {
        crate::control::history(domain, task, env.wall, Party::Person(person), Change::Cancelled, reason, out);
    }
    crate::closing::cancel_tree(domain, env, number, Party::Person(person), reason, out);
}

pub(crate) fn schedule(domain: &mut Domain, env: &Env<Limits>, number: u64) {
    let task = task_mut(domain, number).expect("live chat schedule");
    let until = match task.record.chat_close {
        Some(grace) if mutable(&task.record.phase) => Some(grace.until),
        Some(_) | None => None,
    };
    let alarm = match until {
        Some(until) => match task.chat_alarm {
            Some(old) if old.until == until => Some(old),
            Some(_) | None => Some(crate::domain::Alarm {
                until,
                due: env.now.saturating_add(skein_lib::Duration::from_nanos(
                    until.as_nanos().saturating_sub(env.wall.as_nanos()),
                )),
            }),
        },
        None => None,
    };
    task.chat_alarm = alarm;
    match alarm {
        Some(alarm) => assert!(domain.chat_alarms.arm(number, alarm.due).is_ok(), "one grace per live task"),
        None => {
            domain.chat_alarms.cancel(number);
        }
    }
}

pub(crate) fn fire(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let Some(number) = domain.chat_alarms.expire(env.now) else { return };
    let Some(row) = record(domain, number) else { return };
    let Some(grace) = row.chat_close else { return };
    if !mutable(&row.phase) {
        return;
    }
    crate::closing::cancel_tree(domain, env, number, Party::Person(grace.by), EXPIRED, out);
}
