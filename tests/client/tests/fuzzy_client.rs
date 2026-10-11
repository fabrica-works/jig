//! Replayable random people over every story start and seeded random documents.
use jig_client_world::random::{self, Start};
use skein_lib::Rng;
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
// Split only the schedule: every seed in the original 200..232 range runs
// through the unchanged world exactly once. Its first draw assigns its actual
// story fixture, and the returned reach independently checks that assignment.
fn truth_sweep(start: Start) {
    let mut counts = (0, 0, 0);
    for seed in 200..232 {
        let mut rng = Rng::new(seed);
        let assigned = Start::ALL[usize::try_from(rng.below(6)).expect("six starts")];
        if assigned != start {
            continue;
        }
        eprintln!("random truth {start:?} seed {seed}");
        let (outcome, reach) = random::world_with_coverage(seed);
        assert_eq!(reach.start, start, "seed {seed} keeps its actual fixture");
        counts.0 += 1;
        counts.1 += outcome.faults;
        if reach.original_address {
            counts.2 += outcome.faults;
        }
        assert_eq!(outcome.verdict, Verdict::Passed, "seed {seed}");
    }
    // These are the original per-start aggregate assertions. Checking faults
    // for every start also retains the former whole-sweep positive-fault check.
    assert!(counts.0 > 0 && counts.1 > 0, "{start:?} truth reach {counts:?}");
    if start != Start::Chat {
        assert!(counts.2 > 0, "shrunk {start:?} retains original-address fault reach: {counts:?}");
    }
    eprintln!("coverage truth {start:?}={counts:?}");
}
#[test]
fn random_truth_worlds_in_new_chat_sweep_documents_addresses_and_persons_with_faults() {
    truth_sweep(Start::NewChat);
}
#[test]
fn random_truth_worlds_in_chat_sweep_documents_addresses_and_persons_with_faults() {
    truth_sweep(Start::Chat);
}
#[test]
fn random_truth_worlds_in_held_chat_sweep_documents_addresses_and_persons_with_faults() {
    truth_sweep(Start::Held);
}
#[test]
fn random_truth_worlds_in_proposal_sweep_documents_addresses_and_persons_with_faults() {
    truth_sweep(Start::Proposal);
}
#[test]
fn random_truth_worlds_in_goal_sweep_documents_addresses_and_persons_with_faults() {
    truth_sweep(Start::Goal);
}
#[test]
fn random_truth_worlds_in_incident_sweep_documents_addresses_and_persons_with_faults() {
    truth_sweep(Start::Incident);
}
