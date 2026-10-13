use jig_domain_host as host;
use jig_protocol_worker_link::{Event, Request};
use jig_worker_link_world::{World, fixtures};
use skein_lib::{Duration, Reader};
fn world() -> World {
    open_world(&fixtures::limits())
}
fn open_world(limits: &jig_protocol_worker_link::Limits) -> World {
    let mut world = World::new(limits, 1);
    world.engine(fixtures::engine_accepting());
    world.run();
    world.down(Event::Hello { hello: fixtures::hello() });
    world
}
fn call(world: &mut World, name: &[u8]) {
    world.down(Event::Up { up: fixtures::call(name), parts: Box::new([]) });
}
fn cancel(world: &mut World, name: &[u8]) {
    world.down(Event::CancelRelay { run: fixtures::RUN_SEVEN, attempt: fixtures::ATTEMPT_TWO, call: name.into() });
}
fn redial(world: &mut World) {
    world.engine(fixtures::engine_accepting());
    world.advance(Duration::from_millis(200));
    world.down(Event::Hello { hello: fixtures::hello() });
}
fn kinds(world: &World, channel: usize) -> Vec<u16> {
    world.records()[channel].iter().map(|record| record.kind).collect()
}
fn decoded_call(record: &skein_fake_channel::Observed) -> jig_protocol_worker_wire::Call {
    jig_protocol_worker_wire::Call::decode(&fixtures::codec(), &mut Reader::new(&record.body)).expect("call")
}
fn answers(world: &World) -> Vec<&host::Relayed> {
    world
        .root()
        .iter()
        .filter_map(|request| match request {
            Request::Down { down: host::LinkDown::Relayed { answer, .. }, .. } => Some(answer),
            Request::Ready
            | Request::Resend { .. }
            | Request::GracePassed
            | Request::Closed
            | Request::Lost
            | Request::Down { .. }
            | Request::Value { .. }
            | Request::Dial
            | Request::Lower { .. }
            | Request::Close { .. }
            | Request::Abort { .. }
            | Request::Fact { .. }
            | Request::RelayCancelled { .. } => None,
        })
        .collect()
}
#[test]
fn a_relay_answered_on_a_later_channel_reaches_its_run_once() {
    let mut world = world();
    call(&mut world, b"first");
    world.lose_channel();
    redial(&mut world);
    world.play(fixtures::relayed(b"first", false));
    world.play(fixtures::relayed(b"first", false));
    cancel(&mut world, b"first");
    assert_eq!(answers(&world), [&host::Relayed::Answer { error: false, text: b"answer".as_slice().into() }]);
    assert!(!world.root().iter().any(|request| matches!(request, Request::RelayCancelled { .. })));
}
#[test]
fn a_relay_cancelled_while_the_link_is_down_is_never_sent_again() {
    let mut world = world();
    call(&mut world, b"first");
    world.lose_channel();
    cancel(&mut world, b"first");
    cancel(&mut world, b"first");
    redial(&mut world);
    assert_eq!(kinds(&world, 1), [0x0100]);
    let cancelled: Vec<_> = world
        .root()
        .iter()
        .filter_map(|request| match request {
            Request::RelayCancelled { run, attempt, call } => Some((*run, *attempt, call.as_ref())),
            Request::Ready
            | Request::Resend { .. }
            | Request::GracePassed
            | Request::Closed
            | Request::Lost
            | Request::Down { .. }
            | Request::Value { .. }
            | Request::Dial
            | Request::Lower { .. }
            | Request::Close { .. }
            | Request::Abort { .. }
            | Request::Fact { .. } => None,
        })
        .collect();
    assert_eq!(cancelled, [(fixtures::RUN_SEVEN, fixtures::ATTEMPT_TWO, b"first".as_slice())]);
}
#[test]
fn a_busy_call_goes_up_as_busy_and_ends_its_relay() {
    let mut world = world();
    call(&mut world, b"first");
    world.play(fixtures::relayed(b"first", true));
    assert_eq!(answers(&world), [&host::Relayed::Busy]);
    world.lose_channel();
    redial(&mut world);
    assert_eq!(kinds(&world, 1), [0x0100]);
}
#[test]
fn relays_go_again_after_the_hello_before_the_hubs_resends() {
    let mut limits = fixtures::limits();
    limits.sizes.calls = 3;
    let mut world = open_world(&limits);
    for name in [b"third".as_slice(), b"first", b"second"] {
        call(&mut world, name);
    }
    world.lose_channel();
    redial(&mut world);
    world.down(Event::Up { up: fixtures::turn(), parts: Box::new([]) });
    let channels = world.records();
    let records = &channels[1];
    assert_eq!(records.iter().map(|record| record.kind).collect::<Vec<_>>(), [0x0100, 0x0102, 0x0102, 0x0102, 0x0101]);
    for (record, name) in records[1..4].iter().zip([b"third".as_slice(), b"first", b"second"]) {
        let call = decoded_call(record);
        assert_eq!(call.name(), name);
    }
}
#[test]
fn an_answer_after_cancellation_never_reaches_the_hub() {
    let mut world = world();
    call(&mut world, b"first");
    cancel(&mut world, b"first");
    world.play(fixtures::relayed(b"first", false));
    assert!(answers(&world).is_empty());
}

#[test]
fn a_full_relay_replay_keeps_one_hello_steps_outputs_bounded() {
    let mut limits = fixtures::limits();
    limits.sizes.calls = 40;
    let mut world = open_world(&limits);
    for name in 0_u32..40 {
        call(&mut world, &name.to_be_bytes());
    }
    world.lose_channel();
    redial(&mut world);
    let channels = world.records();
    assert_eq!(channels[1].len(), 41);
    assert_eq!(channels[1][0].kind, 0x0100);
    for (record, name) in channels[1][1..].iter().zip(0_u32..40) {
        let call = decoded_call(record);
        assert_eq!(call.name(), name.to_be_bytes());
    }
}
