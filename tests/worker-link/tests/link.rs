use jig_protocol_worker_link::{Event, Request};
use jig_worker_link_world::{World, fixtures};
use skein_lib::Duration;
fn world() -> World {
    let mut world = World::new(&fixtures::limits(), 1);
    world.engine(fixtures::engine_accepting());
    world.run();
    world
}
fn hello(world: &mut World) {
    world.down(Event::Hello { hello: fixtures::hello() });
}
fn redial(world: &mut World) {
    world.lose_channel();
    world.engine(fixtures::engine_accepting());
    world.advance(Duration::from_millis(1600));
}
fn ready(world: &World) -> usize {
    world.root().iter().filter(|event| matches!(event, Request::Ready)).count()
}
#[test]
fn the_hello_is_the_first_record_on_every_channel() {
    let mut world = world();
    for _ in 0..2 {
        world.down(Event::Up { up: fixtures::turn(), parts: Box::new([]) });
        hello(&mut world);
        hello(&mut world);
        world.down(Event::Up { up: fixtures::turn(), parts: Box::new([]) });
        redial(&mut world);
    }
    let records = world.records();
    for channel in &records[..2] {
        assert_eq!(channel.iter().map(|record| record.kind).collect::<Vec<_>>(), [0x0100, 0x0101]);
    }
}
#[test]
fn the_root_hears_ready_once_per_channel() {
    let mut world = world();
    assert_eq!(ready(&world), 1);
    hello(&mut world);
    redial(&mut world);
    assert_eq!(ready(&world), 2);
}
#[test]
fn a_lost_channel_is_told_once() {
    let mut world = world();
    world.lose_channel();
    world.lose_channel();
    assert_eq!(world.root().iter().filter(|event| matches!(event, Request::Lost)).count(), 1);
}
#[test]
fn a_turn_while_the_link_is_down_is_dropped() {
    let mut world = world();
    world.lose_channel();
    world.down(Event::Up { up: fixtures::turn(), parts: Box::new([]) });
    world.engine(fixtures::engine_accepting());
    world.advance(Duration::from_millis(100));
    hello(&mut world);
    assert_eq!(world.records()[1].iter().map(|record| record.kind).collect::<Vec<_>>(), [0x0100]);
}
#[test]
fn a_channel_carrying_the_hello_and_proving_resets_the_backoff() {
    let mut world = world();
    world.lose_channel();
    world.engine(fixtures::engine_accepting());
    world.advance(Duration::from_millis(100));
    hello(&mut world);
    world.advance(fixtures::LIMITS.proved);
    let lost = world.now();
    world.lose_channel();
    world.engine(fixtures::engine_accepting());
    world.advance(Duration::from_millis(100));
    let gap = world.dials()[2].saturating_since(lost);
    assert!(gap >= Duration::from_millis(50) && gap <= Duration::from_millis(100), "{gap:?}");
}
