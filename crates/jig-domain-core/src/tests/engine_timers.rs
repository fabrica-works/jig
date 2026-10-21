//! The public timer route preserves the ordinary decision journal boundary.
#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::wildcard_enum_match_arm,
    clippy::match_like_matches_macro,
    reason = "entry-point fixtures are ordinary Rust under programming-model.md section 10.2"
)]
use super::*;
use skein_lib::{Journal, JournalLimits, JournalRoom, Released};

pub(super) fn timer_core(answers: bool) -> (Core, Env<Limits>) {
    let mut limits = limits();
    limits.people.sign_ins = 3;
    let mut core = Core::new(config(41), &limits).expect("fixture core configuration");
    let mut deployment = core.counters.deployment();
    deployment.people = 1;
    deployment.sign_ins = 11;
    core.counters = Counters::new(deployment);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let identity = people::Identity {
        key: people::IdentityKey { provider: 0, subject: Box::new([1]) },
        login: Box::from(&b"owner"[..]),
        name: Box::from(&b"A Person"[..]),
    };
    let mut rows = vec![people::Stored::Person { number: 1, identity }];
    for number in [10, 11] {
        rows.push(people::Stored::SignIn {
            number,
            person: 1,
            expires: Wall::from_nanos(limits.people.sign_in_lifetime.as_nanos()),
        });
    }
    if answers {
        for id in [1, 2] {
            rows.push(people::Stored::Answer {
                models: Box::new([]),
                key: people::RequestKey { person: 1, key: [id; 16] },
                ask: Box::new(people::Ask::Stop { project: 1, task: 1 }),
                outcome: people::Outcome::Stopped { task: 1 },
                at: Wall::EPOCH,
            });
        }
    }
    for record in rows {
        drop(step(&mut core, &env, Event::People(people::Event::Restore { record })));
    }
    drop(step(&mut core, &env, Event::People(people::Event::Restored)));
    assert!(core.people.ready());
    (core, env)
}

fn accept(journal: &mut Journal<Write, Box<Held>>, requests: Requests) -> u32 {
    let Requests::Out(mut out) = requests;
    let mut decision = journal.decision(&JournalRoom { writes: 1, held: 1 }).unwrap();
    let mut writes = 0_u32;
    while let Some(request) = out.pop() {
        match request {
            Request::Write(write) => {
                decision.write(write).unwrap();
                writes = writes.checked_add(1).unwrap();
            }
            Request::Held(held) => decision.hold(held).unwrap(),
            Request::Decided => {}
            Request::Ask { .. } | Request::Now(_) => panic!("timer/reply must not escape the journal"),
        }
    }
    journal.accept(decision);
    writes
}

fn journal() -> Journal<Write, Box<Held>> {
    Journal::new(&JournalLimits { commits: 4, writes: 4, held: 4, now: 1, release: 4 })
}

#[test]
fn people_timer_expires_signins_durably_and_later_replies_wait_for_that_commit() {
    let (mut core, mut env) = timer_core(false);
    let due = Time::ZERO.saturating_add(env.limits.people.sign_in_lifetime);
    assert_eq!(core.people.next_deadline(), Some(due));
    let Requests::Out(mut future) = fire(&mut core, &env, Timer::People);
    let Some(Request::Decided) = future.pop() else { panic!("future timer decides without writes") };
    assert!(future.is_empty());
    assert_eq!(core.people.next_deadline(), Some(due));
    env.now = due;
    env.wall = Wall::from_nanos(due.as_nanos());
    let mut journal = journal();
    assert_eq!(accept(&mut journal, fire(&mut core, &env, Timer::People)), 1);
    let mut commit = journal.commit().unwrap();
    assert_eq!(commit.writes.len(), 1);
    let Some(Write::Erase(Key::People(people::Key::SignIn(10)))) = commit.writes.pop() else {
        panic!("expired signin erase")
    };
    assert!(core.people.is_due(env.now), "the second expiration remains for a later bounded fire");
    let reply = step(
        &mut core,
        &env,
        Event::People(people::Event::Ask {
            models: Box::new([]),
            reply_to: ReplyTo::new(Token::new(50)),
            sign_in: 10,
            key: [5; 16],
            ask: people::Ask::Stop { project: 1, task: 1 },
        }),
    );
    assert_eq!(accept(&mut journal, reply), 0);
    assert!(journal.commit().is_none());
    let mut released = Queue::with_capacity(4);
    assert_eq!(journal.release(&mut released), Released::None);
    journal.committed(commit.number);
    assert_eq!(journal.release(&mut released), Released::Some);
    assert_eq!(released.len(), 1);
    let held = released.pop().unwrap();
    let Held::PeopleReply { reply: people::Reply::SignedOut, .. } = held.as_ref() else {
        panic!("expired session reply")
    };
}

#[test]
fn people_timer_erases_retained_answers_then_signins_one_decision_at_a_time() {
    let (mut core, mut env) = timer_core(true);
    env.now = Time::ZERO.saturating_add(env.limits.people.request_retention);
    env.wall = Wall::from_nanos(env.now.as_nanos());
    let expected = [
        people::Key::Answer(people::RequestKey { person: 1, key: [1; 16] }),
        people::Key::Answer(people::RequestKey { person: 1, key: [2; 16] }),
        people::Key::SignIn(10),
        people::Key::SignIn(11),
    ];
    let mut journal = journal();
    for key in expected {
        assert!(core.people.is_due(env.now));
        let Requests::Out(out) = fire(&mut core, &env, Timer::People);
        assert_eq!(out.len(), 2, "one bounded erase and its decided mark");
        assert_eq!(accept(&mut journal, Requests::Out(out)), 1);
        let mut commit = journal.commit().unwrap();
        assert_eq!(commit.writes.len(), 1);
        let Some(Write::Erase(Key::People(erased))) = commit.writes.pop() else { panic!("typed timer erase") };
        assert_eq!(erased, key);
        let mut released = Queue::with_capacity(4);
        assert_eq!(journal.release(&mut released), Released::None);
        journal.committed(commit.number);
        assert_eq!(journal.release(&mut released), Released::None);
        assert!(released.is_empty());
    }
    assert_eq!(core.people.next_deadline(), None);
    for timer in [Timer::People, Timer::Tasks, Timer::Fleet, Timer::Brief, Timer::Account, Timer::View] {
        let Requests::Out(mut out) = fire(&mut core, &env, timer);
        let Some(Request::Decided) = out.pop() else { panic!("empty timer decides without writes") };
        assert!(out.is_empty(), "empty owners remain separately dispatchable: {timer:?}");
    }
}
