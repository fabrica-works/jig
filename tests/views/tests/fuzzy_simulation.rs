use jig_domain_views::{Call, Event, Follow, TurnName};
use jig_views_world::{LIMITS, World};
use skein_lib::{Rng, Token, Wall};
use std::boxed::Box;
fn run(seed: u64) -> Vec<String> {
    let mut world = World::seeded(LIMITS, seed);
    for watcher in 1..=4 {
        world.watch_from(watcher, 6 + watcher, Some(Follow { run: Token::new(7), turn: None, offset: 0 }));
    }
    let mut rng = Rng::new(seed);
    let mut turn = 1;
    for tick in 0..80 {
        let state = rng.next_u64();
        if tick % 17 == 0 {
            turn += 1;
        }
        world.changed(7 + state % 4);
        let call = if state.is_multiple_of(7) {
            Some(Call {
                identity: jig_domain_views::CallIdentity {
                    owner: Token::new(tick),
                    sequence: 1,
                    retry: 0,
                    id: Box::new([]),
                },
                tool: b"tool".as_slice().into(),
                input: vec![u8::try_from(state % 256).expect("byte"); 8].into_boxed_slice(),
                cut: false,
                began: Some(Wall::EPOCH),
            })
        } else {
            None
        };
        world.text(7, TurnName { attempt: 1, turn, retry: 0 }, &[u8::try_from(state % 256).expect("byte"); 7], call);
        for watcher in 1..=4 {
            if state.rotate_left(u32::try_from(watcher).expect("small")).is_multiple_of(3) {
                world.delivered(watcher);
            }
            if state.rotate_right(u32::try_from(watcher).expect("small")).is_multiple_of(5) {
                world.stream_delivered(watcher);
            }
        }
    }
    world.settle();
    for watcher in 1..=4 {
        world.step(Event::Unwatch { watcher: Token::new(watcher) });
    }
    world.settle();
    assert_eq!(world.referee.live(), 0, "seed {seed}");
    world.log().to_vec()
}
#[test]
fn random_subjects_and_slow_streams_replay_and_settle_after_the_last_change() {
    for seed in 1..=64 {
        assert_eq!(run(seed), run(seed), "seed {seed}");
    }
}
