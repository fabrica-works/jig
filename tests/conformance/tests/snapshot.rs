use jig_conformance::{Cut, scenarios::Scenario};
use jig_conformance_world::{Testing, replay};
#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

#[test]
fn a_forked_snapshot_replays_to_the_durable_rows() {
    let world = replay::fresh(Scenario::LateCopy, Cut::None, 81, false).expect("fresh story");
    let rows = world.store.rows.clone();
    let forked = jig_conformance::Harness::<Testing>::from_snapshot(world.cold_snapshot()).expect("fork restarts");
    assert_eq!(forked.store.rows, rows);
}

fn compare(scenario: Scenario, seed: u64, selected: u64) {
    let cuts = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let reached = std::rc::Rc::clone(&cuts);
    let baseline = replay::chosen_cuts_traced(scenario, seed, Some(true), &[selected], move |cut, resumed| {
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
    .expect("chosen snapshot cut");
    assert_eq!(*cuts.borrow(), vec![selected]);
    assert!(baseline.commits >= selected);
    assert!(baseline.commits > 0);
}

#[test]
fn late_copy_first_snapshot_matches_complete_fresh_state() {
    compare(Scenario::LateCopy, 81, 1);
}

#[test]
fn late_copy_second_snapshot_matches_complete_fresh_state() {
    compare(Scenario::LateCopy, 81, 2);
}

#[test]
fn late_copy_third_snapshot_matches_complete_fresh_state() {
    compare(Scenario::LateCopy, 81, 3);
}

#[test]
fn other_hand_first_snapshot_matches_complete_fresh_state() {
    compare(Scenario::OtherHand, 82, 1);
}

#[test]
fn other_hand_second_snapshot_matches_complete_fresh_state() {
    compare(Scenario::OtherHand, 82, 2);
}

#[test]
fn other_hand_third_snapshot_matches_complete_fresh_state() {
    compare(Scenario::OtherHand, 82, 3);
}

#[test]
fn uncertain_restarts_first_snapshot_matches_complete_fresh_state() {
    compare(Scenario::UncertainRestarts, 83, 1);
}

#[test]
fn uncertain_restarts_second_snapshot_matches_complete_fresh_state() {
    compare(Scenario::UncertainRestarts, 83, 2);
}

#[test]
fn uncertain_restarts_third_snapshot_matches_complete_fresh_state() {
    compare(Scenario::UncertainRestarts, 83, 3);
}

#[test]
fn changed_judge_first_snapshot_matches_complete_fresh_state() {
    compare(Scenario::ChangedJudge, 84, 1);
}

#[test]
fn changed_judge_second_snapshot_matches_complete_fresh_state() {
    compare(Scenario::ChangedJudge, 84, 2);
}

#[test]
fn changed_judge_third_snapshot_matches_complete_fresh_state() {
    compare(Scenario::ChangedJudge, 84, 3);
}

#[test]
fn shrinking_pool_first_snapshot_matches_complete_fresh_state() {
    compare(Scenario::ShrinkingPool, 85, 1);
}

#[test]
fn shrinking_pool_second_snapshot_matches_complete_fresh_state() {
    compare(Scenario::ShrinkingPool, 85, 2);
}

#[test]
fn shrinking_pool_third_snapshot_matches_complete_fresh_state() {
    compare(Scenario::ShrinkingPool, 85, 3);
}

#[test]
fn narrowing_policy_first_snapshot_matches_complete_fresh_state() {
    compare(Scenario::NarrowingPolicy, 86, 1);
}

#[test]
fn narrowing_policy_second_snapshot_matches_complete_fresh_state() {
    compare(Scenario::NarrowingPolicy, 86, 2);
}

#[test]
fn narrowing_policy_third_snapshot_matches_complete_fresh_state() {
    compare(Scenario::NarrowingPolicy, 86, 3);
}

#[test]
fn standing_task_first_snapshot_matches_complete_fresh_state() {
    compare(Scenario::StandingTask, 87, 1);
}

#[test]
fn standing_task_second_snapshot_matches_complete_fresh_state() {
    compare(Scenario::StandingTask, 87, 2);
}

#[test]
fn standing_task_third_snapshot_matches_complete_fresh_state() {
    compare(Scenario::StandingTask, 87, 3);
}

#[test]
fn run_budget_first_snapshot_matches_complete_fresh_state() {
    compare(Scenario::RunBudget, 88, 1);
}

#[test]
fn run_budget_second_snapshot_matches_complete_fresh_state() {
    compare(Scenario::RunBudget, 88, 2);
}

#[test]
fn run_budget_third_snapshot_matches_complete_fresh_state() {
    compare(Scenario::RunBudget, 88, 3);
}
