use jig_protocol_worker_link::{REFUSED_FRAMING, REFUSED_LIMITS, Request};
use jig_worker_link_world::{Trace, World, fixtures};
use skein_channel::Closed;
use skein_lib::Duration;

fn world() -> World {
    World::new(&fixtures::limits(), 1)
}
fn engine_world(engine: skein_fake_channel::ScriptedPeer) -> World {
    let mut world = world();
    world.engine(engine);
    world
}
fn accepting_world() -> World {
    engine_world(fixtures::engine_accepting())
}
fn check_redial(world: &mut World, low: u64, high: u64) {
    world.engine(fixtures::engine_accepting());
    world.advance(Duration::from_millis(high));
    assert_eq!(world.dials().len(), 2);
    let gap = world.dials()[1].saturating_since(world.dials()[0]);
    assert!(gap >= Duration::from_millis(low) && gap <= Duration::from_millis(high), "{gap:?}");
}
fn check_local_loss(world: &World, reason: u16) {
    assert_eq!(world.facts(), [Closed::RefusedHere(reason)]);
    assert!(!world.root().iter().any(|request| matches!(request, Request::Down { .. })));
    assert!(world.values().is_empty());
}
#[test]
fn the_worker_dials_at_once() {
    let mut world = accepting_world();
    world.run();
    assert_eq!(world.dials(), [skein_lib::Time::ZERO]);
}
#[test]
fn the_worker_opens_with_its_credential() {
    let mut world = accepting_world();
    world.run();
    let limits = jig_protocol_worker_wire::sizes::limits(&fixtures::SIZES).expect("small limits");
    let credential =
        jig_protocol_worker_wire::Credential::decode(&limits, &mut skein_lib::Reader::new(world.credential()))
            .expect("wire credential");
    assert_eq!(credential.name(), fixtures::WORKER_A);
    assert_eq!(credential.secret().expose(), fixtures::SECRET_A);
    assert_eq!(credential.payload(), fixtures::PAYLOAD);
}
#[test]
fn a_refused_worker_redials_after_its_backoff() {
    let mut world = engine_world(fixtures::engine_refusing(258));
    world.run();
    check_redial(&mut world, 50, 100);
}
#[test]
fn an_unreachable_engine_is_dialled_again_after_the_backoff() {
    let mut world = world();
    world.refuse_dial();
    world.run();
    check_redial(&mut world, 50, 100);
}
#[test]
fn each_jig_reason_is_redialled_after_the_backoff() {
    for reason in 256..=261 {
        let mut world = engine_world(fixtures::engine_refusing(reason));
        world.run();
        assert!(world.facts().contains(&Closed::RefusedPeer(reason)), "reason {reason}");
        check_redial(&mut world, 50, 100);
    }
}
#[test]
fn another_version_is_redialled_after_the_backoff() {
    let mut world = accepting_world();
    world.change_next_engine_version();
    world.run();
    check_redial(&mut world, 50, 100);
}
#[test]
fn engine_terms_below_the_largest_turn_are_refused_as_limits() {
    let mut world = engine_world(fixtures::engine_with_terms(0x0101, 1));
    world.run();
    assert_eq!(world.refusals(), [(2, Box::from(b"257".as_slice()))]);
    check_local_loss(&world, REFUSED_LIMITS);
    assert!(!world.has_channel());
    check_redial(&mut world, 50, 100);
    check_local_loss(&world, REFUSED_LIMITS);
}
#[test]
fn malformed_records_are_refused_as_framing_and_redialled() {
    // Inbound and TurnBusy exercise the ordinary decoder and the Busy-specific path.
    for kind in [0x0181, 0x0184] {
        let mut world = accepting_world();
        world.run();
        world.play(fixtures::frame(kind, b""));
        assert_eq!(world.refusals(), [(3, Box::from(b"".as_slice()))]);
        check_local_loss(&world, REFUSED_FRAMING);
        assert!(!world.trace().iter().any(|trace| matches!(trace, Trace::Busy(..))));
        assert!(!world.has_channel());
        check_redial(&mut world, 50, 100);
        check_local_loss(&world, REFUSED_FRAMING);
        assert_eq!(world.root().iter().filter(|request| matches!(request, Request::Lost)).count(), 1);
        assert_eq!(world.root().iter().filter(|request| matches!(request, Request::Ready)).count(), 2);
    }
}
#[test]
fn an_opening_past_the_handshake_deadline_is_closed_and_redialled() {
    let mut world = accepting_world();
    world.silence_next_engine();
    world.run();
    world.engine(fixtures::engine_accepting());
    world.advance(Duration::from_secs(1));
    assert_eq!(world.dials().len(), 1);
    world.advance(Duration::from_millis(100));
    assert_eq!(world.dials().len(), 2);
}
#[test]
fn a_channel_lost_before_it_proved_keeps_doubling() {
    let mut world = accepting_world();
    world.run();
    world.lose_channel();
    world.engine(fixtures::engine_accepting());
    world.advance(Duration::from_millis(100));
    world.lose_channel();
    world.engine(fixtures::engine_accepting());
    world.advance(Duration::from_millis(200));
    assert_eq!(world.dials().len(), 3);
    let gap = world.dials()[2].saturating_since(skein_lib::Time::ZERO.saturating_add(Duration::from_millis(100)));
    assert!(gap >= Duration::from_millis(100) && gap <= Duration::from_millis(200), "{gap:?}");
}
