use jig_domain_host as host;
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
fn lost(world: &World) -> usize {
    world.root().iter().filter(|request| matches!(request, Request::Lost)).count()
}
fn turn(world: &mut World) {
    world.down(Event::Up { up: fixtures::turn(), parts: Box::new([]) });
}
fn ping(world: &mut World) {
    world.play(fixtures::frame(5, &[]));
}
fn fact(world: &mut World) {
    world.down(Event::Up {
        up: host::LinkUp::Activity {
            run: fixtures::RUN_SEVEN,
            attempt: fixtures::ATTEMPT_TWO,
            activity: host::Activity::LongDone,
        },
        parts: Box::new([]),
    });
}
#[test]
fn a_silent_engine_is_lost_at_the_silence_deadline() {
    let mut world = world();
    world.silence_engine();
    world.advance(Duration::from_millis(14_999));
    assert_eq!(lost(&world), 0);
    world.advance(Duration::from_millis(1));
    assert_eq!(lost(&world), 1);
}
#[test]
fn an_engine_that_stops_reading_is_lost_by_stall() {
    let mut world = world();
    world.withhold_write_room();
    turn(&mut world);
    for _ in 0..5 {
        world.advance(Duration::from_secs(5));
        ping(&mut world);
    }
    world.advance(Duration::from_millis(4_999));
    assert_eq!(lost(&world), 0);
    world.advance(Duration::from_millis(1));
    assert_eq!(lost(&world), 1);
}
#[test]
fn facts_are_dropped_past_the_reserve_while_turns_still_go() {
    let mut world = world();
    world.withhold_write_room();
    for _ in 0..6 {
        fact(&mut world);
    }
    assert_eq!(world.dropped_facts(), 2);
    turn(&mut world);
    assert_eq!(lost(&world), 0);
    world.grant_write_room();
    let channels = world.records();
    assert_eq!(
        channels[0].iter().map(|record| record.kind).collect::<Vec<_>>(),
        [0x0100, 0x0104, 0x0104, 0x0104, 0x0104, 0x0101]
    );
}
#[test]
fn an_idle_channel_sends_a_ping_after_the_interval() {
    let mut world = world();
    world.advance(Duration::from_millis(4_999));
    assert_eq!(world.records()[0].iter().filter(|record| record.kind == 5).count(), 0);
    world.advance(Duration::from_millis(1));
    assert_eq!(world.records()[0].iter().filter(|record| record.kind == 5).count(), 1);
}
#[test]
fn a_ping_restarts_the_silence_deadline() {
    let mut world = world();
    world.advance(Duration::from_secs(14));
    ping(&mut world);
    world.advance(Duration::from_millis(14_999));
    assert_eq!(lost(&world), 0);
    world.advance(Duration::from_millis(1));
    assert_eq!(lost(&world), 1);
}
#[test]
fn a_nonfact_that_fills_the_cap_loses_the_link() {
    let mut world = world();
    world.withhold_write_room();
    let cap = jig_protocol_worker_wire::sizes::worker_cap(&fixtures::SIZES).expect("cap");
    for _ in 0..=cap.cap.frames {
        if world.has_channel() {
            turn(&mut world);
        }
    }
    assert_eq!(lost(&world), 1);
}
