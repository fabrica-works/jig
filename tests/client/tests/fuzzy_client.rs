//! Replayable random people over every story start and seeded random documents.
use jig_client_world::random::{self, Start};
use skein_world::domain::{Verdict, heap::Counting};
#[global_allocator]
static HEAP: Counting = Counting;
fn sweep(start: Start) {
    let mut faults = 0;
    for seed in 100..116 {
        eprintln!("{start:?} seed {seed}");
        let outcome = random::story(seed, start);
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
    for seed in 200..232 {
        eprintln!("random truth seed {seed}");
        let outcome = random::world(seed);
        assert_eq!(outcome.verdict, Verdict::Passed, "seed {seed}");
        faults += outcome.faults;
    }
    assert!(faults > 0);
}
