//! A request carrying a value, in front of the engine (protocol/client.md, 5
//! and 14): sent once and never resent, kept by the engine before the core
//! sees it, and never in the small store nor a root input (the referee's
//! rules, checked on every exchange, save and input).

use super::{FAMILY_REQUEST, HELD_WAIT, PROJECT_NUMBER, PROOF_WAIT, Referee, STOP, Story, family_change, fixtures};
use jig_client::{Answered, Ask, Outcome, contract};
use skein_lib::RequestKey;

/// The name the application gives the carried request.
const NAME: RequestKey = [29; 16];
/// The value it carries, and another.
const VALUE: &[u8] = b"a value no store row holds";
const OTHER_VALUE: &[u8] = b"another value under the name";
/// The first number the engine draws for a carried value (store.md, 9).
const FIRST: u64 = 1 << 63;

fn change() -> Ask {
    Ask::Family {
        scope: contract::Scope::Project(PROJECT_NUMBER),
        record: jig_client_protocol_world::family::request(&family_change()),
    }
}
fn answered(story: &mut Story, key: jig_client::Key) -> Vec<Outcome> {
    assert!(story.run_until(|s| !s.answered(key).is_empty(), PROOF_WAIT));
    story.answered(key)
}

#[test]
fn a_carried_request_is_answered_once_and_its_value_stays_kept() {
    let mut story = Story::live(fixtures::SEED);
    let exchanges = story.exchanges();
    story.carry(FAMILY_REQUEST, NAME, change(), VALUE);
    let answers = answered(&mut story, FAMILY_REQUEST);
    let [Outcome::Answered(Answered::Family(answer))] = answers.as_slice() else {
        panic!("one owner's answer: {answers:?}");
    };
    assert!(jig_client_protocol_world::family::answer(answer).is_some());
    assert_eq!(story.exchanges(), exchanges + 1);
    assert_eq!(story.carried(), vec![FIRST]);
    assert_eq!(story.made_under(NAME), 1);
}

#[test]
fn the_same_name_and_value_again_gets_the_first_answer_and_its_value_is_erased() {
    let mut story = Story::live(fixtures::SEED);
    story.carry(FAMILY_REQUEST, NAME, change(), VALUE);
    let first = answered(&mut story, FAMILY_REQUEST);
    story.carry(STOP, NAME, change(), VALUE);
    assert_eq!(answered(&mut story, STOP), first);
    assert_eq!(story.carried(), vec![FIRST]);
    assert_eq!(story.made_under(NAME), 1);
}

#[test]
fn the_same_name_with_another_value_is_refused_once_and_its_value_is_erased() {
    let mut story = Story::live(fixtures::SEED);
    story.carry(FAMILY_REQUEST, NAME, change(), VALUE);
    answered(&mut story, FAMILY_REQUEST);
    story.carry(STOP, NAME, change(), OTHER_VALUE);
    // One answer: jig refuses a family's request under jig's owner, which
    // the link reads as undecodable, so the client tells it lost.
    assert_eq!(answered(&mut story, STOP).len(), 1);
    assert_eq!(story.carried(), vec![FIRST]);
    assert_eq!(story.made_under(NAME), 1);
}

#[test]
fn a_carried_request_whose_answer_is_cut_is_lost_and_never_resent() {
    let mut story = Story::live(fixtures::SEED);
    story.cut_answer();
    let exchanges = story.exchanges();
    story.carry(FAMILY_REQUEST, NAME, change(), VALUE);
    assert_eq!(answered(&mut story, FAMILY_REQUEST), vec![Outcome::Lost]);
    assert!(!story.run_until(|s| s.exchanges() > exchanges + 1, HELD_WAIT));
    assert_eq!(story.made_under(NAME), 1);
    assert_eq!(story.carried(), vec![FIRST]);
}

#[test]
fn a_carried_value_sent_twice_or_saved_is_caught() {
    let mut referee = Referee::default();
    referee.carry(VALUE);
    assert_eq!(referee.carrying(NAME, VALUE, Some(fixtures::FIRST_OWNER)), Ok(()));
    assert_eq!(
        referee.carrying(NAME, VALUE, Some(fixtures::FIRST_OWNER)),
        Err("carried value sent more often than it was given")
    );
    assert_eq!(referee.saved(VALUE), Err("carried value reached the small store"));
}
