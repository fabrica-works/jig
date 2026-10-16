//! Durable finish graces for every task (domain/tasks.md, sections 7 and 8.2).
use crate::domain::{Domain, publish, record, task_mut};
use crate::{Change, Limits, MessageKind, Party, Phase, Refusal, Request, Word};
use alloc::boxed::Box;
use skein_lib::{Env, Queue, Wall};

/// Who asked a task to finish, when its grace ends, and the cancellation reason.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Grace {
    pub by: Party,
    pub until: Wall,
    pub reason: Box<[u8]>,
    /// The single finish word, kept apart from ordinary merged amendments.
    pub message: u64,
}

#[expect(clippy::too_many_arguments, reason = "one durable finish request with its maker, deadline and message")]
pub(crate) fn ask_to_finish(
    domain: &mut Domain,
    env: &Env<Limits>,
    number: u64,
    by: Party,
    until: Wall,
    message: u64,
    reason: &[u8],
    out: &mut Queue<Request>,
) -> Result<(), Refusal> {
    ask(domain, env, number, by, until, message, reason, reason, out)
}

#[expect(clippy::too_many_arguments, reason = "the chat keeps its existing distinct finish and cancellation texts")]
pub(crate) fn ask_chat(
    domain: &mut Domain,
    env: &Env<Limits>,
    number: u64,
    person: u64,
    until: Wall,
    message: u64,
    words: &[u8],
    reason: &[u8],
    out: &mut Queue<Request>,
) -> Result<(), Refusal> {
    ask(domain, env, number, Party::Person(person), until, message, words, reason, out)
}

#[expect(clippy::too_many_arguments, reason = "one durable finish request includes distinct chat texts")]
fn ask(
    domain: &mut Domain,
    env: &Env<Limits>,
    number: u64,
    by: Party,
    until: Wall,
    message: u64,
    words: &[u8],
    reason: &[u8],
    out: &mut Queue<Request>,
) -> Result<(), Refusal> {
    let old = record(domain, number).ok_or(Refusal::Unknown)?;
    if message == 0
        || message <= old.last_message
        || words.len() > usize::try_from(env.limits.result_bytes).expect("reason bound")
    {
        return Err(Refusal::Read);
    }
    match old.phase {
        Phase::Waiting | Phase::Held { .. } => {
            cancel(domain, env, number, by, reason, out);
            return Ok(());
        }
        Phase::Active(_) => {}
        Phase::Closing(_) | Phase::Ended(_) => return Err(Refusal::State),
    }
    if let Some(grace) = &old.grace {
        if until < grace.until {
            let grace = task_mut(domain, number).expect("live grace").record.grace.as_mut().expect("saved grace");
            grace.by = by;
            grace.until = until;
            grace.reason = reason.into();
            publish(domain, env, number, out);
        }
        return Ok(());
    }
    let revision = old.revision.checked_add(1).ok_or(Refusal::State)?;
    let word = Word {
        number: message,
        from: by,
        kind: MessageKind::Amendment { revision },
        words: words.into(),
        at: env.wall,
        hits: 1,
        eligible: true,
    };
    let prepared = crate::inbox::prepare(domain, &env.limits, number, word, crate::inbox::Admission::RoomKept)
        .expect("grace has its reserved inbox slot");
    let task = task_mut(domain, number).expect("live grace task");
    task.record.grace = Some(Grace { by, until, reason: reason.into(), message });
    task.record.last_activity = env.wall;
    task.record.last_message = message;
    task.record.inbox = prepared.inbox;
    crate::activity::message(&mut task.record, &prepared.activity_word, env.limits.inbox_messages, out);
    crate::control::history(domain, number, env.wall, by, Change::Amended, reason, out);
    publish(domain, env, number, out);
    crate::wake::after_message(domain, env, number, prepared.previous, prepared.word, out);
    Ok(())
}

pub(crate) fn cancel(
    domain: &mut Domain,
    env: &Env<Limits>,
    number: u64,
    by: Party,
    reason: &[u8],
    out: &mut Queue<Request>,
) {
    let mut selected = skein_lib::List::with_capacity(env.limits.tasks);
    for (child, _) in &domain.names {
        if crate::moving::descendant(domain, *child, number, env.limits.tasks) {
            selected.push(*child).expect("live subtree bound");
        }
    }
    for child in selected.into_boxed() {
        crate::control::history(domain, child, env.wall, by, Change::Cancelled, reason, out);
    }
    crate::closing::cancel_tree(domain, env, number, by, reason, out);
}

pub(crate) fn schedule(domain: &mut Domain, env: &Env<Limits>, number: u64) {
    let task = task_mut(domain, number).expect("live chat schedule");
    let until = match &task.record.grace {
        Some(grace) if crate::chat::mutable(&task.record.phase) => Some(grace.until),
        Some(_) | None => None,
    };
    let alarm = match until {
        Some(until) => match task.grace_alarm {
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
    task.grace_alarm = alarm;
    match alarm {
        Some(alarm) => assert!(domain.grace_alarms.arm(number, alarm.due).is_ok(), "one grace per live task"),
        None => {
            domain.grace_alarms.cancel(number);
        }
    }
}

pub(crate) fn fire(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let Some(number) = domain.grace_alarms.expire(env.now) else { return };
    let Some(row) = record(domain, number) else { return };
    let Some(grace) = row.grace.clone() else { return };
    if !crate::chat::mutable(&row.phase) {
        return;
    }
    crate::closing::cancel_tree(domain, env, number, grace.by, &grace.reason, out);
}
