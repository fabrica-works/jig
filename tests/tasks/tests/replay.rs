#[test]
fn seed_replays_the_full_frozen_world_and_fact_consumption_changes_no_behavior() {
    for seed in [23, 24] {
        assert_eq!(
            jig_tasks_world::run_story_traced(seed, true, true),
            jig_tasks_world::run_story_traced(seed, true, true)
        );
    }
    let (trace, consumed) = jig_tasks_world::run_story_traced(23, true, true);
    let (other, retained) = jig_tasks_world::run_story_traced(23, false, true);
    assert_eq!(trace, other);
    assert_eq!(consumed.records, retained.records);
    assert_eq!(consumed.replies, retained.replies);
    assert_eq!(consumed.contexts, retained.contexts);
    assert_eq!(consumed.activations, retained.activations);
    assert_eq!(consumed.runs, retained.runs);
    assert_eq!(consumed.stops, retained.stops);
    assert_eq!(consumed.closing, retained.closing);
    assert_eq!(consumed.results, retained.results);
}

#[test]
fn tracing_off_keeps_no_copies_and_on_replays_the_whole_task_boundary() {
    let (off, frozen) = jig_tasks_world::run_story_traced(23, true, false);
    let (on, replayed) = jig_tasks_world::run_story_traced(23, true, true);
    assert!(off.lines.is_empty(), "seed 23 keeps no sweep trace");
    assert_eq!(frozen, replayed, "seed 23 trace settings change no complete state");
    assert_eq!(off.coverage, on.coverage, "seed 23 typed coverage needs no trace");
    assert!(on.lines.len() > 100, "seed 23 records every input and referee observation");
    assert!(
        on.lines.iter().any(|line| line.contains("reply_to: ReplyTo(Token(")),
        "seed 23 keeps the full reply identity"
    );
    assert!(on.lines.iter().any(|line| line.contains("Stored { live:")), "seed 23 keeps full referee payloads");
    assert_eq!(on, jig_tasks_world::run_story_traced(23, true, true).0);
}
