use jig_engine_workers_world::{
    World,
    fixtures::{self, SECRET_A, SECRET_B, WORKER_A, WORKER_B},
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
fn busy(world: &mut World, channel: Token) {
    world.down(Event::TurnBusy { channel, run: Token::new(7), attempt: Token::new(2), turn: 3 });
}
fn closed(world: &World) -> usize {
    world.root().iter().filter(|r| matches!(r, Request::Closed)).count()
}
#[test]
fn a_stop_refuses_a_new_opening_as_stopping() {
    let mut world = world();
    world.down(Event::Stop);
    let connection = world.connect(fixtures::reporting_worker(WORKER_A, SECRET_A));
    world.run();
    assert_eq!(world.refusal(connection), Some((261, Box::from(b"".as_slice()))));
    assert!(world.root().is_empty());
}
#[test]
fn a_close_waits_for_queued_records_to_drain() {
    let mut world = world();
    let (connection, channel) = reporting(&mut world);
    world.withhold_write_room(connection);
    busy(&mut world, channel);
    world.down(Event::Stop);
    world.down(Event::Close);
    world.run();
    assert!(!world.gone(connection));
    assert_eq!(closed(&world), 0);
    world.grant_write_room(connection);
    world.run();
    assert_eq!(world.observed(connection).iter().map(|r| r.kind).collect::<Vec<_>>(), [0x0184]);
    assert!(world.gone(connection));
    assert_eq!(closed(&world), 1);
    referee::judge(world.root()).expect("closed last");
}
#[test]
fn an_abort_closes_every_connection_at_once() {
    let mut world = world();
    let (first, channel) = reporting(&mut world);
    let (second, _) = world.reporting(WORKER_B, SECRET_B);
    world.withhold_write_room(first);
    busy(&mut world, channel);
    world.down(Event::Abort);
    world.run();
    assert!(world.gone(first));
    assert!(world.gone(second));
    assert!(world.observed(first).is_empty());
    assert_eq!(closed(&world), 1);
    referee::judge(world.root()).expect("closed last");
}
#[test]
fn an_abort_interrupts_a_close_that_waits_for_drain() {
    let mut world = world();
    let (connection, channel) = reporting(&mut world);
    world.withhold_write_room(connection);
    busy(&mut world, channel);
    world.down(Event::Close);
    world.run();
    world.down(Event::Abort);
    world.run();
    assert!(world.gone(connection));
    assert_eq!(closed(&world), 1);
}
#[test]
fn a_closing_channel_is_lost_when_its_output_stalls() {
    let mut world = world();
    let (connection, channel) = reporting(&mut world);
    world.withhold_write_room(connection);
    busy(&mut world, channel);
    world.down(Event::Close);
    world.advance(Duration::from_millis(29_999));
    assert!(!world.gone(connection));
    world.advance(Duration::from_millis(1));
    assert!(world.gone(connection));
    assert_eq!(closed(&world), 1);
}
#[test]
fn a_closed_component_has_only_one_terminal() {
    let mut world = world();
    world.down(Event::Close);
    world.down(Event::Abort);
    world.down(Event::Close);
    assert_eq!(closed(&world), 1);
}
