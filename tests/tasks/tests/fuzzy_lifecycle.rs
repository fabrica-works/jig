use jig_tasks_world::run_story;

#[test]
fn random_faults_and_restart_cuts_reach_every_retained_ending() {
    let mut seen = std::collections::BTreeSet::new();
    let markers = jig_tasks_world::coverage::MARKERS;
    for seed in 0..256 {
        let (trace, frozen) = run_story(seed);
        assert_eq!(frozen.results.len(), 5, "seed {seed}");
        for marker in markers {
            if trace.coverage.0.contains(&marker) {
                seen.insert(marker);
            }
        }
    }
    for marker in markers {
        assert!(seen.contains(marker), "sweep reaches {marker:?}");
    }
}
