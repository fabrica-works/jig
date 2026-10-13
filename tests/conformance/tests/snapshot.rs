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

fn compare(scenario: Scenario, seed: u64) {
    let cuts = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let reached = std::rc::Rc::clone(&cuts);
    let baseline = replay::chosen_cuts_traced(scenario, seed, Some(true), &[1, 2, 3], move |cut, resumed| {
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
    assert_eq!(*cuts.borrow(), vec![1, 2, 3]);
    assert!(baseline.commits >= 3);
}

#[test]
fn late_copy_snapshots_match_complete_fresh_state() {
    compare(Scenario::LateCopy, 81);
}

#[test]
fn other_hand_snapshots_match_complete_fresh_state() {
    compare(Scenario::OtherHand, 82);
}

#[test]
fn uncertain_restarts_snapshots_match_complete_fresh_state() {
    compare(Scenario::UncertainRestarts, 83);
}

#[test]
fn changed_judge_snapshots_match_complete_fresh_state() {
    compare(Scenario::ChangedJudge, 84);
}

#[test]
fn shrinking_pool_snapshots_match_complete_fresh_state() {
    compare(Scenario::ShrinkingPool, 85);
}

#[test]
fn narrowing_policy_snapshots_match_complete_fresh_state() {
    compare(Scenario::NarrowingPolicy, 86);
}

#[test]
fn standing_task_snapshots_match_complete_fresh_state() {
    compare(Scenario::StandingTask, 87);
}

#[test]
fn run_budget_snapshots_match_complete_fresh_state() {
    compare(Scenario::RunBudget, 88);
}
