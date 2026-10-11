use jig_conformance::{
    Cut,
    scenarios::{ALL, Scenario, run},
};
use jig_conformance_world::Testing;
#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

#[test]
fn every_shared_adversarial_story_runs_on_the_testing_application() {
    for scenario in ALL {
        run::<Testing>(scenario, Cut::None, 31)
            .unwrap_or_else(|error| panic!("{scenario:?}: {error:?}"))
            .expect("application binds all required kinds");
    }
}

fn chosen_cut(scenario: Scenario) {
    let commit = if scenario == Scenario::RunBudget { 6 } else { 7 };
    run::<Testing>(scenario, Cut::AfterCommit(commit), 31)
        .unwrap_or_else(|error| panic!("{scenario:?}, commit {commit}: {error:?}"))
        .expect("application binds all required kinds");
}

#[test]
fn late_copy_chosen_effect_admission_and_completion_cut_ends_the_story() {
    chosen_cut(Scenario::LateCopy);
}

#[test]
fn other_hand_chosen_effect_admission_and_completion_cut_ends_the_story() {
    chosen_cut(Scenario::OtherHand);
}

#[test]
fn uncertain_restarts_chosen_effect_admission_and_completion_cut_ends_the_story() {
    chosen_cut(Scenario::UncertainRestarts);
}

#[test]
fn changed_judge_chosen_effect_admission_and_completion_cut_ends_the_story() {
    chosen_cut(Scenario::ChangedJudge);
}

#[test]
fn shrinking_pool_chosen_effect_admission_and_completion_cut_ends_the_story() {
    chosen_cut(Scenario::ShrinkingPool);
}

#[test]
fn narrowing_policy_chosen_effect_admission_and_completion_cut_ends_the_story() {
    chosen_cut(Scenario::NarrowingPolicy);
}

#[test]
fn standing_task_chosen_effect_admission_and_completion_cut_ends_the_story() {
    chosen_cut(Scenario::StandingTask);
}

#[test]
fn run_budget_chosen_effect_admission_and_completion_cut_ends_the_story() {
    chosen_cut(Scenario::RunBudget);
}
