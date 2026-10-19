//! A send under a name, in front of the engine (client/domain/client.md, 6; contract.md, 3.1).

use super::{PROOF_WAIT, SECOND_STOP, STOP, Story, TASK, fixtures, stop};
use jig_client::Outcome;
use skein_lib::RequestKey;

/// A name the application gives, as an agent's key digested would be.
const NAME: RequestKey = [23; 16];

#[test]
fn a_second_key_naming_a_held_name_is_told_held() {
    let mut story = Story::live(fixtures::SEED);
    story.send_named(STOP, NAME, stop(TASK));
    story.send_named(SECOND_STOP, NAME, stop(TASK));
    assert!(story.run_until(|s| !s.answered(STOP).is_empty(), PROOF_WAIT));
    assert_eq!(story.answered(SECOND_STOP), vec![Outcome::Held]);
    assert_eq!(story.made_under(NAME), 1);
}
