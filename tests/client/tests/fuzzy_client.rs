//! Replayable random people over every story start and seeded random documents.
use jig_client_world::random::{self, Start};
use skein_world::domain::Verdict;
// Every first dedicated seed reached 8 faults in the unchanged 16-seed inventory.
// Two keep drawn slack. Chat retains 16: the 32 worlds never opened its original address.
const FIRST_START_SEED: u64 = 100;
const SHORT_START_SEEDS: u64 = 2;
const CHAT_START_SEEDS: u64 = 16;
fn sweep(start: Start) {
    let mut faults = 0;
    let count = if start == Start::Chat { CHAT_START_SEEDS } else { SHORT_START_SEEDS };
    for seed in FIRST_START_SEED..FIRST_START_SEED + count {
        eprintln!("{start:?} seed {seed}");
        let outcome = random::story_without_heap(seed, start);
        assert_eq!(outcome.verdict, Verdict::Passed, "{start:?} seed {seed}");
        faults += outcome.faults;
    }
    assert!(faults > 0);
}
#[test]
fn random_people_in_new_chat_starts() {
    sweep(Start::NewChat);
}
#[test]
fn random_people_in_chat_starts() {
    sweep(Start::Chat);
}
#[test]
fn random_people_in_held_chat_starts() {
    sweep(Start::Held);
}
#[test]
fn random_people_in_proposal_starts() {
    sweep(Start::Proposal);
}
#[test]
fn random_people_in_goal_starts() {
    sweep(Start::Goal);
}
#[test]
fn random_people_in_incident_starts() {
    sweep(Start::Incident);
}
#[test]
fn random_truth_worlds_sweep_documents_addresses_and_persons_with_faults() {
    let mut faults = 0;
    let mut reached = std::collections::BTreeMap::<Start, (u64, u64, u64)>::new();
    for seed in 200..232 {
        eprintln!("random truth seed {seed}");
        let (outcome, reach) = random::world_with_coverage(seed);
        let tally = reached.entry(reach.start).or_default();
        tally.0 += 1;
        tally.1 += outcome.faults;
        if reach.original_address {
            tally.2 += outcome.faults;
        }
        assert_eq!(outcome.verdict, Verdict::Passed, "seed {seed}");
        faults += outcome.faults;
    }
    assert!(faults > 0);
    for start in Start::ALL {
        let counts = reached.get(&start).expect("random documents reach every actual story fixture");
        assert!(counts.0 > 0 && counts.1 > 0, "{start:?} truth reach {counts:?}");
        if start != Start::Chat {
            assert!(counts.2 > 0, "shrunk {start:?} retains original-address fault reach: {counts:?}");
        }
    }
    eprintln!("coverage truth starts={reached:?}");
}
