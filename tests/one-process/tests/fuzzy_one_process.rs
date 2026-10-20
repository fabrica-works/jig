use jig_one_process_world::{fixtures, world};
use skein_world::domain::Verdict;

#[test]
fn drawn_kills_and_drains_keep_the_conformance_rules() {
    for seed in 1..=7 {
        for (label, cut) in [("kill", skein_world::Cut::Kill), ("power-loss", skein_world::Cut::PowerLoss)] {
            let mut draw = skein_lib::Rng::new(seed);
            // Cut in the run/commit window after the real key request is kept.
            let before = 130 + u32::try_from(draw.below(20)).expect("bounded submission draw");
            eprintln!("{label} seed {seed}, before {before}");
            let outcome = world::crash(fixtures::calm(seed), cut, before);
            assert_eq!(outcome.verdict, Verdict::Passed, "{label} seed {seed}, before {before}");
            assert_eq!(outcome.restarts, 1, "{label} seed {seed}, before {before}");
            assert!(outcome.drained, "{label} seed {seed}, before {before}");
        }
    }
}
