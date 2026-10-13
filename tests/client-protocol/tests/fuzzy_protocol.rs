//! Replayable random people, stories and byte cuts share the joint referee.
use jig_client_protocol_world::stories as bytes;
use jig_client_world::{FrontFault, Until};
use skein_lib::{Duration, Rng};
const FIRST_SEED: u64 = 8600;
// Complete replayable range; no earlier production finding is omitted.
const SEEDS: u64 = 9;
#[test]
fn random_people_over_bytes_keep_the_referee_s_rules() {
    let mut reached = [0; 4];
    for seed in FIRST_SEED..FIRST_SEED + SEEDS {
        let mut rng = Rng::new(seed);
        let (mut world, client) = bytes::ready(seed);
        let choice = rng.between(0, 3);
        reached[usize::try_from(choice).expect("drawn range")] += 1;
        match choice {
            0 => world.front().cut_upload(),
            1 => world.fault(FrontFault::NoAnswer),
            2 => world.fault(FrontFault::DropWatch),
            3 => world.fault(FrontFault::Slow { span: Duration::from_millis(100) }),
            _ => panic!("drawn range"),
        }
        if rng.chance(500) {
            bytes::submit_stop(&mut world, client);
            bytes::held(&mut world, client);
        } else {
            let words = format!("Please continue {}", rng.between(0, 99));
            bytes::send_words(&mut world, client, words.as_bytes());
            bytes::finish(&mut world);
            assert!(world.front().words_reached_worker(words.as_bytes()), "drawn words seed {seed}");
        }
        if choice < 3 {
            assert!(world.front().cuts() > 0, "drawn cut seed {seed}");
        }
        world.random_person_without_button(client, b"Sign out");
        let outcome = world.run(Until::Done);
        assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed, "seed {seed}: {outcome:?}");
        assert!(world.front().rules().records > 0 && world.front().rules().domains > 0, "seed {seed}");
    }
    assert!(reached.iter().all(|count| *count > 0), "each drawn fault reached: {reached:?}");
}
