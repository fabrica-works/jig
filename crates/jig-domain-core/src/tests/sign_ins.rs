//! A sign-in is named only by its own reply (`domain/people.md`, section 3).
use super::engine_timers::timer_core;
use super::*;

fn newcomer() -> people::Identity {
    people::Identity {
        key: people::IdentityKey { provider: 0, subject: Box::new([2]) },
        login: Box::from(&b"newcomer"[..]),
        name: Box::from(&b"A Newcomer"[..]),
    }
}

#[test]
fn a_sign_in_reply_names_the_sign_in_its_decision_saved() {
    let (mut core, env) = timer_core(false);
    let Requests::Out(mut out) =
        step(&mut core, &env, Event::SignIn { reply_to: ReplyTo::new(Token::new(60)), identity: newcomer() });
    let mut saved = None;
    let mut named = None;
    for _ in 0..out.len() {
        let request = out.pop().expect("output count");
        if let Request::Write(Write::Save(Record::People(people::Stored::SignIn { number, .. }))) = &request {
            saved = Some(*number);
        }
        if let Request::Held(held) = request
            && let Held::PeopleReply { reply: people::Reply::SignedIn { sign_in, .. }, .. } = *held
        {
            named = Some(sign_in);
        }
    }
    assert!(saved.is_some(), "the decision saved one sign-in");
    assert_eq!(named, saved);
}

#[test]
fn the_core_is_quiescent_once_a_sign_in_reply_has_left() {
    let (mut core, env) = timer_core(false);
    drop(step(&mut core, &env, Event::SignIn { reply_to: ReplyTo::new(Token::new(61)), identity: newcomer() }));
    let _committed = core.counters.next_commit();
    assert!(core.quiescent(true));
}

#[test]
fn a_replayed_live_sign_in_names_the_same_sign_in() {
    let (mut core, env) = timer_core(false);
    let identity = people::Identity {
        key: people::IdentityKey { provider: 0, subject: Box::new([1]) },
        login: Box::from(&b"owner"[..]),
        name: Box::from(&b"A Person"[..]),
    };
    let Requests::Out(mut out) = step(
        &mut core,
        &env,
        Event::People(people::Event::SignedIn {
            reply_to: ReplyTo::new(Token::new(62)),
            person: 1,
            sign_in: 10,
            identity,
            kind: people::Kind::Person,
        }),
    );
    let mut replayed = None;
    for _ in 0..out.len() {
        if let Request::Held(held) = out.pop().expect("output count")
            && let Held::PeopleReply { reply: people::Reply::SignedIn { sign_in, .. }, .. } = *held
        {
            replayed = Some(sign_in);
        }
    }
    assert_eq!(replayed, Some(10));
}
