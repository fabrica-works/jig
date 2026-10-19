//! A send under a name, in front of the engine (client/domain/client.md, 6; contract.md, 3.1).

use super::{PROOF_WAIT, SECOND_STOP, STOP, Story, TASK, assert_one_refusal, fixtures, stop};
use jig_client::{Answered, Outcome, contract};
use skein_lib::RequestKey;

/// A name the application gives, as an agent's key digested would be.
const NAME: RequestKey = [23; 16];
/// Another ask: the next task number, which no story starts.
const OTHER_TASK: u64 = TASK + 1;

#[test]
fn a_second_key_naming_a_held_name_is_told_held() {
    let mut story = Story::live(fixtures::SEED);
    story.send_named(STOP, NAME, stop(TASK));
    story.send_named(SECOND_STOP, NAME, stop(TASK));
    assert!(story.run_until(|s| !s.answered(STOP).is_empty(), PROOF_WAIT));
    assert_eq!(story.answered(SECOND_STOP), vec![Outcome::Held]);
    assert_eq!(story.made_under(NAME), 1);
}

#[test]
fn a_named_stop_kept_across_a_reload_is_answered_once_under_its_name() {
    let mut story = Story::live(fixtures::SEED);
    story.cut_answer();
    story.send_named(STOP, NAME, stop(TASK));
    assert!(story.run_until(|s| s.made_under(NAME) == 1, PROOF_WAIT));
    story.reload();
    assert!(story.run_until(|s| !s.answered(STOP).is_empty(), PROOF_WAIT));
    assert!(story.kept(STOP));
    assert_one_refusal(&story);
    assert_eq!(story.made_under(NAME), 1);
}

#[test]
fn a_name_asked_again_once_its_holder_is_erased_gets_the_first_answer() {
    let mut story = Story::live(fixtures::SEED);
    story.send_named(STOP, NAME, stop(TASK));
    assert!(story.run_until(|s| !s.answered(STOP).is_empty(), PROOF_WAIT));
    story.send_named(SECOND_STOP, NAME, stop(TASK));
    assert!(story.run_until(|s| !s.answered(SECOND_STOP).is_empty(), PROOF_WAIT));
    assert_eq!(story.answered(SECOND_STOP), story.answered(STOP));
    assert_eq!(story.made_under(NAME), 1);
}

#[test]
fn a_name_sent_with_another_ask_is_refused_as_a_conflict() {
    let mut story = Story::live(fixtures::SEED);
    story.send_named(STOP, NAME, stop(TASK));
    assert!(story.run_until(|s| !s.answered(STOP).is_empty(), PROOF_WAIT));
    story.send_named(SECOND_STOP, NAME, stop(OTHER_TASK));
    assert!(story.run_until(|s| !s.answered(SECOND_STOP).is_empty(), PROOF_WAIT));
    let answers = story.answered(SECOND_STOP);
    let [Outcome::Answered(Answered::Jig(contract::Answer::Refused { refusal, .. }))] = answers.as_slice() else {
        panic!("one refusal: {answers:?}");
    };
    assert_eq!(*refusal, contract::Refusal::KeyConflict);
    assert_eq!(story.made_under(NAME), 1);
}
