use jig_protocol_worker_link::{Event, Request};
use jig_worker_link_world::{World, fixtures, referee};
use skein_lib::{Duration, Rng};
fn cancel(world: &mut World, name: &[u8]) {
    world.down(Event::CancelRelay { run: fixtures::RUN_SEVEN, attempt: fixtures::ATTEMPT_TWO, call: name.into() });
}
fn stopped(world: &World) -> bool {
    world.root().iter().any(|request| matches!(request, Request::Closed))
}
fn root_up(world: &mut World, rng: &mut Rng, name: &[u8]) {
    if !world.has_channel() {
        return;
    }
    world.down(Event::Up { up: fixtures::call(name), parts: Box::new([]) });
    world.down(Event::Up { up: fixtures::turn(), parts: Box::new([]) });
    if rng.below(2) == 0 {
        world.play(fixtures::relayed(name, rng.below(2) == 0));
    } else {
        cancel(world, name);
    }
    world.play(fixtures::turn_busy());
    if rng.below(2) == 0 {
        world.play(fixtures::turn_kept());
    }
}
fn exercise(seed: u64) -> (u32, u32, usize) {
    let mut world = World::new(&fixtures::limits(), seed);
    world.script_root();
    world.engine(fixtures::engine_accepting());
    world.run();
    let mut rng = Rng::new(seed);
    let mut cuts = 0;
    for turn in 0_u8..6 {
        root_up(&mut world, &mut rng, &[turn]);
        if world.has_channel() && rng.below(2) == 0 {
            world.lose_channel();
            cuts += 1;
        }
        if rng.below(3) == 0 {
            world.engine(fixtures::engine_refusing(258));
        } else {
            world.engine(fixtures::engine_accepting());
        }
        if world.has_channel() && rng.below(2) == 0 {
            world.withhold_write_room();
        }
        world.advance(Duration::from_millis(rng.between(1, 3_000)));
        if world.has_channel() {
            world.grant_write_room();
        }
        cancel(&mut world, &[turn]);
    }
    let stalls = world.blocked_writes();
    let resends = world.root().iter().filter(|request| matches!(request, Request::Resend { .. })).count();
    world.down(Event::Stop);
    world.advance(Duration::from_secs(30));
    if !stopped(&world) {
        assert!(world.dial_pending(), "only the dial may remain, seed {seed}");
        world.engine(fixtures::engine_accepting());
        world.connect_pending();
    }
    assert!(stopped(&world), "seed {seed}");
    referee::judge(&world).unwrap_or_else(|why| panic!("seed {seed}: {why}"));
    (cuts, stalls, resends)
}
#[test]
fn fuzzy_engines_cuts_and_stalls_keep_relays_ending_once() {
    let mut counts = (0, 0, 0);
    for seed in 0..40 {
        let (cuts, stalls, resends) = exercise(seed);
        counts.0 += cuts;
        counts.1 += stalls;
        counts.2 += resends;
    }
    assert!(counts.0 > 0 && counts.1 > 0 && counts.2 > 0);
}
