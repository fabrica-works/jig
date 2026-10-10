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
            follow: None,
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

fn follow(domain: &mut Domain, watcher: u64, run: u64, turn: Option<TurnName>, offset: u32) -> Box<[Request]> {
    drive(
        domain,
        Event::Watch {
            watcher: Token::new(watcher),
            party: Token::new(watcher),
            subject: Token::new(7),
            window: Window { rows: 10, after: None },
            follow: Some(Follow { run: Token::new(run), turn, offset }),
        },
    )
}
fn text(domain: &mut Domain, turn: u32, offset: u32, text: &[u8]) -> Box<[Request]> {
    drive(
        domain,
        Event::Text {
            run: Token::new(7),
            turn: TurnName { attempt: 1, turn, retry: 0 },
            offset,
            text: Box::from(text),
            call: None,
        },
    )
}
fn stream(watcher: u64, turn: u32, from: u32, to: u32, text: &[u8], cut: bool) -> Request {
    Request::Stream {
        watcher: Token::new(watcher),
        turn: TurnName { attempt: 1, turn, retry: 0 },
        from,
        to,
        text: Box::from(text),
        cut,
        call: None,
    }
}
#[test]
fn each_watcher_reads_a_turn_from_its_own_offset() {
    let mut domain = Domain::new(&LIMITS);
    let turn = TurnName { attempt: 1, turn: 1, retry: 0 };
    follow(&mut domain, 1, 7, Some(turn), 0);
    follow(&mut domain, 2, 7, Some(turn), 3);
    assert_eq!(
        text(&mut domain, 1, 0, b"abcdef").as_ref(),
        &[stream(1, 1, 0, 6, b"abcdef", false), stream(2, 1, 3, 6, b"def", false)]
    );
    drive(&mut domain, Event::StreamDelivered { watcher: Token::new(2) });
    assert_eq!(text(&mut domain, 1, 6, b"ghi").as_ref(), &[stream(2, 1, 6, 9, b"ghi", false)]);
    assert_eq!(
        drive(&mut domain, Event::StreamDelivered { watcher: Token::new(1) }).as_ref(),
        &[stream(1, 1, 6, 9, b"ghi", false)]
    );
}
#[test]
fn a_new_turn_starts_at_zero_without_changing_an_earlier_flight() {
    let mut domain = Domain::new(&LIMITS);
    follow(&mut domain, 1, 7, None, 0);
    assert_eq!(text(&mut domain, 1, 0, b"earlier").as_ref(), &[stream(1, 1, 0, 7, b"earlier", false)]);
    assert!(text(&mut domain, 2, 0, b"new").is_empty());
    assert_eq!(
        drive(&mut domain, Event::StreamDelivered { watcher: Token::new(1) }).as_ref(),
        &[stream(1, 2, 0, 3, b"new", false)]
    );
    assert!(text(&mut domain, 1, 7, b"stale").is_empty());
    drive(&mut domain, Event::StreamDelivered { watcher: Token::new(1) });
    assert_eq!(text(&mut domain, 2, 3, b"er").as_ref(), &[stream(1, 2, 3, 5, b"er", false)]);
}
#[test]
fn a_slow_stream_gets_a_bounded_suffix_with_its_cut_stated() {
    let mut domain = Domain::new(&LIMITS);
    follow(&mut domain, 1, 7, None, 0);
    text(&mut domain, 1, 0, b"abc");
    assert!(text(&mut domain, 1, 3, b"defghijklmnop").is_empty());
    assert!(text(&mut domain, 1, 16, b"qrstuvwxyz").is_empty());
    assert_eq!(
        drive(&mut domain, Event::StreamDelivered { watcher: Token::new(1) }).as_ref(),
        &[stream(1, 1, 10, 26, b"klmnopqrstuvwxyz", true)]
    );
}
#[test]
fn closing_waits_for_both_document_and_stream_terminals() {
    let mut domain = Domain::new(&LIMITS);
    follow(&mut domain, 1, 7, None, 0);
    ready(&mut domain, 1);
    text(&mut domain, 1, 0, b"abc");
    drive(&mut domain, Event::Unwatch { watcher: Token::new(1) });
    assert!(drive(&mut domain, Event::Delivered { watcher: Token::new(1) }).is_empty());
    assert_eq!(
        drive(&mut domain, Event::StreamDelivered { watcher: Token::new(1) }).as_ref(),
        &[Request::Ended { watcher: Token::new(1), end: End::Unwatched }]
    );
    assert_eq!(domain.runs(), 0);
}
#[test]
fn followed_runs_past_the_limit_are_refused_busy() {
    let mut domain = Domain::new(&LIMITS);
    follow(&mut domain, 1, 7, None, 0);
    follow(&mut domain, 2, 8, None, 0);
    assert_eq!(
        follow(&mut domain, 3, 9, None, 0).as_ref(),
        &[Request::Refused { watcher: Token::new(3), refusal: Refusal::Busy }]
    );
    assert_eq!(follow(&mut domain, 4, 7, None, 0).len(), 2);
}
#[test]
fn ending_a_run_discards_text_and_a_new_attempt_can_write() {
    let mut domain = Domain::new(&LIMITS);
    follow(&mut domain, 1, 7, None, 0);
    text(&mut domain, 1, 0, b"old");
    drive(&mut domain, Event::RunEnded { run: Token::new(7) });
    assert!(drive(&mut domain, Event::StreamDelivered { watcher: Token::new(1) }).is_empty());
    assert!(text(&mut domain, 1, 0, b"old").is_empty());
    let out = drive(
        &mut domain,
        Event::Text {
            run: Token::new(7),
            turn: TurnName { attempt: 2, turn: 1, retry: 0 },
            offset: 0,
            text: Box::from(&b"new"[..]),
            call: None,
        },
    );
    assert_eq!(
        out.as_ref(),
        &[Request::Stream {
            watcher: Token::new(1),
            turn: TurnName { attempt: 2, turn: 1, retry: 0 },
            from: 0,
            to: 3,
            text: Box::from(&b"new"[..]),
            cut: false,
            call: None
        }]
    );
}
#[test]
fn current_calls_are_bounded_and_paced_with_the_stream() {
    let mut domain = Domain::new(&LIMITS);
    follow(&mut domain, 1, 7, None, 0);
    let call = Call { number: 4, tool: Box::from(&b"tool"[..]), input: Box::from(&b"abcdefghijkl"[..]), cut: true };
    let out = drive(
        &mut domain,
        Event::Text {
            run: Token::new(7),
            turn: TurnName { attempt: 1, turn: 1, retry: 0 },
            offset: 0,
            text: Box::new([]),
            call: Some(call),
        },
    );
    assert_eq!(
        out.as_ref(),
        &[Request::Stream {
            watcher: Token::new(1),
            turn: TurnName { attempt: 1, turn: 1, retry: 0 },
            from: 0,
            to: 0,
            text: Box::new([]),
            cut: false,
            call: Some(Call {
                number: 4,
                tool: Box::from(&b"tool"[..]),
                input: Box::from(&b"abcdefghijkl"[..]),
                cut: true
            })
        }]
    );
}
