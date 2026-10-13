use jig_engine_workers_world::{
    World,
    fixtures::{self, SECRET_A, WORKER_A},
    referee,
};
use jig_protocol_engine_workers::{Event, Request};
use skein_lib::{Duration, Token};

fn world() -> World {
    World::new(&fixtures::limits(), &fixtures::workers(), 1)
}
fn reporting(world: &mut World) -> (Token, Token) {
    world.reporting(WORKER_A, SECRET_A)
}
fn ping(world: &mut World, connection: Token) {
    world.play(connection, fixtures::frame(5, &[]));
    world.run();
}
fn busy(world: &mut World, channel: Token) {
    world.down(Event::TurnBusy { channel, run: Token::new(7), attempt: Token::new(2), turn: 3 });
}
fn observed(world: &World, connection: Token) -> Vec<u16> {
    world.observed(connection).iter().map(|r| r.kind).collect()
}
fn lost(world: &World) -> usize {
    world.root().iter().filter(|r| matches!(r, Request::Lost { .. })).count()
}

#[test]
fn a_silent_worker_is_lost_at_the_silence_deadline_and_not_before() {
    let mut world = world();
    let (connection, _) = reporting(&mut world);
    world.advance(Duration::from_millis(14_999));
    assert!(!world.gone(connection));
    assert_eq!(lost(&world), 0);
    world.advance(Duration::from_millis(1));
    assert!(world.gone(connection));
    assert_eq!(lost(&world), 1);
    referee::judge(world.root()).expect("ordered loss");
}

#[test]
fn a_worker_that_stops_reading_is_lost_by_stall() {
    let mut world = world();
    let (connection, channel) = reporting(&mut world);
    world.withhold_write_room(connection);
    busy(&mut world, channel);
    for _ in 0..5 {
        world.advance(Duration::from_secs(5));
        ping(&mut world, connection);
    }
    world.advance(Duration::from_millis(4_999));
    assert!(!world.gone(connection));
    world.advance(Duration::from_millis(1));
    assert!(world.gone(connection));
    assert_eq!(lost(&world), 1);
}

#[test]
fn an_idle_channel_sends_a_ping_after_the_interval() {
    let mut world = world();
    let (connection, _) = reporting(&mut world);
    world.advance(Duration::from_millis(4_999));
    assert!(world.observed(connection).is_empty());
    world.advance(Duration::from_millis(1));
    assert_eq!(observed(&world, connection), [5]);
    assert_eq!(world.root().len(), 1);
}

#[test]
fn a_link_that_fills_its_cap_is_lost() {
    let mut world = world();
    let (connection, channel) = reporting(&mut world);
    world.withhold_write_room(connection);
    let cap = jig_protocol_worker_wire::sizes::engine_cap(&fixtures::SIZES).expect("cap");
    for _ in 0..cap.frames {
        busy(&mut world, channel);
    }
    assert!(!world.gone(connection));
    busy(&mut world, channel);
    world.run();
    assert!(world.gone(connection));
    assert_eq!(lost(&world), 1);
}

#[test]
fn a_stream_ending_mid_frame_loses_the_channel_once() {
    let mut world = world();
    let (connection, _) = reporting(&mut world);
    let frame = fixtures::turn_frame();
    world.inject(connection, &frame.bytes()[..10]);
    world.end_stream(connection);
    world.run();
    world.end_stream(connection);
    world.run();
    assert!(world.gone(connection));
    assert_eq!(lost(&world), 1);
}

#[test]
fn a_ping_restarts_the_silence_deadline() {
    let mut world = world();
    let (connection, _) = reporting(&mut world);
    world.advance(Duration::from_secs(10));
    ping(&mut world, connection);
    world.advance(Duration::from_millis(14_999));
    assert!(!world.gone(connection));
    world.advance(Duration::from_millis(1));
    assert!(world.gone(connection));
}

#[test]
fn a_drained_record_restarts_the_ping_interval() {
    let mut world = world();
    let (connection, channel) = reporting(&mut world);
    world.advance(Duration::from_secs(4));
    busy(&mut world, channel);
    world.run();
    world.advance(Duration::from_millis(4_999));
    assert_eq!(observed(&world, connection), [0x0184]);
    world.advance(Duration::from_millis(1));
    assert_eq!(observed(&world, connection), [0x0184, 5]);
}

#[test]
fn a_stream_failure_loses_the_channel_once() {
    let mut world = world();
    let (connection, _) = reporting(&mut world);
    world.fail_stream(connection);
    world.run();
    world.fail_stream(connection);
    world.run();
    assert!(world.gone(connection));
    assert_eq!(lost(&world), 1);
}

#[test]
fn a_workers_refusal_loses_the_channel_once() {
    let mut world = world();
    let (connection, _) = reporting(&mut world);
    let frame = skein_channel::control_frame(
        &skein_channel::Control::Refuse { reason: 258, text: Box::new([]) },
        &fixtures::channel_limits(),
    )
    .expect("refusal");
    world.play(connection, frame);
    world.run();
    assert!(world.gone(connection));
    assert_eq!(lost(&world), 1);
}

#[test]
fn draining_one_frame_restarts_the_stall_deadline() {
    let mut world = world();
    let (connection, channel) = reporting(&mut world);
    world.withhold_write_room(connection);
    busy(&mut world, channel);
    busy(&mut world, channel);
    world.advance(Duration::from_secs(10));
    ping(&mut world, connection);
    world.grant_one_frame(connection);
    world.run();
    for _ in 0..2 {
        world.advance(Duration::from_secs(10));
        ping(&mut world, connection);
    }
    world.advance(Duration::from_millis(9_999));
    assert!(!world.gone(connection));
    world.advance(Duration::from_millis(1));
    assert!(world.gone(connection));
    assert_eq!(lost(&world), 1);
}
