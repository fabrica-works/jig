//! Accounts as accounts.md, 3 has them: numbered, held, with sources each in its own state, and
//! kept by the store: each change saved in the step that makes it, and restored at the start.
use jig_accounts_world::{LIMITS, World, account, provider, providers, source};
use jig_domain_accounts::{
    Bound, Event, Form, FromComponent, Held, Holder, Key, Kind, Refusal, Request, Scheme, SignIn, State, Stored,
    ToComponent, room_for_sign_in,
};
use skein_lib::{Duration, Time, Wall};
use std::collections::BTreeMap;

const ALICE: Holder = Holder::Party(9);
const BOB: Holder = Holder::Party(4);

fn want(source: u32) -> Request {
    Request::Component(ToComponent::Want { account: source })
}
fn save(record: Stored) -> Request {
    Request::Save { record }
}
fn granted(source: u32) -> Event {
    Event::Component(FromComponent::Granted {
        form: Form::Credential(Scheme::Bearer),
        account: source,
        generation: 1,
        valid: Duration::from_secs(100),
    })
}
/// Providers 1 and 2 offer a sign-in and a key; 3 offers none of its own: the `none` source.
fn fresh() -> World {
    World::with(providers(vec![provider(1, true, true), provider(2, true, false), provider(3, false, false)]), &[])
}
fn limit<T>(bound: Bound) -> Result<T, Refusal> {
    Err(Refusal::Limit { bound })
}
fn signed_in(number: u32, provider: u32, holder: Holder, subject: &[u8]) -> Vec<Stored> {
    vec![account(number, provider, holder, subject), source(number, number + 1, 0, Kind::SignIn)]
}

#[test]
fn an_unstarted_child_has_no_accounts_from_configuration() {
    let world = World::with(providers(vec![provider(1, true, false)]), &[]);
    assert_eq!(world.domain.accounts(), 0);
    assert_eq!(world.domain.next_number(), 0);
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
    let made = vec![account(0, 3, Holder::Deployment, b""), source(0, 1, 0, Kind::None), Stored::Next { number: 2 }];
    assert_eq!(
        world.step(Event::Want),
        made.iter()
            .cloned()
            .map(save)
            .chain([Request::Signing(jig_domain_accounts::Signing::Restored)])
            .collect::<Vec<_>>()
    );
    assert_eq!(world.domain.accounts(), 1);
    let none = world.domain.account(0).expect("made at the start");
    assert_eq!((none.holder(), none.provider()), (Holder::Deployment, Some(3)));
    let fact = world.domain.source(1).expect("its source");
    assert_eq!((fact.kind, fact.state, fact.usable), (Kind::None, State::Granted, true));
    assert!(world.domain.usable(0));
    assert_eq!(world.domain.grant(0, world.now), None, "a none source holds nothing to grant");
    let mut again = fresh();
    again.restore(made);
    assert_eq!(
        again.step(Event::Want),
        vec![Request::Signing(jig_domain_accounts::Signing::Restored)],
        "nothing made, nothing saved"
    );
    assert_eq!((again.domain.accounts(), again.domain.next_number()), (1, 2));
}

#[test]
fn a_subject_has_one_holder() {
    let mut world = fresh();
    world.restore(signed_in(5, 1, ALICE, b"alice"));
    let domain = &world.domain;
    assert_eq!(domain.sign_in(&LIMITS, ALICE, 1, b"alice"), Ok(SignIn::Renew { account: 5 }));
    assert_eq!(domain.sign_in(&LIMITS, BOB, 1, b"alice"), Err(Refusal::Held { provider: 1 }));
    assert_eq!(domain.sign_in(&LIMITS, BOB, 1, b"bob"), Ok(SignIn::New));
    assert_eq!(domain.sign_in(&LIMITS, BOB, 2, b"alice"), Ok(SignIn::New), "a subject is its provider's");
    assert_eq!(domain.sign_in(&LIMITS, BOB, 3, b"bob"), Err(Refusal::Provider { provider: 3 }));
    assert_eq!(domain.sign_in(&LIMITS, BOB, 1, b""), Err(Refusal::Subject));
}

#[test]
fn each_bound_is_refused_naming_it() {
    let mut world = fresh();
    world.restore(signed_in(0, 1, ALICE, b"a"));
    world.restore(signed_in(2, 1, ALICE, b"b"));
    assert_eq!(world.domain.sign_in(&LIMITS, ALICE, 1, b"c"), limit(Bound::HolderAccounts));
    assert_eq!(world.domain.sign_in(&LIMITS, BOB, 1, b"c"), Ok(SignIn::New));
    world.restore(signed_in(4, 1, BOB, b"c"));
    world.restore(vec![
        source(4, 6, 1, Kind::Key),
        Stored::Source {
            account: 4,
            number: 7,
            place: 2,
            kind: Kind::Program,
            name: Box::from(&b"claude-code"[..]),
            held: None,
        },
    ]);
    assert_eq!(world.domain.unrestored(), 0);
    assert_eq!(world.domain.sign_in(&LIMITS, BOB, 1, b"d"), limit(Bound::Accounts));
    assert_eq!(world.domain.room_for_source(&LIMITS, 4), limit(Bound::Sources));
    assert_eq!(world.domain.room_for_source(&LIMITS, 0), Ok(()));
    assert_eq!(world.domain.room_for_source(&LIMITS, 9), Err(Refusal::Unknown));
    assert_eq!(room_for_sign_in(&LIMITS, LIMITS.sign_ins), limit(Bound::SignIns));
    assert_eq!(room_for_sign_in(&LIMITS, LIMITS.sign_ins - 1), Ok(()));
    let mut world = World::with(providers(vec![provider(1, true, false)]), &[]);
    world.restore(vec![Stored::Next { number: u32::MAX - 1 }]);
    assert_eq!(world.domain.sign_in(&LIMITS, BOB, 1, b"d"), Err(Refusal::Numbers));
}

#[test]
fn a_sources_state_is_its_own() {
    let mut world = fresh();
    world.restore(vec![account(1, 1, Holder::Deployment, b"team"), source(1, 2, 0, Kind::SignIn)]);
    world.restore(vec![source(1, 3, 1, Kind::Key)]);
    let wants = world.step(Event::Want).into_iter().filter(|request| matches!(request, Request::Component(_)));
    assert_eq!(wants.collect::<Vec<_>>(), vec![want(2), want(3)], "the component serves signed-in and LLM-key sources");
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

/// The store's family as the core commits it: each save under its record's key, each erase
/// removing its key; a restart gives the rows in key order.
fn commit(store: &mut BTreeMap<Key, Stored>, requests: Vec<Request>) {
    for request in requests {
        match request {
            Request::Save { record } => drop(store.insert(record.key(), record)),
            Request::Erase { key } => drop(store.remove(&key)),
            Request::Component(_)
            | Request::Signing(_)
            | Request::Answer { .. }
            | Request::Granted { .. }
            | Request::Closed => {}
        }
    }
}

#[test]
fn each_record_restores_to_the_same_model() {
    let configured = [account(7, 1, Holder::Deployment, b""), source(7, 8, 0, Kind::SignIn)];
    let start = |store: &BTreeMap<Key, Stored>| {
        let mut world = World::with(
            providers(vec![provider(1, true, true), provider(2, true, false), provider(3, false, false)]),
            &configured,
        );
        world.wall = Wall::from_nanos(1_000_000_000_000);
        world.restore(store.values().cloned().collect());
        world
    };
    let mut store = BTreeMap::new();
    for record in [
        account(20, 1, ALICE, b"alice"),
        source(20, 21, 0, Kind::SignIn),
        source(20, 22, 1, Kind::Key),
        Stored::Next { number: 23 },
    ] {
        store.insert(record.key(), record);
    }
    let mut world = start(&store);
    let wanted = world.step(Event::Want);
    assert_eq!(
        wanted[..3],
        [
            save({
                let mut row = account(23, 3, Holder::Deployment, b"");
                if let Stored::Account { place, .. } = &mut row {
                    *place = 1;
                }
                row
            }),
            save(source(23, 24, 0, Kind::None)),
            save(Stored::Next { number: 25 })
        ]
    );
    commit(&mut store, wanted);
    let reset = Some(Time::ZERO.saturating_add(Duration::from_secs(90)));
    for event in [
        Event::Limited { account: 21, reset },
        Event::Refused { account: 22, code: Box::new([]), reason: Box::new([]) },
        Event::Limited { account: 7, reset: None },
    ] {
        let saved = world.step(event);
        assert_eq!(saved.len(), 1, "a changed hold saves its source's record");
        commit(&mut store, saved);
    }
    assert_eq!(
        store.len(),
        7,
        "the counter, two accounts and four sources: a configured account is the configuration's, its source's record its hold"
    );
    let mut again = start(&store);
    assert_eq!(again.domain.unrestored(), 0);
    assert_eq!(
        again.step(Event::Want),
        vec![
            Request::Signing(jig_domain_accounts::Signing::Restore { source: 21, provider: 1, kind: Kind::SignIn }),
            Request::Signing(jig_domain_accounts::Signing::Restore { source: 22, provider: 1, kind: Kind::Key }),
            Request::Signing(jig_domain_accounts::Signing::Restored),
            want(7),
            want(21),
            want(22)
        ],
        "a restart makes nothing and saves nothing"
    );
    assert_eq!(again.domain.next_number(), world.domain.next_number());
    assert_eq!(again.domain.source_views().as_slice(), world.domain.source_views().as_slice());
    assert_eq!(again.domain.source(21).expect("kept").state, State::Limited { until: reset });
    for number in [7, 20, 23] {
        let (was, is) = (world.domain.account(number).expect("kept"), again.domain.account(number).expect("restored"));
        assert_eq!(
            (was.provider(), was.holder(), was.subject(), was.display(), was.sources().iter().collect::<Vec<_>>()),
            (is.provider(), is.holder(), is.subject(), is.display(), is.sources().iter().collect::<Vec<_>>())
        );
    }
    let closed = world.step(Event::Close);
    assert!(!closed.iter().any(|request| matches!(request, Request::Save { .. })), "a close keeps every hold");
}

#[test]
fn a_record_breaking_a_rule_is_counted_and_dropped() {
    let records = vec![
        account(7, 1, ALICE, b"alice"),
        source(7, 8, 1, Kind::Key),
        Stored::Source {
            account: 7,
            number: 7,
            place: 0,
            kind: Kind::SignIn,
            name: Box::new([]),
            held: Some(Held::Refused),
        },
        account(9, 5, BOB, b"bob"),
        source(9, 10, 0, Kind::SignIn),
    ];
    let mut world = World::with(providers(vec![provider(1, true, true)]), &[]);
    world.restore(records.clone());
    assert_eq!(
        world.domain.unrestored(),
        3,
        "an account and source cannot share a number; an unknown provider's rows wait"
    );
    let alice = world.domain.account(7).expect("restored");
    assert_eq!((alice.holder(), alice.subject()), (ALICE, &b"alice"[..]));
    assert_eq!(alice.sources().iter().collect::<Vec<_>>(), vec![(&1, &8)]);
    assert!(world.domain.source(7).is_none());
    assert!(world.domain.account(9).is_none());
    assert_eq!(world.domain.next_number(), 11, "a record dropped still passes its numbers");
    let mut again = World::with(providers(vec![provider(1, true, true), provider(5, true, false)]), &[]);
    again.restore(records);
    assert_eq!(again.domain.unrestored(), 1);
    assert_eq!(again.domain.account(9).expect("restored").sources().len(), 1);
}
