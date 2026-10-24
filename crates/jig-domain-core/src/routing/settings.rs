//! Owner settings decisions, atomic with their keyed answer (engine.md, 17.2).
use crate::{Core, Event, Limits, Request, Requests, settings as s};
use alloc::boxed::Box;
use jig_domain_people as people;
use skein_lib::{Env, Queue, ReplyTo, Token};

#[expect(clippy::too_many_arguments, reason = "the keyed envelope and its value cross admission together")]
pub(super) fn start(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    reply_to: ReplyTo,
    sign_in: u64,
    key: [u8; 16],
    digest: [u8; 32],
    change: Box<s::Change>,
) -> Requests {
    assert!(core.settings_input.is_none(), "one settings admission at a time");
    let against = change.against;
    let room = room(&env.limits)
        .expect("validated settings room")
        .writes
        .saturating_sub(1)
        .saturating_sub(people::max_out(&env.limits.people));
    core.settings_input = Some((
        match s::change_shape(&change.items, &env.limits) {
            Ok(()) => Ok(change),
            Err(failure) => Err(failure),
        },
        room,
    ));
    let result = super::people_routes::route_people(
        core,
        env,
        work,
        people::Event::Ask {
            reply_to,
            sign_in,
            key,
            ask: people::Ask::ChangeSettings { against, digest },
            models: Box::new([]),
        },
    );
    drop(core.settings_input.take());
    result
}
pub(super) fn admitted(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    out: &mut Queue<Request>,
    request: Token,
    person: u64,
    against: u32,
) {
    let outcome = match core.settings_input.take() {
        Some(_) if against != core.current.newest() => {
            people::Outcome::Refused(people::Refusal::Stale { newest: core.current.newest() })
        }
        Some((Ok(change), room)) => decide(core, env, work, out, person, *change, room),
        Some((Err(failure), _)) => malformed(failure),
        None => people::Outcome::Refused(people::Refusal::NotOffered),
    };
    work.push(Event::People(people::Event::Decided { request, outcome }));
}
fn malformed(failure: s::Failure) -> people::Outcome {
    people::Outcome::Refused(people::Refusal::NotWellFormed {
        at: 0,
        why: people::Malformed::Settings { setting: failure.setting, fault: failure.fault },
    })
}
fn decide(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    out: &mut Queue<Request>,
    person: u64,
    change: s::Change,
    room: u32,
) -> people::Outcome {
    let previous = core.current.newest();
    if change.against != previous {
        return people::Outcome::Refused(people::Refusal::Stale { newest: previous });
    }
    if !s::changes_current(&core.current, &change.items) {
        return people::Outcome::Refused(people::Refusal::NotWellFormed {
            at: 0,
            why: people::Malformed::SettingsUnchanged,
        });
    }
    if let Err(failure) = s::check_change(&core.current, &change.items, &core.settings, &core.connectors, &env.limits) {
        return malformed(failure);
    }
    let charters = s::check::changed_charters(&core.current, &change.items, &core.settings, &env.limits);
    let (tasks, more) = core.tasks.incompatible_charters(&charters, env.limits.tasks.circle);
    if !tasks.is_empty() || more != 0 {
        return people::Outcome::Refused(people::Refusal::Full { revision: previous, tasks, more });
    }
    let Some(number) = previous.checked_add(1) else {
        return people::Outcome::Refused(people::Refusal::Limit { which: people::LimitKind::Arithmetic });
    };
    let mut templates = false;
    let mut rules = false;
    let mut charters = false;
    for item in &change.items {
        if s::changes_current(&core.current, core::slice::from_ref(item)) {
            match item.setting {
                s::Setting::Charter(_) => charters = true,
                s::Setting::Template(_) | s::Setting::Policy(_) => templates = true,
                s::Setting::Rules => rules = true,
            }
        }
    }
    let writes = s::apply(&mut core.current, &change.items, number);
    core.current.head = s::Head {
        number,
        against: previous,
        at: env.wall,
        by: s::Decider::Party(person),
        proposal: None,
        moved: change.running == s::Running::Moves,
    };
    let revision = s::Revision { head: core.current.head.clone(), items: change.items };
    let (writes, oldest) = s::revision_writes(
        Some(revision),
        writes,
        core.current.oldest,
        core.current.newest(),
        env.limits.settings.history,
        room,
    );
    core.current.oldest = oldest;
    for write in writes.into_boxed() {
        out.push(Request::Write(write));
    }
    core.tell_replaced_settings(env, rules, charters);
    if charters || rules {
        core.touch_deployment();
        core.touch_run_waits();
    }
    super::watch_marks::settings_revised(core, work, templates);
    people::Outcome::SettingsChanged { revision: number }
}

pub(super) fn room(limits: &Limits) -> Option<skein_lib::JournalRoom> {
    let people = people::max_out(&limits.people);
    Some(skein_lib::JournalRoom {
        writes: s::writes_worst_case(limits)?.checked_add(people)?,
        held: people.checked_add(limits.views.watchers.checked_mul(4)?)?,
    })
}
