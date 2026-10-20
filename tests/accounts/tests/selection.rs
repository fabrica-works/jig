//! Ordered account choice and the source barrier belong to accounts, below placement.
use jig_accounts_world::{World, account, provider, providers, source};
use jig_domain_accounts::{
    Coverage, Event, Excluded, Failure, Form, FromComponent, Held, Holder, HostSource, Kind, Place, Selection, State,
    Stored, Unread, Use, WaitWhy,
};
use skein_lib::{Duration, Time};

// More than eleven simulated days, far beyond the grant and retry spans.
const LONG_ADVANCE: u64 = 1_000_000;
const PROVIDER: u32 = 1;
const ALICE: u64 = 7;
const BOB: u64 = 8;
const OWN: u32 = 10;
const OTHER: u32 = 20;
const DEPLOYMENT: u32 = 30;
const FIRST: u32 = 11;
const SECOND: u32 = 12;
const OTHER_SOURCE: u32 = 21;
const DEPLOYMENT_SOURCE: u32 = 31;
const PROOFS: [Coverage; 3] = [
    Coverage { account: OWN, serves: true },
    Coverage { account: OTHER, serves: true },
    Coverage { account: DEPLOYMENT, serves: true },
];

fn fixture(rows: Vec<Stored>) -> World {
    let mut world = World::with(providers(vec![provider(PROVIDER, true, true)]), &[]);
    world.restore(rows);
    world.step(Event::Want);
    world
}
fn grant_generation(world: &mut World, source: u32, generation: u64) {
    world.step(Event::Component(FromComponent::Granted {
        form: Form::Credential(jig_domain_accounts::Scheme::Key),
        account: source,
        generation,
        valid: Duration::from_secs(60),
    }));
}
fn grant(world: &mut World, source: u32) {
    grant_generation(world, source, 1);
}
fn choose(
    world: &World,
    preferred: Option<u32>,
    excluded: Option<Excluded>,
    proofs: &[Coverage],
    host: &[HostSource],
) -> Selection {
    world.domain.select(Use::Party(ALICE), PROVIDER, preferred, excluded, proofs, host, world.now)
}
fn ranked(own_holder: Holder, own_place: u32) -> World {
    ranked_with_other(own_holder, own_place, Holder::Party(ALICE))
}
fn ranked_with_other(own_holder: Holder, own_place: u32, other_holder: Holder) -> World {
    let mut first = account(OWN, PROVIDER, own_holder, b"");
    if let Stored::Account { place, .. } = &mut first {
        *place = own_place;
    }
    let mut other = account(OTHER, PROVIDER, other_holder, b"");
    if let Stored::Account { uses, .. } = &mut other {
        *uses = Some(Box::new([Use::Party(ALICE)]));
    }
    let mut world = fixture(vec![
        first,
        source(OWN, FIRST, 0, Kind::Key),
        other,
        source(OTHER, OTHER_SOURCE, 0, Kind::Key),
        account(DEPLOYMENT, PROVIDER, Holder::Deployment, b""),
        source(DEPLOYMENT, DEPLOYMENT_SOURCE, 0, Kind::Key),
    ]);
    for source in [FIRST, OTHER_SOURCE, DEPLOYMENT_SOURCE] {
        grant(&mut world, source);
    }
    world
}

#[test]
fn holder_order_precedes_explicit_uses_and_deployment_fallback() {
    let world = ranked(Holder::Party(ALICE), 1);
    assert_eq!(choose(&world, None, None, &PROOFS, &[]), Selection::Chosen { account: OTHER, source: OTHER_SOURCE });
    let world = ranked(Holder::Party(BOB), 0);
    assert_eq!(choose(&world, None, None, &PROOFS, &[]), Selection::Chosen { account: OTHER, source: OTHER_SOURCE });
    let narrowed = [Coverage { account: OWN, serves: false }, Coverage { account: OTHER, serves: false }, PROOFS[2]];
    assert_eq!(
        choose(&world, None, None, &narrowed, &[]),
        Selection::Chosen { account: DEPLOYMENT, source: DEPLOYMENT_SOURCE }
    );
}

#[test]
fn a_preferred_account_waits_while_fresh_selection_can_continue() {
    let mut world = ranked(Holder::Party(ALICE), 0);
    world.step(Event::Limited { account: FIRST, reset: None });
    assert_eq!(
        choose(&world, Some(OWN), None, &PROOFS, &[]),
        Selection::Wait { account: OWN, source: Some(FIRST), why: WaitWhy::Source(State::Limited { until: None }) }
    );
    assert_eq!(choose(&world, None, None, &PROOFS, &[]), Selection::Chosen { account: OTHER, source: OTHER_SOURCE });
    let excluded = Some(Excluded { holder: Holder::Party(ALICE), provider: PROVIDER });
    assert_eq!(
        choose(&world, Some(OWN), excluded, &PROOFS, &[]),
        Selection::Wait { account: OWN, source: Some(FIRST), why: WaitWhy::Source(State::Limited { until: None }) }
    );
    assert_eq!(
        choose(&world, None, excluded, &PROOFS, &[]),
        Selection::Chosen { account: DEPLOYMENT, source: DEPLOYMENT_SOURCE }
    );
    let ineligible = [Coverage { account: OWN, serves: false }, PROOFS[1], PROOFS[2]];
    assert_eq!(
        choose(&world, Some(OWN), None, &ineligible, &[]),
        Selection::Chosen { account: OTHER, source: OTHER_SOURCE }
    );
    assert_eq!(
        world.domain.select(Use::Party(ALICE), 99, Some(OWN), None, &PROOFS, &[], world.now),
        Selection::NoAccount
    );
}

fn two_sources(kind: Kind, held: Option<Held>) -> World {
    let mut earlier = source(OWN, FIRST, 0, kind);
    if let Stored::Source { name, held: state, .. } = &mut earlier {
        *state = held;
        if kind == Kind::Program {
            *name = b"runner".as_slice().into();
        }
    }
    let mut rows = vec![account(OWN, PROVIDER, Holder::Party(ALICE), b""), earlier, source(OWN, SECOND, 1, Kind::Key)];
    if kind == Kind::Program {
        rows.push(Stored::Issued {
            number: FIRST,
            holder: Holder::Party(ALICE),
            name: b"runner".as_slice().into(),
            signed_out: false,
        });
    }
    let mut world = fixture(rows);
    grant(&mut world, SECOND);
    world
}

#[test]
fn global_source_holds_and_refreshes_block_later_llm_keys() {
    for (held, state) in
        [(Some(Held::Limited { reset: None }), State::Limited { until: None }), (Some(Held::Refused), State::Refused)]
    {
        let world = two_sources(Kind::Program, held);
        assert_eq!(
            choose(&world, Some(OWN), None, &PROOFS, &[]),
            Selection::Wait { account: OWN, source: Some(FIRST), why: WaitWhy::Source(state) }
        );
    }
    for (failure, state) in [
        (Failure::SignInAgain, State::NeedsSignIn),
        (Failure::Misconfigured, State::Misconfigured),
        (
            Failure::Unloaded { at: Place::File, why: Unread::Unreadable },
            State::Unloaded { at: Place::File, why: Unread::Unreadable },
        ),
    ] {
        let mut world = two_sources(Kind::SignIn, None);
        world.step(Event::Component(FromComponent::Failed { account: FIRST, failure }));
        assert_eq!(
            choose(&world, Some(OWN), None, &PROOFS, &[]),
            Selection::Wait { account: OWN, source: Some(FIRST), why: WaitWhy::Source(state) }
        );
    }
    let world = two_sources(Kind::SignIn, None);
    assert_eq!(
        choose(&world, Some(OWN), None, &PROOFS, &[]),
        Selection::Wait { account: OWN, source: Some(FIRST), why: WaitWhy::Refreshing }
    );
}

#[test]
fn only_host_local_program_unavailability_passes_to_the_next_source() {
    let world = two_sources(Kind::Program, None);
    assert_eq!(choose(&world, Some(OWN), None, &PROOFS, &[]), Selection::Chosen { account: OWN, source: SECOND });
    assert_eq!(
        choose(&world, Some(OWN), None, &PROOFS, &[HostSource { source: FIRST, usable: true }]),
        Selection::Chosen { account: OWN, source: FIRST }
    );
}

#[test]
fn a_none_source_is_chosen_without_a_grant() {
    // These are the ordinary counter's first account and source, rather than sentinels.
    const NONE_ACCOUNT: u32 = 0;
    const NONE_SOURCE: u32 = 1;
    let mut world = World::with(providers(vec![provider(PROVIDER, false, false)]), &[]);
    world.step(Event::Want);
    let proof = [Coverage { account: NONE_ACCOUNT, serves: true }];
    assert_eq!(
        choose(&world, None, None, &proof, &[]),
        Selection::Chosen { account: NONE_ACCOUNT, source: NONE_SOURCE }
    );
    assert_eq!(world.domain.source_grant(NONE_SOURCE, Time::ZERO), None);
}

#[test]
fn unknown_reset_waits_across_time_grants_and_restart_until_resume() {
    let rows = |held| {
        vec![
            account(OWN, PROVIDER, Holder::Party(ALICE), b""),
            Stored::Source { account: OWN, number: FIRST, place: 0, kind: Kind::Key, name: Box::new([]), held },
        ]
    };
    let mut world = fixture(rows(None));
    grant(&mut world, FIRST);
    world.step(Event::Limited { account: FIRST, reset: None });
    world.fire_at(LONG_ADVANCE);
    grant_generation(&mut world, FIRST, 2);
    let wait =
        Selection::Wait { account: OWN, source: Some(FIRST), why: WaitWhy::Source(State::Limited { until: None }) };
    assert_eq!(
        (world.domain.source(FIRST).expect("kept source").generation, choose(&world, Some(OWN), None, &PROOFS, &[])),
        (Some(2), wait)
    );
    let mut world = fixture(rows(Some(Held::Limited { reset: None })));
    grant(&mut world, FIRST);
    assert_eq!(choose(&world, Some(OWN), None, &PROOFS, &[]), wait);
    world.step(Event::Change {
        request: skein_lib::Token::new(1),
        holder: Holder::Party(ALICE),
        change: jig_domain_accounts::Change::Resume { source: FIRST },
    });
    assert_eq!(choose(&world, Some(OWN), None, &PROOFS, &[]), Selection::Chosen { account: OWN, source: FIRST });
}

#[test]
fn explicit_zero_reset_lifts_but_later_notices_never_shorten_a_limit() {
    let mut world = ranked(Holder::Party(ALICE), 0);
    world.step(Event::Limited { account: FIRST, reset: Some(Time::ZERO) });
    world.fire_at(0);
    assert_eq!(choose(&world, Some(OWN), None, &PROOFS, &[]), Selection::Chosen { account: OWN, source: FIRST });
    let reset = Time::ZERO.saturating_add(Duration::from_secs(50));
    world.step(Event::Limited { account: FIRST, reset: Some(reset) });
    world.step(Event::Limited { account: FIRST, reset: Some(Time::ZERO.saturating_add(Duration::from_secs(10))) });
    world.fire_at(10);
    assert_eq!(
        choose(&world, Some(OWN), None, &PROOFS, &[]),
        Selection::Wait {
            account: OWN,
            source: Some(FIRST),
            why: WaitWhy::Source(State::Limited { until: Some(reset) })
        }
    );
}

#[test]
fn selection_evidence_is_bounded_and_duplicate_evidence_is_refused() {
    let world = ranked(Holder::Party(ALICE), 0);
    let over = [PROOFS[0], PROOFS[1], PROOFS[2], Coverage { account: 99, serves: true }];
    assert_eq!(choose(&world, None, None, &over, &[]), Selection::NoAccount);
    assert_eq!(choose(&world, None, None, &[PROOFS[0], PROOFS[0]], &[]), Selection::NoAccount);
    assert_eq!(
        choose(
            &world,
            None,
            None,
            &PROOFS,
            &[HostSource { source: FIRST, usable: false }, HostSource { source: FIRST, usable: true }]
        ),
        Selection::NoAccount
    );
}

#[test]
fn a_removed_preference_allows_fresh_account_choice() {
    let mut world = ranked(Holder::Party(ALICE), 0);
    world.step(Event::Change {
        request: skein_lib::Token::new(1),
        holder: Holder::Party(ALICE),
        change: jig_domain_accounts::Change::Remove { account: OWN },
    });
    assert_eq!(
        choose(&world, Some(OWN), None, &PROOFS, &[]),
        Selection::Chosen { account: OTHER, source: OTHER_SOURCE }
    );
}

#[test]
fn a_changed_main_provider_allows_fresh_account_choice() {
    const NEW_PROVIDER: u32 = 2;
    let mut world =
        World::with(providers(vec![provider(PROVIDER, false, true), provider(NEW_PROVIDER, false, true)]), &[]);
    world.restore(vec![
        account(OWN, PROVIDER, Holder::Party(ALICE), b""),
        source(OWN, FIRST, 0, Kind::Key),
        account(DEPLOYMENT, NEW_PROVIDER, Holder::Deployment, b""),
        source(DEPLOYMENT, DEPLOYMENT_SOURCE, 0, Kind::Key),
    ]);
    world.step(Event::Want);
    grant(&mut world, FIRST);
    grant(&mut world, DEPLOYMENT_SOURCE);
    assert_eq!(
        world.domain.select(Use::Party(ALICE), NEW_PROVIDER, Some(OWN), None, &PROOFS, &[], world.now),
        Selection::Chosen { account: DEPLOYMENT, source: DEPLOYMENT_SOURCE }
    );
}

#[test]
fn host_rules_filter_direct_accounts_before_fresh_ranking() {
    let world = ranked_with_other(Holder::Party(ALICE), 0, Holder::Party(BOB));
    let host = [
        HostSource { source: FIRST, usable: false },
        HostSource { source: OTHER_SOURCE, usable: true },
        HostSource { source: DEPLOYMENT_SOURCE, usable: false },
    ];
    assert_eq!(choose(&world, None, None, &PROOFS, &host), Selection::Chosen { account: OTHER, source: OTHER_SOURCE });
    assert_eq!(
        choose(&world, Some(OWN), None, &PROOFS, &host),
        Selection::Wait { account: OWN, source: Some(FIRST), why: WaitWhy::Host }
    );
    let mut world = two_sources(Kind::SignIn, None);
    grant(&mut world, FIRST);
    assert_eq!(
        choose(&world, Some(OWN), None, &PROOFS, &host),
        Selection::Wait { account: OWN, source: Some(FIRST), why: WaitWhy::Host }
    );
}
