//! Accounts as accounts.md, 3 has them: numbered, held, with sources each in its own state.
use jig_accounts_world::{World, account, provider, providers, source};
use jig_domain_accounts::{Configured, Event, FromComponent, Held, Holder, Kind, Request, State, Stored, ToComponent};
use skein_lib::{Duration, Time, Wall};

const ALICE: Holder = Holder::Party(9);
const BOB: Holder = Holder::Party(4);

fn want(source: u32) -> Request {
    Request::Component(ToComponent::Want { account: source })
}
fn granted(source: u32) -> Event {
    Event::Component(FromComponent::Granted { account: source, generation: 1, valid: Duration::from_secs(100) })
}
/// Providers 1 and 2 offer a sign-in and a key; 3 offers none of its own: the `none` source.
fn fresh() -> World {
    World::with(providers(vec![provider(1, true, true), provider(2, true, false), provider(3, false, false)]), &[])
}
fn signed_in(number: u32, provider: u32, holder: Holder, subject: &[u8]) -> Vec<Stored> {
    vec![account(number, provider, holder, subject), source(number, number + 1, 0, Kind::SignIn)]
}

#[test]
fn configured_accounts_are_the_deployments_with_one_signed_in_source() {
    let mut world = World::with(
        providers(vec![provider(1, true, false)]),
        &[Configured { number: 7, provider: Some(1) }, Configured { number: 8, provider: None }],
    );
    let analyst = world.domain.account(7).expect("configured");
    assert_eq!((analyst.holder(), analyst.provider(), analyst.subject()), (Holder::Deployment, Some(1), &b""[..]));
    assert_eq!(analyst.sources().iter().collect::<Vec<_>>(), vec![(&0, &7)]);
    assert_eq!(world.domain.account(8).expect("configured").provider(), None);
    assert_eq!(world.domain.source(8).expect("its source").kind, Kind::SignIn);
    assert_eq!(world.domain.next_number(), 9);
    assert_eq!(world.step(Event::Want), vec![want(7), want(8)]);
}

#[test]
fn numbers_are_never_given_twice() {
    let mut world = fresh();
    world.restore(vec![Stored::Next { number: 12 }]);
    world.step(Event::Want);
    let made = world.domain.account(12).expect("the counter's first number");
    assert_eq!((made.holder(), made.provider()), (Holder::Deployment, Some(3)));
    assert_eq!(world.domain.next_number(), 14);
    let mut world = fresh();
    world.restore(vec![
        Stored::Next { number: 2 },
        account(40, 9, ALICE, &[b'x'; 256]),
        source(41, 50, 0, Kind::SignIn),
    ]);
    assert_eq!(world.domain.unrestored(), 2);
    assert_eq!(world.domain.next_number(), 51, "a record refused still passes its numbers");
    world.step(Event::Want);
    assert!(world.domain.account(51).is_some());
}

#[test]
fn the_none_account_is_made_at_the_first_start_alone() {
    let mut world = fresh();
    assert_eq!(world.domain.accounts(), 0);
    assert_eq!(world.step(Event::Want), vec![]);
    assert_eq!(world.domain.accounts(), 1);
    let none = world.domain.account(0).expect("made at the start");
    assert_eq!((none.holder(), none.provider()), (Holder::Deployment, Some(3)));
    let fact = world.domain.source(1).expect("its source");
    assert_eq!((fact.kind, fact.state, fact.usable), (Kind::None, State::Granted, true));
    assert!(world.domain.usable(0));
    assert_eq!(world.domain.grant(0, world.now), None, "a none source holds nothing to grant");
    let mut again = fresh();
    again.restore(vec![
        account(0, 3, Holder::Deployment, b""),
        source(0, 1, 0, Kind::None),
        Stored::Next { number: 2 },
    ]);
    again.step(Event::Want);
    assert_eq!((again.domain.accounts(), again.domain.next_number()), (1, 2));
}

#[test]
fn a_sources_state_is_its_own() {
    let mut world = fresh();
    world.restore(vec![account(1, 1, Holder::Deployment, b"team"), source(1, 2, 0, Kind::SignIn)]);
    world.restore(vec![source(1, 3, 1, Kind::Key)]);
    assert_eq!(world.step(Event::Want), vec![want(2)], "the component serves signed-in sources alone");
    world.step(granted(2));
    world.facts();
    world.step(Event::Limited { account: 3, reset: None });
    let facts = world.facts();
    assert_eq!(facts.len(), 1);
    assert_eq!((facts[0].account, facts[0].source, facts[0].kind), (1, 3, Kind::Key));
    assert_eq!(facts[0].state, State::Limited { until: None });
    let signed = world.domain.source(2).expect("kept");
    assert_eq!((signed.state, signed.usable), (State::Granted, true));
    assert_eq!(world.domain.grant(1, world.now).expect("its sign-in's").account, 2);
    world.step(Event::Limited { account: 2, reset: None });
    assert!(!world.domain.usable(1), "the first source unusable holds the account's runs");
    assert_eq!(world.views()[0].source, 2);
    let mut world = fresh();
    world.restore(vec![account(1, 1, Holder::Deployment, b"team"), source(1, 3, 0, Kind::Key)]);
    world.restore(vec![source(1, 2, 1, Kind::SignIn)]);
    world.step(Event::Want);
    world.step(granted(2));
    assert!(!world.domain.usable(1), "the first source, unusable, holds the account though a later one is usable");
    assert_eq!(world.views()[0].source, 3);
}

#[test]
fn a_kept_limit_means_the_same_after_a_restart() {
    let mut world = fresh();
    world.wall = Wall::from_nanos(1_000_000_000_000);
    let key = |number, place, kept| Stored::Source {
        account: 1,
        number,
        place,
        kind: Kind::Key,
        name: Box::new([]),
        held: Some(kept),
    };
    world.restore(vec![
        account(1, 1, Holder::Deployment, b""),
        key(2, 0, Held::Limited { reset: Some(Wall::from_nanos(1_060_000_000_000)) }),
        key(3, 1, Held::Limited { reset: Some(Wall::from_nanos(999_000_000_000)) }),
        key(4, 2, Held::Refused),
    ]);
    let until = Time::ZERO.saturating_add(Duration::from_secs(60));
    assert_eq!(world.domain.source(2).expect("kept").state, State::Limited { until: Some(until) });
    assert_eq!(world.domain.source(3).expect("kept").state, State::Asking, "a reset passed while down lifts it");
    assert_eq!(world.domain.source(4).expect("kept").state, State::Refused);
    assert!(world.domain.source(4).expect("kept").attention);
    assert_eq!(world.domain.next_deadline(), Some(until));
}

#[test]
fn records_that_break_the_rules_are_counted_not_restored() {
    let mut world = fresh();
    world.restore(signed_in(0, 1, ALICE, b"alice"));
    let named = |number, place, name: &[u8]| Stored::Source {
        account: 0,
        number,
        place,
        kind: Kind::Program,
        name: name.into(),
        held: None,
    };
    world.restore(vec![
        account(0, 2, BOB, b"bob"),
        account(2, 1, BOB, b"alice"),
        account(3, 1, BOB, &[b's'; 256]),
        source(9, 4, 0, Kind::Key),
        source(0, 5, 0, Kind::Key),
        source(0, 6, 1, Kind::SignIn),
        named(7, 1, b"Upper"),
        named(8, 1, b"runner"),
        named(10, 2, b"runner"),
        account(11, 2, BOB, b"bob"),
    ]);
    assert_eq!(world.domain.unrestored(), 8);
    world.step(Event::Want);
    assert_eq!(world.domain.unrestored(), 9, "an account with no source is dropped at the start");
    assert_eq!(world.domain.accounts(), 2, "alice's and the none account");
    assert_eq!(world.domain.account(0).expect("kept").sources().len(), 2);
}
