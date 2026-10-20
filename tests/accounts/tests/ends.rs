//! The accounts child owns sign-in progress and the bounded newest history of its ends.
use jig_accounts_world::{LIMITS, World, provider, providers};
use jig_domain_accounts::{End, Ended, Event, Failure, Holder, Pending, Refusal, Request, Signing};
use skein_lib::Token;

const ALICE: u64 = 7;
const BOB: u64 = 9;
const PROVIDER: u32 = 1;

fn started() -> World {
    let mut world = World::with(providers(vec![provider(PROVIDER, true, false)]), &[]);
    world.step(Event::Want);
    world
}
fn ask(world: &mut World, by: u64, holder: Holder) -> u32 {
    let requests = world.step(Event::SignIn { request: Token::new(by), holder, by, provider: PROVIDER });
    requests
        .into_iter()
        .find_map(|request| match request {
            Request::Signing(Signing::SignIn { source, .. }) => Some(source),
            _ => None,
        })
        .expect("a sign-in started")
}
fn finish(world: &mut World, source: u32) {
    world.step(Event::SignedIn {
        source,
        subject: b"subject".as_slice().into(),
        display: b"Name".as_slice().into(),
        declined: false,
    });
}
fn end(by: u64, source: u32, end: End) -> Ended {
    Ended { request: Token::new(by), source, by, end }
}

#[test]
fn a_sign_in_under_way_is_listed_with_who_asked_it_until_it_ends() {
    let mut world = started();
    let source = ask(&mut world, ALICE, Holder::Party(ALICE));
    assert_eq!(
        world.domain.pending().as_slice(),
        &[Pending { source, provider: PROVIDER, holder: Holder::Party(ALICE), by: ALICE }]
    );
    finish(&mut world, source);
    assert!(world.domain.pending().is_empty());
}

#[test]
fn a_deployment_sign_in_is_asked_by_a_person_and_held_by_the_deployment() {
    let mut world = started();
    let source = ask(&mut world, BOB, Holder::Deployment);
    assert_eq!(
        world.domain.pending().as_slice(),
        &[Pending { source, provider: PROVIDER, holder: Holder::Deployment, by: BOB }]
    );
    finish(&mut world, source);
    let ended = world.domain.last_end(BOB).expect("the deployment sign-in ended");
    let End::Made { account, .. } = ended.end else { panic!("made: {ended:?}") };
    assert_eq!(ended.by, BOB);
    assert_eq!(world.domain.account(account).expect("made").holder(), Holder::Deployment);
}

#[test]
fn an_end_names_who_asked_it() {
    let mut world = started();
    let made = ask(&mut world, ALICE, Holder::Party(ALICE));
    finish(&mut world, made);
    // The new source is numbered first; its account takes the following number.
    const ACCOUNT: u32 = 1;
    assert_eq!(world.domain.last_end(ALICE), Some(end(ALICE, made, End::Made { account: ACCOUNT, declined: false })));
    let renewed = ask(&mut world, ALICE, Holder::Party(ALICE));
    finish(&mut world, renewed);
    let renewal = end(ALICE, renewed, End::Renewed { account: ACCOUNT, declined: false });
    assert_eq!(world.domain.last_end(ALICE), Some(renewal));
    let refused = ask(&mut world, BOB, Holder::Party(BOB));
    finish(&mut world, refused);
    assert_eq!(
        world.domain.last_end(BOB),
        Some(end(BOB, refused, End::Refused { refusal: Refusal::Held { provider: PROVIDER } }))
    );
    assert_eq!(world.domain.last_end(ALICE), Some(renewal));
}

#[test]
fn a_full_history_of_ends_drops_its_oldest_and_loses_no_fact() {
    let mut world = started();
    let lost = world.domain.facts_lost();
    // One distinct asker per retained end, then one past the history bound.
    for by in 0..=u64::from(LIMITS.facts) {
        let source = ask(&mut world, by, Holder::Deployment);
        world.step(Event::SignInFailed { source, failure: Failure::Cancelled });
    }
    assert_eq!(world.domain.last_end(0), None);
    assert!(world.domain.last_end(u64::from(LIMITS.facts)).is_some());
    assert_eq!(world.ended().len(), usize::try_from(LIMITS.facts).expect("facts fit"));
    assert_eq!(world.domain.facts_lost(), lost);
}
