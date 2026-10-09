use crate::*;
use alloc::boxed::Box;
use skein_lib::{Env, List, Queue, Time, Token, Wall};
const LIMITS: Limits =
    Limits { runs: 2, watchers: 4, watchers_per_party: 2, subjects_per_change: 4, text_bytes: 16, facts: 32 };
fn drive(domain: &mut Domain, event: Event) -> Box<[Request]> {
    let mut out = Queue::with_capacity(max_out(&LIMITS));
    step(domain, &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: LIMITS }, event, &mut out);
    let mut requests = List::with_capacity(max_out(&LIMITS));
    for _ in 0..out.len() {
        requests.push(out.pop().expect("output count")).expect("bounded output");
    }
    requests.into_boxed()
}
fn watch(domain: &mut Domain, watcher: u64, party: u64) -> Box<[Request]> {
    drive(
        domain,
        Event::Watch {
            watcher: Token::new(watcher),
            party: Token::new(party),
            subject: Token::new(7),
            window: Window { rows: 10, after: None },
        },
    )
}
fn ready(domain: &mut Domain, watcher: u64) {
    assert_eq!(
        drive(domain, Event::Made { watcher: Token::new(watcher), outcome: Made::Ready }),
        Box::<[Request]>::from([Request::Deliver { watcher: Token::new(watcher) }])
    );
}
fn change(domain: &mut Domain) -> Box<[Request]> {
    let mut subjects = List::with_capacity(1);
    subjects.push(Token::new(7)).expect("one subject");
    drive(domain, Event::Changed { subjects })
}
#[test]
fn ten_changes_in_flight_make_one_next_delivery() {
    let mut domain = Domain::new(&LIMITS);
    assert_eq!(watch(&mut domain, 1, 1).len(), 2);
    ready(&mut domain, 1);
    for _ in 0_u32..10 {
        assert!(change(&mut domain).is_empty());
    }
    assert_eq!(drive(&mut domain, Event::Delivered { watcher: Token::new(1) }).len(), 1);
    ready(&mut domain, 1);
    assert!(drive(&mut domain, Event::Delivered { watcher: Token::new(1) }).is_empty());
}
#[test]
fn a_change_while_idle_makes_a_document_at_once() {
    let mut domain = Domain::new(&LIMITS);
    watch(&mut domain, 1, 1);
    ready(&mut domain, 1);
    drive(&mut domain, Event::Delivered { watcher: Token::new(1) });
    assert_eq!(change(&mut domain).len(), 1);
}
#[test]
fn unwatch_waits_for_a_delivery_to_end() {
    let mut domain = Domain::new(&LIMITS);
    watch(&mut domain, 1, 1);
    ready(&mut domain, 1);
    assert!(drive(&mut domain, Event::Unwatch { watcher: Token::new(1) }).is_empty());
    assert!(change(&mut domain).is_empty());
    assert_eq!(
        drive(&mut domain, Event::Delivered { watcher: Token::new(1) }),
        Box::<[Request]>::from([Request::Ended { watcher: Token::new(1), end: End::Unwatched }])
    );
}
#[test]
fn a_gone_subject_waits_and_preserves_its_reason() {
    let mut domain = Domain::new(&LIMITS);
    watch(&mut domain, 1, 1);
    ready(&mut domain, 1);
    assert!(drive(&mut domain, Event::Gone { subject: Token::new(7), why: End::NotAllowed }).is_empty());
    assert_eq!(
        drive(&mut domain, Event::Delivered { watcher: Token::new(1) }),
        Box::<[Request]>::from([Request::Ended { watcher: Token::new(1), end: End::NotAllowed }])
    );
}
#[test]
fn closing_during_construction_waits_for_the_parent() {
    let mut domain = Domain::new(&LIMITS);
    watch(&mut domain, 1, 1);
    assert!(drive(&mut domain, Event::Unwatch { watcher: Token::new(1) }).is_empty());
    assert_eq!(
        drive(&mut domain, Event::Made { watcher: Token::new(1), outcome: Made::Ready }),
        Box::<[Request]>::from([Request::Ended { watcher: Token::new(1), end: End::Unwatched }])
    );
}
#[test]
fn changes_during_construction_are_not_lost() {
    let mut domain = Domain::new(&LIMITS);
    watch(&mut domain, 1, 1);
    change(&mut domain);
    ready(&mut domain, 1);
    assert_eq!(drive(&mut domain, Event::Delivered { watcher: Token::new(1) }).len(), 1);
}
#[test]
fn watches_past_global_and_party_bounds_are_refused_busy() {
    let mut domain = Domain::new(&LIMITS);
    watch(&mut domain, 1, 1);
    watch(&mut domain, 2, 1);
    assert_eq!(
        watch(&mut domain, 3, 1),
        Box::<[Request]>::from([Request::Refused { watcher: Token::new(3), refusal: Refusal::Busy }])
    );
    watch(&mut domain, 3, 2);
    watch(&mut domain, 4, 2);
    assert_eq!(
        watch(&mut domain, 5, 3),
        Box::<[Request]>::from([Request::Refused { watcher: Token::new(5), refusal: Refusal::Busy }])
    );
}
#[test]
fn a_missing_document_ends_without_a_delivery() {
    let mut domain = Domain::new(&LIMITS);
    watch(&mut domain, 1, 1);
    assert_eq!(
        drive(&mut domain, Event::Made { watcher: Token::new(1), outcome: Made::Gone { why: End::Gone } }),
        Box::<[Request]>::from([Request::Ended { watcher: Token::new(1), end: End::Gone }])
    );
}
