//! Trace settings change diagnostic retention only, never outside behavior.
use jig_ops_world::conformance::{Config, Ops, Story};
#[test]
fn tracing_off_keeps_no_copies_and_on_replays_the_whole_application_boundary() {
    let run = |enabled| {
        let mut world = jig_conformance::Harness::<Ops>::with_trace(Config::new(23, Story::Night), 23, enabled);
        let outcome = world.drain().expect("seed 23 cold-world replay");
        (world, outcome)
    };
    let (off, result) = run(false);
    let (on, traced) = run(true);
    assert!(off.trace.is_empty(), "seed 23 keeps no sweep trace");
    assert_eq!(result.commits, traced.commits);
    assert_eq!(result.iterations, traced.iterations);
    assert_eq!(result.stopped, traced.stopped);
    assert_eq!(result.crashes, traced.crashes);
    assert!(on.trace.len() > 10, "seed 23 keeps every application input/output");
    assert!(on.trace.iter().any(|line| line.starts_with("input Event(")), "seed 23 keeps full translated inputs");
    assert!(on.trace.iter().any(|line| line.starts_with("output Deliver(")), "seed 23 keeps full translated outputs");
    assert!(on.trace.iter().any(|line| line.contains("ReplyTo(Token(")), "seed 23 keeps the complete reply identity");
    let (again, _) = run(true);
    assert_eq!(&*on.trace, &*again.trace, "seed 23 replays the whole boundary trace");
}
