//! Account keys cross the value boundary before a core input can name them.
use super::{STOP, Story, fixtures};
use jig_client::{Answered, Ask, Outcome, contract as c};
use skein_lib::{Duration, sha256};
const VALUE: &[u8] = b"an LLM account key sent once";
const NAME: [u8; 16] = [53; 16];
const WAIT: Duration = Duration::from_secs(1);
fn ask(replace: bool, digest: [u8; 32]) -> Ask {
    Ask::Jig(Box::new(c::Ask::AccountChange {
        holder: c::AccountHolder::Own,
        change: if replace {
            c::AccountChange::ReplaceKey { source: 999, digest }
        } else {
            c::AccountChange::AddKey { provider: 999, join: None, digest }
        },
    }))
}
#[test]
fn a_key_ask_carries_its_value_into_the_secrets_store() {
    for replace in [false, true] {
        let mut story = Story::live(fixtures::SEED);
        story.carry(STOP, NAME, ask(replace, sha256::digest(VALUE)), VALUE);
        assert!(story.run_until(|s| !s.account_key_inputs().is_empty(), WAIT));
        assert_eq!(
            story.account_key_inputs(),
            &[jig_domain_core::Carried { number: 1 << 63, digest: sha256::digest(VALUE) }]
        );
    }
}
#[test]
fn a_key_ask_without_a_value_is_malformed() {
    for replace in [false, true] {
        let mut story = Story::live(fixtures::SEED);
        story.send(STOP, ask(replace, sha256::digest(VALUE)));
        assert!(story.run_until(|s| s.malformed_requests() == 1, WAIT));
        assert!(story.account_key_inputs().is_empty());
        assert!(story.carried().is_empty());
    }
}
#[test]
fn a_value_on_another_ask_is_not_a_record() {
    let mut story = Story::live(fixtures::SEED);
    story.carry(STOP, NAME, Ask::Jig(Box::new(c::Ask::Stop { project: 1, task: 9 })), VALUE);
    assert!(story.run_until(|s| s.malformed_requests() == 1, WAIT));
    assert!(story.account_key_inputs().is_empty());
    assert!(story.carried().is_empty());
}
#[test]
fn a_key_digest_different_from_its_value_is_not_a_record() {
    for replace in [false, true] {
        let mut story = Story::live(fixtures::SEED);
        story.carry(STOP, NAME, ask(replace, sha256::digest(b"another LLM key")), VALUE);
        assert!(story.run_until(|s| s.malformed_requests() == 1, WAIT));
        assert!(story.account_key_inputs().is_empty());
        assert!(story.carried().is_empty());
    }
}
#[test]
fn an_unused_carried_key_is_erased() {
    let mut story = Story::live(fixtures::SEED);
    story.carry(STOP, NAME, ask(false, sha256::digest(VALUE)), VALUE);
    assert!(story.run_until(|s| !s.answered(STOP).is_empty(), WAIT));
    assert!(matches!(story.answered(STOP).as_slice(), [Outcome::Answered(Answered::Jig(c::Answer::Refused { .. }))]));
    assert_eq!(story.erased_carried(), &[1 << 63]);
    assert!(story.carried().is_empty());
}

#[test]
fn a_finish_is_answered_not_waiting_and_its_held_address_erased() {
    let mut story = Story::live(fixtures::SEED);
    let address = skein_lib::Secret::new(Box::from(b"http://127.0.0.1:1455/auth/callback?code=c&state=s".as_slice()));
    story.event(jig_client::accounts::finish_sign_in(STOP, NAME, c::AccountHolder::Own, 999, address));
    assert!(story.run_until(|s| !s.answered(STOP).is_empty(), WAIT));
    assert_eq!(
        story.answered(STOP).as_slice(),
        &[Outcome::Answered(Answered::Jig(c::Answer::Refused {
            refusal: c::Refusal::SignInRedirect { why: c::SignInRedirectWhy::NotWaiting },
            at: None,
        }))]
    );
    assert_eq!(story.erased_carried(), &[(1 << 63) + (1 << 62)]);
    assert!(story.carried().is_empty());
}
