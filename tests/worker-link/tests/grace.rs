use jig_protocol_worker_link::{Event, Request};
use jig_worker_link_world::{World, fixtures};
use skein_lib::Duration;
fn world() -> World {
    let mut world = World::new(&fixtures::limits(), 1);
    world.engine(fixtures::engine_accepting());
    world.run();
    world.down(Event::Hello { hello: fixtures::hello() });
    world
}
fn queued(world: &mut World) {
    world.withhold_write_room();
    world.down(Event::Up { up: fixtures::turn(), parts: Box::new([]) });
}
fn pending_stop() -> World {
    let mut world = World::new(&fixtures::limits(), 1);
    world.run();
    world.down(Event::Stop);
    world
}
fn count(world: &World, grace: bool) -> usize {
    world
        .root()
        .iter()
        .filter(
            |request| if grace { matches!(request, Request::GracePassed) } else { matches!(request, Request::Closed) },
        )
        .count()
}
#[test]
fn the_grace_passing_with_no_channel_tells_the_root_once() {
    let mut world = world();
    world.lose_channel();
    world.advance(Duration::from_millis(9_999));
    assert_eq!(count(&world, true), 0);
    world.advance(Duration::from_millis(1));
    assert_eq!(count(&world, true), 1);
    world.advance(Duration::from_secs(20));
    assert_eq!(count(&world, true), 1);
}
#[test]
fn a_channel_back_just_within_the_grace_stops_its_clock() {
    let mut world = world();
    world.lose_channel();
    world.advance(Duration::from_millis(9_999));
    world.engine(fixtures::engine_accepting());
    world.connect_pending();
    world.advance(Duration::from_millis(1));
    assert_eq!(count(&world, true), 0);
    assert!(world.has_channel());
}
#[test]
fn a_stop_never_redials() {
    let mut world = world();
    world.down(Event::Stop);
    world.advance(Duration::from_secs(60));
    assert_eq!(world.dials().len(), 1);
    assert_eq!(count(&world, false), 1);
    assert_eq!(world.next_deadline(), None);
    world.down(Event::Stop);
    world.down(Event::Abort);
    assert_eq!(count(&world, false), 1);
}
#[test]
fn stopping_a_pending_dial_waits_for_its_terminal() {
    let mut world = pending_stop();
    assert_eq!(count(&world, false), 0);
    world.down(Event::Unreachable);
    assert_eq!(count(&world, false), 1);
    assert_eq!(world.next_deadline(), None);
}
#[test]
fn an_abort_closes_even_when_output_cannot_drain() {
    let mut world = world();
    queued(&mut world);
    world.down(Event::Abort);
    assert_eq!(count(&world, false), 1);
    assert!(!world.has_channel());
    assert_eq!(world.next_deadline(), None);
}
#[test]
fn a_stop_drains_queued_output_before_closed() {
    let mut world = world();
    queued(&mut world);
    world.down(Event::Stop);
    assert_eq!(count(&world, false), 0);
    world.grant_write_room();
    assert_eq!(world.records()[0].iter().filter(|record| record.kind == 0x0101).count(), 1);
    assert_eq!(count(&world, false), 1);
    assert_eq!(world.next_deadline(), None);
}

#[test]
fn a_stop_with_stalled_output_is_bounded() {
    let mut world = world();
    queued(&mut world);
    world.down(Event::Stop);
    world.advance(Duration::from_millis(29_999));
    assert_eq!(count(&world, false), 0);
    world.advance(Duration::from_millis(1));
    assert_eq!(count(&world, false), 1);
    assert_eq!(world.next_deadline(), None);
}

#[test]
fn stopping_a_pending_dial_closes_its_late_connection() {
    let mut world = pending_stop();
    world.engine(fixtures::engine_accepting());
    world.connect_pending();
    assert_eq!(count(&world, false), 1);
    assert!(!world.has_channel());
    assert_eq!(world.next_deadline(), None);
}
