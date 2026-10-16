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

#[test]
fn a_sign_in_past_its_lifetime_is_answered_signed_out() {
    use jig_fake_parties::{Act, Ask, Reply};
    let config = jig_conformance_world::Config::new(39, false, 1, None);
    let lifetime = jig_core_world::world::limits().core.people.sign_in_lifetime.as_nanos();
    let mut world = jig_conformance::Harness::<Testing>::new(config, 39);
    world.drain().expect("signed in and assigned");
    world.advance(lifetime + 1);
    world.drain().expect("expiration committed");
    world.peers.peers.parties[0].extend(Box::new([Act::Request { key: [2; 16], ask: Ask::SeeResults { through: 0 } }]));
    world.drain().expect("expired sign-in answered");
    assert_eq!(world.peers.peers.parties[0].replies.last().map(|(_, reply)| reply), Some(&Reply::SignedOut));
}

#[test]
fn racing_decisions_chosen_effect_admission_and_completion_cut_ends_the_story() {
    chosen_cut(Scenario::RacingDecisions);
}

#[test]
fn conversation_ends_while_read_chosen_cut_ends_the_story() {
    const RESULTS_COMMIT: u64 = 17;
    run::<Testing>(Scenario::ConversationEndsWhileRead, Cut::AfterCommit(RESULTS_COMMIT), 757)
        .expect("results mark and forget survive their commit cut")
        .expect("application binds the conversation story");
}
