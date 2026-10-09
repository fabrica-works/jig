//! Authenticated whole chat closes (domain/people.md, section 7).
//! Core owns standing, authority and the configured grace; tasks owns the
//! all-or-nothing disposals, funding transfer, finish amendment and deadline.
use crate::{Core, Event, Limits, PersonTaskRoute};
use jig_core_people as people;
use jig_core_tasks as tasks;
use skein_lib::{Env, List, Queue, ReplyTo, Token, Wall};

fn refuse(work: &mut Queue<Event>, request: Token, why: people::Refusal) {
    work.push(Event::People(people::Event::Decided { request, outcome: people::Outcome::Refused(why) }));
}

#[expect(clippy::too_many_arguments, reason = "one authenticated close carries its keyed party and whole disposals")]
#[expect(clippy::too_many_lines, reason = "all standing and disposal checks precede the one child event")]
pub(crate) fn route(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    request: Token,
    person: u64,
    project: u32,
    role: Option<people::Role>,
    chat: u64,
    disposals: List<people::Disposal>,
) {
    let Some(row) = core.tasks.task(chat) else { return refuse(work, request, people::Refusal::Unknown) };
    if row.project != project || role.is_none() || core.chat_requester(row) != Some(person) {
        return refuse(work, request, people::Refusal::Unknown);
    }
    if row.chat_close.is_some() {
        return refuse(work, request, people::Refusal::NotWaiting);
    }
    let mutable = match &row.phase {
        tasks::Phase::Waiting
        | tasks::Phase::Active(_)
        | tasks::Phase::Held { was: tasks::Was::Waiting | tasks::Was::Active(_), .. } => true,
        tasks::Phase::Closing(_) | tasks::Phase::Held { was: tasks::Was::Closing(_), .. } | tasks::Phase::Ended(_) => {
            false
        }
    };
    if !mutable {
        return refuse(work, request, people::Refusal::NotWaiting);
    }
    for (at, disposal) in disposals.iter().enumerate() {
        if !row.delegates.contains(&disposal.task) {
            return refuse(
                work,
                request,
                people::Refusal::Disposal { task: disposal.task, issue: people::DisposalIssue::Unknown },
            );
        }
        for earlier in disposals.iter().take(at) {
            if earlier.task == disposal.task {
                return refuse(
                    work,
                    request,
                    people::Refusal::Disposal { task: disposal.task, issue: people::DisposalIssue::Repeated },
                );
            }
        }
    }
    for delegate in &row.delegates {
        let mut named = false;
        for disposal in &disposals {
            if disposal.task == *delegate {
                named = true;
            }
        }
        if !named {
            return refuse(
                work,
                request,
                people::Refusal::Disposal { task: *delegate, issue: people::DisposalIssue::Missing },
            );
        }
    }
    let mut children = List::with_capacity(env.limits.tasks.delegates);
    let mut cancelled = 0_u32;
    let mut kept = 0_u32;
    for disposal in disposals.into_boxed() {
        let keep = match disposal.keep {
            people::Keep::Cancel => {
                cancelled = cancelled.checked_add(1).expect("bounded disposals");
                tasks::Keep::Cancel
            }
            people::Keep::MakeMine => {
                if let Err(why) = core.move_admit(
                    person,
                    role,
                    project,
                    disposal.task,
                    env.limits.tasks.tree_tasks,
                    env.limits.tasks.depth,
                ) {
                    return refuse(work, request, why);
                }
                kept = kept.checked_add(1).expect("bounded disposals");
                tasks::Keep::MakeMine
            }
        };
        if children.push(tasks::Disposal { task: disposal.task, keep }).is_err() {
            return refuse(work, request, people::Refusal::Limit { which: people::LimitKind::LiveDelegates });
        }
    }
    if core.settings.close_grace == skein_lib::Duration::ZERO {
        return refuse(work, request, people::Refusal::NotOffered);
    }
    let Some(until) = env.wall.as_nanos().checked_add(core.settings.close_grace.as_nanos()) else {
        return refuse(work, request, people::Refusal::Limit { which: people::LimitKind::Arithmetic });
    };
    let Some(message) = crate::fresh(&mut core.counters, crate::Family::Message) else {
        return refuse(work, request, people::Refusal::Limit { which: people::LimitKind::Arithmetic });
    };
    assert!(
        core.person_tasks.insert(request, PersonTaskRoute::Close { chat, cancelled, kept }) == Ok(None),
        "one keyed chat close"
    );
    work.push(Event::Tasks(tasks::Event::CloseChat {
        reply_to: ReplyTo::new(request),
        task: chat,
        person,
        message,
        until: Wall::from_nanos(until),
        disposals: children,
        period: core.settings.period,
        pool_budget: core.settings.person_budget,
        period_budget: core.settings.period_budget,
    }));
}
