//! The worker's host child domain at random: many rough worlds, each settled,
//! every ending reached among them.

use std::collections::BTreeSet;

use jig_domain_host::Limits;
use jig_host_world::engine::ENDINGS;
use jig_host_world::{Settings, Span, World};

// The unchanged 1000-seed inventory stops growing at 372: 20 endings and 23 paths.
// Keep twice that exact prefix, including the rare agent-start cancellation.
const COVERAGE_PREFIX: u64 = 372;
const SWEEP_SEEDS: u64 = 2 * COVERAGE_PREFIX;

const ITERATIONS: u32 = 200_000;

fn run(settings: &Settings) -> World {
    let mut world = World::new(*settings);
    world.run(ITERATIONS);
    world
}

/// A rough world, varied by seed: some shut down, some lose contact for
/// longer than the grace, some let runs make several calls at once and hold
/// several inbound events.
fn rough(seed: u64) -> Settings {
    let rough = Settings::rough(seed);
    let shutdown = if seed.is_multiple_of(4) { Some(Span::millis(20_000, 120_000)) } else { None };
    let host = if seed.is_multiple_of(2) { rough.host } else { Limits { run_calls: 3, held: 3, ..rough.host } };
    Settings { shutdown, host, ..rough }
}

#[test]
fn random_worlds_settle_and_reach_every_ending() {
    let mut reached = BTreeSet::new();
    let mut seen = BTreeSet::new();
    let mut last_growth = 0;
    for seed in 0..SWEEP_SEEDS {
        let previous = (reached.len(), seen.len());
        let stats = run(&rough(seed)).stats();
        for ending in stats.endings.keys() {
            reached.insert(*ending);
        }
        let paths = [
            ("busy calls", stats.busy),
            ("calls unavailable", stats.unavailable),
            ("late calls", stats.parent.late_calls),
            ("relays unanswered", stats.engine.unanswered),
            ("events too large", stats.engine.too_large),
            ("events past the hold", stats.engine.full),
            ("events to a run ending", stats.engine.ending),
            ("stale messages", stats.stale),
            ("reports", stats.reports),
            ("cancels of every run", stats.cancel_alls),
            ("runs forgotten on reconnecting", stats.engine.forgotten),
            ("requests to agents gone", stats.parent.dropped),
            ("saves with a workspace changed", stats.parent.saves_stale),
            ("saves with a delivery failed", stats.parent.saves_failed),
            ("prepares aborted", stats.parent.aborts),
        ];
        for (path, count) in paths {
            if count > 0 {
                seen.insert(path);
            }
        }
        for path in stats.paths.keys() {
            seen.insert(path);
        }
        if previous != (reached.len(), seen.len()) {
            last_growth = seed + 1;
        }
    }
    assert!(last_growth <= COVERAGE_PREFIX, "coverage prefix grew to {last_growth}");
    let missing: Vec<&str> = ENDINGS.iter().copied().filter(|ending| !reached.contains(ending)).collect();
    assert!(missing.is_empty(), "every ending is reached: {missing:?} are not");
    let paths = [
        "duplicate assignments",
        "oversized outcomes",
        "endings said during a stop",
        "deliveries settled during a stop",
        "cancels as a workspace was prepared",
        "cancels as a workspace failed to prepare",
        "cancels as an agent failed to start",
        "cancels as an agent started",
    ];
    let missing: Vec<&str> = paths.iter().copied().filter(|path| !seen.contains(path)).collect();
    assert!(missing.is_empty(), "every path is taken: {missing:?} are not");
    assert_eq!(seen.len(), 15 + paths.len(), "every path is taken: only {seen:?} are");
}
