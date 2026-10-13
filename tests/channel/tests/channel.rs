use jig_channel_world::{fixtures, world};
use skein_world::domain::Verdict;

#[test]
fn a_person_s_chat_runs_on_the_test_worker_and_is_acknowledged_once() {
    let outcome = world::run(fixtures::calm(fixtures::SEED), fixtures::one_chat(b"Hello"));
    assert_eq!(outcome.verdict, Verdict::Passed);
    assert_eq!(outcome.turns_committed, 1);
    assert_eq!(outcome.answers_acknowledged, 1);
}
