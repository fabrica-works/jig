use jig_tasks_world::{run_story, run_story_choices, stories::Story};

// Stable draw stream for timing/price and fault/cut variations after exhaustive choices.
const DRAW_SOURCE: u64 = 47;
const DRAWN_SEEDS: usize = 8;

#[test]
fn random_faults_and_restart_cuts_reach_every_retained_ending() {
    let mut seen = std::collections::BTreeSet::new();
    let markers = jig_tasks_world::coverage::MARKERS;
    let choices = Story::all();
    assert_eq!(choices.iter().copied().collect::<std::collections::BTreeSet<_>>().len(), 48);
    for (index, choice) in choices.into_iter().enumerate() {
        let timing_seed = u64::try_from(index).expect("48 choice timings");
        let (trace, frozen) = run_story_choices(timing_seed, choice);
        assert_eq!(frozen.results.len(), 5, "seed {timing_seed}, choice {choice:?}");
        seen.extend(trace.coverage.0);
    }
    let mut random = skein_lib::Rng::new(DRAW_SOURCE);
    let mut drawn = std::collections::BTreeSet::new();
    while drawn.len() < DRAWN_SEEDS {
        drawn.insert(random.below(256));
    }
    eprintln!("lifecycle drawn seeds={drawn:?}");
    for seed in drawn {
        let (trace, frozen) = run_story(seed);
        assert_eq!(frozen.results.len(), 5, "seed {seed}");
        seen.extend(trace.coverage.0);
    }
    for marker in markers {
        assert!(seen.contains(&marker), "sweep reaches {marker:?}");
    }
}
