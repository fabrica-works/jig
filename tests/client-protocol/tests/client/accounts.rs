//! LLM account sign-in through both real client protocol boundaries.
use jig_client::{Answered, Ask, Asker, Key, Outcome, contract as c};
use jig_client_protocol_world::{client::Story, fixtures};
use skein_lib::Duration;

const FIRST: Key = Key { asker: Asker(7), number: 1 };
const SECOND: Key = Key { asker: Asker(7), number: 2 };
const WAIT: Duration = Duration::from_secs(1);
fn ask() -> Ask {
    Ask::Jig(Box::new(c::Ask::SignIn { holder: c::AccountHolder::Own, provider: 3 }))
}
fn answered(story: &mut Story, key: Key) -> Vec<Outcome> {
    assert!(story.run_until(|s| !s.answered(key).is_empty(), WAIT));
    story.answered(key)
}
fn started(answers: &[Outcome]) -> (u32, &[u8]) {
    let [
        Outcome::Answered(Answered::Jig(c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::AccountSignInStarted {
            source,
            address,
        })))),
    ] = answers
    else {
        panic!("one account sign-in started answer: {answers:?}")
    };
    (*source, address)
}
#[test]
fn a_sign_in_asked_through_a_client_is_answered_with_the_address() {
    let mut story = Story::account_sign_in(fixtures::SEED, true);
    story.send(FIRST, ask());
    let answers = answered(&mut story, FIRST);
    let (source, address) = started(&answers);
    assert_eq!(address, format!("https://issuer.test/visit/{source}").as_bytes());
}
#[test]
fn a_sign_in_for_a_provider_with_none_is_not_offered() {
    let mut story = Story::account_sign_in(fixtures::SEED, false);
    story.send(FIRST, ask());
    assert_eq!(
        answered(&mut story, FIRST),
        vec![Outcome::Answered(Answered::Jig(c::Answer::Refused { refusal: c::Refusal::NotOffered, at: None }))]
    );
}
#[test]
fn a_sign_in_beyond_the_sign_ins_bound_is_refused_naming_it() {
    let mut story = Story::account_sign_in(fixtures::SEED, true);
    story.hold_account_sign_in_burst();
    story.send(FIRST, ask());
    story.send(SECOND, ask());
    assert!(story.run_until(|s| s.account_sign_in_burst_len() == 2, WAIT));
    story.release_account_sign_in_burst();
    assert_eq!(
        answered(&mut story, SECOND),
        vec![Outcome::Answered(Answered::Jig(c::Answer::Refused {
            refusal: c::Refusal::Limit { which: c::Limit::Admission { which: c::AdmissionLimit::SignIns } },
            at: None,
        }))]
    );
    started(&answered(&mut story, FIRST));
}
#[test]
fn a_sign_out_is_answered_with_confirmed() {
    let mut story = Story::account_sign_in(fixtures::SEED, true);
    story.send(FIRST, ask());
    let (source, _) = started(&answered(&mut story, FIRST));
    story.send(SECOND, Ask::Jig(Box::new(c::Ask::SignOut { holder: c::AccountHolder::Own, source })));
    assert_eq!(
        answered(&mut story, SECOND),
        vec![Outcome::Answered(Answered::Jig(c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::AccountSignedOut {
            confirmed: true
        }))))]
    );
}
#[test]
fn a_sign_in_sent_again_after_its_answer_gets_the_same_answer() {
    let mut story = Story::account_sign_in(fixtures::SEED, true);
    story.send_named(FIRST, [37; 16], ask());
    let first = answered(&mut story, FIRST);
    started(&first);
    story.send_named(SECOND, [37; 16], ask());
    assert_eq!(answered(&mut story, SECOND), first);
}
