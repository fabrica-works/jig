use jig_domain_host as host;
use jig_ops_domain as root;
use jig_ops_world::conformance::turn_backoff::World;
use skein_lib::{Time, Token};

fn busy_world() -> (World, Time) {
    let mut world = World::started();
    world.turn(1);
    world.turn(2);
    let deadline = world.domain.next_turn_deadline().expect("fleet's second turn is busy");
    world.settle();
    (world, deadline)
}

fn transcripts(world: &World) -> Vec<(u32, &[u8])> {
    world.saved.iter().map(|row| (row.turn, row.transcript.as_ref())).collect()
}

#[test]
fn an_engine_turn_answered_busy_is_kept_once_after_its_backoff() {
    let (mut world, deadline) = busy_world();
    assert!((50_000_000..=100_000_000).contains(&deadline.as_nanos()));
    assert_eq!(world.kept, [1]);
    assert_eq!(transcripts(&world), [(1, [1].as_slice())]);
    world.fire_at(Time::from_nanos(deadline.as_nanos() - 1));
    assert_eq!(world.kept, [1], "no retry before the deadline");
    world.fire_at(deadline);
    assert_eq!(world.kept, [1, 2]);
    assert_eq!(transcripts(&world), [(1, [1].as_slice()), (2, [2].as_slice())]);
    assert_eq!(world.domain.next_turn_deadline(), None);
    world.fire_at(Time::from_nanos(1_000_000_000));
    assert_eq!(world.kept, [1, 2], "the retained turn is acknowledged once");
}

#[test]
fn a_busy_turn_no_longer_retained_by_the_hub_is_dropped_at_its_deadline() {
    let (mut world, deadline) = busy_world();
    let row = &world.saved[0];
    world.send(root::Event::Host(host::Event::Down(host::LinkDown::TurnKept {
        run: Token::new(row.task),
        attempt: Token::new(row.attempt),
        turn: 2,
    })));
    world.settle();
    world.fire_at(deadline);
    assert_eq!(world.domain.next_turn_deadline(), None);
    assert_eq!(world.kept, [1]);
    assert_eq!(transcripts(&world), [(1, [1].as_slice())]);
}
