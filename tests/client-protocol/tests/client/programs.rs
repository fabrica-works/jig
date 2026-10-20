//! The public producer crosses both real client translations.
use super::{Story, fixtures};
use jig_client::{Answered, Ask, Asker, Key, Outcome, contract as c};
use skein_lib::Duration;
const FIRST: Key = Key { asker: Asker(7), number: 81 };
const SECOND: Key = Key { asker: Asker(7), number: 82 };
const WAIT: Duration = Duration::from_secs(1);
fn declaration() -> Ask {
    Ask::Jig(Box::new(c::Ask::AccountChange {
        holder: c::AccountHolder::Own,
        change: c::AccountChange::DeclareProgram { provider: 3, join: None, name: b"personal".as_slice().into() },
    }))
}
fn answer(story: &mut Story, key: Key) -> c::Answer {
    assert!(story.run_until(|s| !s.answered(key).is_empty(), WAIT));
    let answers = story.answered(key);
    let [Outcome::Answered(Answered::Jig(answer))] = answers.as_slice() else { panic!("one answer: {answers:?}") };
    answer.clone()
}
fn declared(story: &mut Story) -> u32 {
    story.send_named(FIRST, [81; 16], declaration());
    let answer = answer(story, FIRST);
    let c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::AccountProgramDeclared { source, name })) = answer.clone()
    else {
        panic!("declared: {answer:?}")
    };
    assert_eq!(name.as_ref(), b"personal");
    source
}
#[test]
fn a_public_declaration_returns_the_source_name() {
    let mut story = Story::program_sources(fixtures::SEED);
    declared(&mut story);
}
#[test]
fn a_public_check_answers_after_requesting_inventory() {
    let mut story = Story::program_sources(fixtures::SEED);
    let source = declared(&mut story);
    story.send(
        SECOND,
        Ask::Jig(Box::new(c::Ask::AccountChange {
            holder: c::AccountHolder::Own,
            change: c::AccountChange::CheckProgram { source },
        })),
    );
    assert_eq!(answer(&mut story, SECOND), c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::AccountChanged)));
}
#[test]
fn a_public_sign_out_names_the_offline_worker() {
    let mut story = Story::program_sources(fixtures::SEED);
    let source = declared(&mut story);
    story.send(SECOND, Ask::Jig(Box::new(c::Ask::SignOut { holder: c::AccountHolder::Own, source })));
    assert_eq!(
        answer(&mut story, SECOND),
        c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::AccountSignedOut {
            confirmed: false,
            failed_hosts: Box::new([c::AccountHostFailure {
                name: b"unavailable".as_slice().into(),
                why: c::AccountHostWhy::NotInContact
            }])
        }))
    );
}
#[test]
fn a_public_declaration_replays_under_its_retained_name() {
    let mut story = Story::program_sources(fixtures::SEED);
    let source = declared(&mut story);
    story.send_named(SECOND, [81; 16], declaration());
    assert_eq!(
        answer(&mut story, SECOND),
        c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::AccountProgramDeclared {
            source,
            name: b"personal".as_slice().into()
        }))
    );
}

#[test]
fn program_declarations_carry_no_secret() {
    let mut story = Story::program_sources(fixtures::SEED);
    declared(&mut story);
    assert!(story.carried().is_empty());
}
