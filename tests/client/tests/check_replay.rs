//! Focused cause reproduction for the optional check across a crash retry.
use jig_client_world::joint::random;
use skein_world::domain::Verdict;
const CRASH_DOUBLE_PRESS_SEED: u64 = 4500;
#[test]
fn a_retried_decision_keeps_the_facts_seen_before_a_crash() {
    let (outcome, _) = random::world(CRASH_DOUBLE_PRESS_SEED);
    assert_eq!(outcome.verdict, Verdict::Passed);
}
