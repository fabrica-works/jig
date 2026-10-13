use jig_one_process_world::{fixtures, world};
use skein_world::domain::Verdict;

#[test]
fn a_person_s_chat_is_answered_by_the_in_process_worker() {
    let outcome = world::run(fixtures::calm(fixtures::SEED), fixtures::one_person_chat(b"Hello"));
    assert_eq!(outcome.verdict, Verdict::Passed);
    assert_eq!(outcome.chats_made, 1);
    assert_eq!(outcome.runs_answered, 1);
}
