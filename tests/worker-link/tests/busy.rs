use jig_domain_host as host;
use jig_protocol_worker_link::{Event, Request};
use jig_worker_link_world::{World, fixtures};
use skein_lib::Duration;
fn world() -> World {
    let mut world = World::new(&fixtures::limits(), 1);
    world.engine(fixtures::engine_accepting());
    world.run();
    world.down(Event::Hello { hello: fixtures::hello() });
    turn(&mut world);
    world
}
fn turn(world: &mut World) {
    world.down(Event::Up { up: fixtures::turn(), parts: Box::new([]) });
}
fn resends(world: &World) -> usize {
    world.root().iter().filter(|request| matches!(request, Request::Resend { .. })).count()
}
#[test]
fn a_busy_turn_is_asked_again_after_its_backoff() {
    let mut world = world();
    for expected in 1..=3 {
        world.play(fixtures::turn_busy());
        world.advance(Duration::from_millis(99));
        assert_eq!(resends(&world), expected - 1);
        world.advance(Duration::from_millis(101));
        assert_eq!(resends(&world), expected);
        assert!(world.root().iter().any(|request| matches!(request, Request::Resend { run, attempt, turn: 1 } if *run == fixtures::RUN_SEVEN && *attempt == fixtures::ATTEMPT_TWO)));
        turn(&mut world);
    }
    world.play(fixtures::turn_kept());
    world.advance(Duration::from_millis(200));
    assert_eq!(resends(&world), 3);
    assert_eq!(world.records()[0].iter().filter(|record| record.kind == 0x0101).count(), 4);
}
#[test]
fn a_busy_turn_kept_meanwhile_is_not_asked_again() {
    let mut world = world();
    world.play(fixtures::turn_busy());
    world.play(fixtures::turn_kept());
    world.advance(Duration::from_millis(200));
    assert_eq!(resends(&world), 0);
}
#[test]
fn a_loss_drops_every_busy_turn() {
    let mut world = world();
    world.play(fixtures::turn_busy());
    world.lose_channel();
    world.advance(Duration::from_millis(200));
    assert_eq!(resends(&world), 0);
}
#[test]
fn the_hub_never_hears_a_turn_busy() {
    let mut world = world();
    let before = world.root().len();
    world.play(fixtures::turn_busy());
    assert_eq!(world.root().len(), before);
    assert!(world.refusals().is_empty());
}
#[test]
fn acknowledging_the_attempt_cancels_its_busy_resend() {
    let mut world = world();
    world.play(fixtures::turn_busy());
    world.play(fixtures::acknowledge());
    world.advance(Duration::from_millis(200));
    assert_eq!(resends(&world), 0);
    assert!(
        world
            .root()
            .iter()
            .any(|request| matches!(request, Request::Down { down: host::LinkDown::Acknowledge { .. }, .. }))
    );
}
