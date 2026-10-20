use jig_one_process_world::{fixtures, world};
use skein_world::domain::Verdict;

#[test]
fn a_person_s_chat_is_answered_by_the_in_process_worker() {
    let outcome = world::run(fixtures::calm(fixtures::SEED), fixtures::one_person_chat(b"Hello"));
    assert_eq!(outcome.verdict, Verdict::Passed);
    assert_eq!(outcome.chats_made, 1);
    assert_eq!(outcome.runs_answered, 1);
}

#[test]
fn a_signal_mid_run_drains_then_ends_the_worker_with_nothing_waiting() {
    let outcome = world::drain_mid_run(fixtures::calm(fixtures::SEED), false);
    assert_eq!(outcome.verdict, Verdict::Passed);
    assert!(outcome.drained);
    assert_eq!(outcome.chats_made, 1);
    assert_eq!(outcome.runs_answered, 0);
}

#[test]
fn the_next_start_settles_the_worker_s_claims_at_once() {
    let outcome = world::drain_mid_run(fixtures::calm(fixtures::SEED), true);
    assert_eq!(outcome.verdict, Verdict::Passed);
    assert!(outcome.drained);
    assert_eq!(outcome.claims_settled_at_restart, 1);
}

#[test]
fn a_kill_before_a_commit_loses_nothing_a_person_saw() {
    let outcome = crash(skein_world::Cut::Kill, 130);
    assert_eq!(outcome.verdict, Verdict::Passed);
    assert_eq!(outcome.restarts, 1);
    assert!(outcome.drained);
}
#[test]
fn a_power_loss_mid_run_restarts_and_adopts_nothing_from_the_dead_worker() {
    let outcome = crash(skein_world::Cut::PowerLoss, 145);
    assert_eq!(outcome.verdict, Verdict::Passed);
    assert_eq!(outcome.restarts, 1);
    assert_eq!(outcome.claims_settled_at_restart, 1);
    assert!(outcome.drained);
}

fn crash(cut: skein_world::Cut, before: u32) -> world::Outcome {
    world::crash(fixtures::calm(fixtures::SEED), cut, before)
}
