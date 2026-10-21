use super::*;
use crate::{AgentShortfall, Bounds, CharterNeeds, ModelBound, ModelNeed, Needs};

fn needs_window(window: u32) -> Needs {
    Needs {
        charters: Box::new([CharterNeeds {
            charter: 1,
            kinds: Kinds::Workers,
            models: Box::new([ModelNeed { name: b"model".as_slice().into(), window, output: 1024 }]),
            turns: 8,
            time: Duration::from_secs(60),
            waiting: Duration::from_secs(10),
            change_reserve: None,
        }]),
        spend: 100,
        message_bytes: 64,
        start_messages: 4,
        unacknowledged: 2,
    }
}
fn bounds_window(window: u32) -> Bounds {
    Bounds {
        models: Box::new([ModelBound { name: b"model".as_slice().into(), window, output: 1024 }]),
        turns: 8,
        spend: 100,
        time: Duration::from_secs(60),
        waiting: Duration::from_secs(10),
        checks: Duration::from_secs(5),
        message_bytes: 64,
        inbox: 4,
        unacknowledged: 2,
    }
}
fn hello(bounds: Bounds) -> Hello {
    Hello { stop_bound: Duration::from_nanos(1), slots: 2, workstreams: Box::new([]), hosting: Box::new([]), bounds }
}
fn rejected(needs: Needs, bounds: Bounds) -> HelloRefusal {
    let expected = crate::shortfall(&needs, &bounds, HostKind::Worker).expect("agent shortfall");
    let mut h = Harness::with_needs(LIMITS, needs);
    assert_eq!(
        &*h.step(Event::Hello { worker: 0, channel: C1, hello: hello(bounds) }),
        &[Request::Refuse { channel: C1 }]
    );
    let actual = refusal(&h);
    assert_eq!(actual, HelloRefusal::Agent { why: expected });
    actual
}
#[test]
fn a_worker_whose_model_window_is_smaller_is_refused_naming_it() {
    assert_eq!(
        rejected(needs_window(8192), bounds_window(4096)),
        HelloRefusal::Agent { why: AgentShortfall::Window { charter: 1, model: 0 } }
    );
}
#[test]
fn a_worker_lacking_a_charters_model_is_admitted() {
    let mut bounds = bounds_window(8192);
    bounds.models = Box::new([]);
    let mut h = Harness::with_needs(LIMITS, needs_window(8192));
    assert!(h.step(Event::Hello { worker: 0, channel: C1, hello: hello(bounds) }).is_empty());
    assert_eq!(h.domain.workers(), 1);
}
#[test]
fn a_worker_whose_inbox_holds_fewer_than_a_start_is_refused() {
    let mut bounds = bounds_window(8192);
    bounds.inbox = 3;
    assert_eq!(rejected(needs_window(8192), bounds), HelloRefusal::Agent { why: AgentShortfall::Inbox });
}
#[test]
fn a_worker_whose_checks_outlast_a_change_reserve_is_refused() {
    let mut needs = needs_window(8192);
    needs.charters[0].change_reserve = Some(Duration::from_secs(4));
    assert_eq!(
        rejected(needs, bounds_window(8192)),
        HelloRefusal::Agent { why: AgentShortfall::Checks { charter: 1 } }
    );
}
#[test]
fn a_charter_for_the_engine_only_needs_nothing_of_workers() {
    let mut needs = needs_window(8192);
    needs.charters[0].kinds = Kinds::Engine;
    let mut h = Harness::with_needs(LIMITS, needs);
    let mut bounds = bounds_window(0);
    bounds.models = Box::new([]);
    assert!(h.step(Event::Hello { worker: 0, channel: C1, hello: hello(bounds) }).is_empty());
    assert_eq!(h.domain.workers(), 1);
}
#[test]
fn a_worker_holding_every_charter_is_admitted() {
    let mut h = Harness::with_needs(LIMITS, needs_window(8192));
    assert!(h.step(Event::Hello { worker: 0, channel: C1, hello: hello(bounds_window(8192)) }).is_empty());
    assert_eq!(h.domain.workers(), 1);
}
#[test]
fn the_engines_first_hello_is_checked_against_its_charters() {
    let mut needs = needs_window(8192);
    needs.charters[0].kinds = Kinds::Engine;
    let mut h = Harness::with_needs(LIMITS, needs);
    assert_eq!(
        &*h.step(Event::Hello { worker: 0, channel: Token::new(0), hello: hello(bounds_window(4096)) }),
        &[Request::Refuse { channel: Token::new(0) }]
    );
    assert_eq!(refusal(&h), HelloRefusal::Agent { why: AgentShortfall::Window { charter: 1, model: 0 } });
}
#[test]
fn a_second_hello_on_the_engine_channel_is_refused() {
    let mut h = Harness::new(LIMITS);
    assert!(h.step(Event::Hello { worker: 0, channel: Token::new(0), hello: hello(bounds_window(8192)) }).is_empty());
    assert_eq!(
        &*h.step(Event::Hello { worker: 0, channel: Token::new(0), hello: hello(bounds_window(8192)) }),
        &[Request::Refuse { channel: Token::new(0) }]
    );
    assert_eq!(refusal(&h), HelloRefusal::ReservedChannel);
}

fn refusal(h: &Harness) -> HelloRefusal {
    h.domain.hello_refusals().get(0).unwrap().why
}

#[test]
#[should_panic(expected = "configured agent needs fit their byte bound")]
fn needs_past_their_byte_bound_fail_construction() {
    core::mem::drop(Domain::new(&Limits { needs_bytes: 0, ..LIMITS }, needs_window(8192)));
}

#[test]
fn a_worker_whose_model_output_is_smaller_is_refused() {
    let mut bounds = bounds_window(8192);
    bounds.models[0].output = 1023;
    assert_eq!(
        rejected(needs_window(8192), bounds),
        HelloRefusal::Agent { why: AgentShortfall::Output { charter: 1, model: 0 } }
    );
}
#[test]
fn a_worker_with_fewer_turns_than_a_charter_is_refused() {
    let mut bounds = bounds_window(8192);
    bounds.turns = 7;
    assert_eq!(rejected(needs_window(8192), bounds), HelloRefusal::Agent { why: AgentShortfall::Turns { charter: 1 } });
}
#[test]
fn a_worker_with_less_spend_than_the_deployment_is_refused() {
    let mut bounds = bounds_window(8192);
    bounds.spend = 99;
    assert_eq!(rejected(needs_window(8192), bounds), HelloRefusal::Agent { why: AgentShortfall::Spend });
}
#[test]
fn a_worker_with_less_time_than_a_charter_is_refused() {
    let mut bounds = bounds_window(8192);
    bounds.time = Duration::from_secs(59);
    assert_eq!(rejected(needs_window(8192), bounds), HelloRefusal::Agent { why: AgentShortfall::Time { charter: 1 } });
}
#[test]
fn a_worker_with_less_waiting_than_a_charter_is_refused() {
    let mut bounds = bounds_window(8192);
    bounds.waiting = Duration::from_secs(9);
    assert_eq!(
        rejected(needs_window(8192), bounds),
        HelloRefusal::Agent { why: AgentShortfall::Waiting { charter: 1 } }
    );
}
#[test]
fn a_worker_with_a_smaller_message_bound_is_refused() {
    let mut bounds = bounds_window(8192);
    bounds.message_bytes = 63;
    assert_eq!(rejected(needs_window(8192), bounds), HelloRefusal::Agent { why: AgentShortfall::Message });
}
#[test]
fn a_worker_keeping_fewer_unacknowledged_turns_is_refused() {
    let mut bounds = bounds_window(8192);
    bounds.unacknowledged = 1;
    assert_eq!(rejected(needs_window(8192), bounds), HelloRefusal::Agent { why: AgentShortfall::Unacknowledged });
}
#[test]
fn a_smaller_alternative_names_its_position() {
    let mut needs = needs_window(8192);
    needs.charters[0].models = Box::new([
        needs.charters[0].models[0].clone(),
        ModelNeed { name: b"alternative".as_slice().into(), window: 8192, output: 1024 },
    ]);
    let mut bounds = bounds_window(8192);
    bounds.models = Box::new([
        bounds.models[0].clone(),
        ModelBound { name: b"alternative".as_slice().into(), window: 4096, output: 1024 },
    ]);
    assert_eq!(rejected(needs, bounds), HelloRefusal::Agent { why: AgentShortfall::Window { charter: 1, model: 1 } });
}

#[test]
fn each_per_charter_shortfall_names_the_second_applicable_charter() {
    let mut needs = needs_window(8192);
    let mut first = needs.charters[0].clone();
    first.charter = 17;
    first.turns = 7;
    first.time = Duration::from_secs(59);
    first.waiting = Duration::from_secs(9);
    first.change_reserve = Some(Duration::from_secs(6));
    let mut second = needs.charters[0].clone();
    second.charter = 73;
    second.change_reserve = Some(Duration::from_secs(5));
    let mut unrelated = second.clone();
    unrelated.charter = 99;
    unrelated.kinds = Kinds::Engine;
    unrelated.turns = 100;
    needs.charters = Box::new([unrelated, first, second]);

    let adequate = bounds_window(8192);
    assert_eq!(crate::shortfall(&needs, &adequate, HostKind::Worker), None);
    let mut h = Harness::with_needs(LIMITS, needs.clone());
    assert!(h.step(Event::Hello { worker: 0, channel: C1, hello: hello(adequate.clone()) }).is_empty());
    assert_eq!(h.domain.workers(), 1);
    for (bounds, expected) in [
        (Bounds { turns: 7, ..adequate.clone() }, AgentShortfall::Turns { charter: 73 }),
        (Bounds { time: Duration::from_secs(59), ..adequate.clone() }, AgentShortfall::Time { charter: 73 }),
        (Bounds { waiting: Duration::from_secs(9), ..adequate.clone() }, AgentShortfall::Waiting { charter: 73 }),
        (Bounds { checks: Duration::from_secs(6), ..adequate.clone() }, AgentShortfall::Checks { charter: 73 }),
    ] {
        assert_eq!(rejected(needs.clone(), bounds), HelloRefusal::Agent { why: expected });
    }
    // The canonical decision remains first-shortfall only when several bounds fail.
    let several = Bounds {
        turns: 7,
        time: Duration::from_secs(59),
        waiting: Duration::from_secs(9),
        checks: Duration::from_secs(6),
        message_bytes: 63,
        ..adequate
    };
    assert_eq!(rejected(needs, several), HelloRefusal::Agent { why: AgentShortfall::Turns { charter: 73 } });
}
