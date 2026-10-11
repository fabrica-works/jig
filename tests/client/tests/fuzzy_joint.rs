//! Random people on authentic core worlds; each drawn engine/client pair is checked.
use jig_client_world::joint::random::{self, ClientFault, EngineFault};
use skein_world::domain::Verdict;
// New sweep's complete replayable range. No earlier production finding is omitted.
const FIRST_SEED: u64 = 4500;
const SEEDS: u64 = 64;
const PINNED: &[u64] = &[];
fn sweep(engine: EngineFault, client: ClientFault) {
    let mut reached = 0;
    for seed in (FIRST_SEED..FIRST_SEED + SEEDS).chain(PINNED.iter().copied()) {
        if random::assigned(seed) != (engine, client) {
            continue;
        }
        let (outcome, reach) = random::world(seed);
        assert_eq!(outcome.verdict, Verdict::Passed, "{engine:?}/{client:?} seed {seed}: {outcome:?}");
        assert_eq!((reach.engine, reach.client), (engine, client));
        assert!(outcome.judged.0 > 0 && outcome.faults > 0);
        assert!(reach.submitted > 0 && reach.durable_answers >= 2 && reach.acknowledged > 0);
        reached += 1;
    }
    assert!(reached > 0, "each actual drawn pair reached: {engine:?}/{client:?}");
}

#[test]
fn random_people_with_held_commits_and_drop_watch() {
    sweep(EngineFault::Held, ClientFault::DropWatch);
}

#[test]
fn random_people_with_held_commits_and_slow() {
    sweep(EngineFault::Held, ClientFault::Slow);
}

#[test]
fn random_people_with_held_commits_and_no_answer() {
    sweep(EngineFault::Held, ClientFault::NoAnswer);
}

#[test]
fn random_people_with_held_commits_and_reload() {
    sweep(EngineFault::Held, ClientFault::Reload);
}

#[test]
fn random_people_with_held_commits_and_double_press() {
    sweep(EngineFault::Held, ClientFault::DoublePress);
}

#[test]
fn random_people_with_crash_commits_and_drop_watch() {
    sweep(EngineFault::Crash, ClientFault::DropWatch);
}

#[test]
fn random_people_with_crash_commits_and_slow() {
    sweep(EngineFault::Crash, ClientFault::Slow);
}

#[test]
fn random_people_with_crash_commits_and_no_answer() {
    sweep(EngineFault::Crash, ClientFault::NoAnswer);
}

#[test]
fn random_people_with_crash_commits_and_reload() {
    sweep(EngineFault::Crash, ClientFault::Reload);
}

#[test]
fn random_people_with_crash_commits_and_double_press() {
    sweep(EngineFault::Crash, ClientFault::DoublePress);
}

#[test]
fn random_people_with_failed_commits_and_drop_watch() {
    sweep(EngineFault::Failed, ClientFault::DropWatch);
}

#[test]
fn random_people_with_failed_commits_and_slow() {
    sweep(EngineFault::Failed, ClientFault::Slow);
}

#[test]
fn random_people_with_failed_commits_and_no_answer() {
    sweep(EngineFault::Failed, ClientFault::NoAnswer);
}

#[test]
fn random_people_with_failed_commits_and_reload() {
    sweep(EngineFault::Failed, ClientFault::Reload);
}

#[test]
fn random_people_with_failed_commits_and_double_press() {
    sweep(EngineFault::Failed, ClientFault::DoublePress);
}
