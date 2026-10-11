use jig_conformance::{Cut, scenarios::Scenario};
use jig_conformance_world::replay;
#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

fn compare(scenario: Scenario, seed: u64) {
    let cuts = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let reached = std::rc::Rc::clone(&cuts);
    let baseline = replay::every_cut_traced(scenario, seed, Some(true), move |cut, resumed| {
        let actual = replay::outside(&resumed);
        assert!(resumed.outcome().crashes.contains(&cut));
        drop(resumed);
        let fresh = replay::fresh(scenario, Cut::AfterCommit(cut), seed, true).expect("fresh selected cut");
        assert_eq!(
            actual,
            replay::outside(&fresh),
            "{scenario:?}, cut {cut}, seed {seed}: complete outside state and whole trace"
        );
        reached.borrow_mut().push(cut);
    })
    .expect("all snapshot cuts");
    assert_eq!(*cuts.borrow(), (1..=baseline.commits).collect::<Vec<_>>());
    assert!(baseline.commits > 0);
}

#[test]
fn late_copy_every_snapshot_matches_complete_fresh_state() {
    compare(Scenario::LateCopy, 81);
}
#[test]
fn other_hand_every_snapshot_matches_complete_fresh_state() {
    compare(Scenario::OtherHand, 82);
}
#[test]
fn uncertain_restarts_every_snapshot_matches_complete_fresh_state() {
    compare(Scenario::UncertainRestarts, 83);
}
#[test]
fn changed_judge_every_snapshot_matches_complete_fresh_state() {
    compare(Scenario::ChangedJudge, 84);
}
#[test]
fn shrinking_pool_every_snapshot_matches_complete_fresh_state() {
    compare(Scenario::ShrinkingPool, 85);
}
#[test]
fn narrowing_policy_every_snapshot_matches_complete_fresh_state() {
    compare(Scenario::NarrowingPolicy, 86);
}
#[test]
fn standing_task_every_snapshot_matches_complete_fresh_state() {
    compare(Scenario::StandingTask, 87);
}
#[test]
fn run_budget_every_snapshot_matches_complete_fresh_state() {
    compare(Scenario::RunBudget, 88);
}
