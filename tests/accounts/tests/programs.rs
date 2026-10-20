//! Declarations reserve a durable name before any host operation is possible.
use jig_accounts_world::{LIMITS, World, account, provider, providers, source};
use jig_domain_accounts::{Event, Holder, Kind, Refusal, Request, Stored, programs};
use skein_lib::{Env, Queue};
const HOLDER: Holder = Holder::Party(7);
const OTHER: Holder = Holder::Party(8);
const PROVIDER: u32 = 1;
fn configured() -> jig_domain_accounts::Providers {
    let mut offered = provider(PROVIDER, true, true);
    offered.offers.programs = Box::new([b"claude_code".as_slice().into()]);
    providers(vec![offered])
}
fn fresh() -> World {
    let mut world = World::with(configured(), &[]);
    world.step(Event::Want);
    world
}
fn declare(
    world: &mut World,
    holder: Holder,
    provider: u32,
    join: Option<u32>,
    name: &[u8],
) -> (Result<u32, programs::DeclarationRefusal>, Vec<Request>) {
    let mut out = Queue::with_capacity(jig_domain_accounts::max_out(&LIMITS));
    let env = Env { now: world.now, wall: world.wall, limits: LIMITS };
    let result = programs::declare(&mut world.domain, &env, holder, provider, join, name.into(), &mut out);
    let mut requests = Vec::new();
    while let Some(request) = out.pop() {
        requests.push(request);
    }
    (result, requests)
}
fn issued(number: u32, name: &[u8]) -> Stored {
    Stored::Issued { number, holder: HOLDER, name: name.into() }
}
#[test]
fn a_declaration_saves_its_reservation_atomically() {
    let mut world = fresh();
    assert_eq!(
        declare(&mut world, HOLDER, PROVIDER, None, b"personal"),
        (
            Ok(1),
            vec![
                Request::Save { record: account(0, PROVIDER, HOLDER, b"") },
                Request::Save { record: issued(1, b"personal") },
                Request::Save {
                    record: Stored::Source {
                        account: 0,
                        number: 1,
                        place: 0,
                        kind: Kind::Program,
                        name: b"personal".as_slice().into(),
                        held: None
                    }
                },
                Request::Save { record: Stored::Next { number: 2 } },
            ]
        )
    );
    let account = world.domain.account(0).expect("kept account");
    assert!(account.plan());
    assert!(account.subject().is_empty());
}
#[test]
fn a_program_joins_at_the_last_source_place() {
    let mut world = World::with(configured(), &[account(20, PROVIDER, HOLDER, b""), source(20, 21, 0, Kind::Key)]);
    world.step(Event::Want);
    assert_eq!(declare(&mut world, HOLDER, PROVIDER, Some(20), b"personal").0, Ok(22));
    assert_eq!(
        world.domain.account(20).expect("kept account").sources().iter().map(|(p, s)| (*p, *s)).collect::<Vec<_>>(),
        vec![(0, 21), (1, 22)]
    );
}
#[test]
fn a_foreign_holder_cannot_join_an_account() {
    let mut world = World::with(configured(), &[account(20, PROVIDER, OTHER, b""), source(20, 21, 0, Kind::Key)]);
    world.step(Event::Want);
    assert_eq!(
        declare(&mut world, HOLDER, PROVIDER, Some(20), b"personal"),
        (Err(programs::DeclarationRefusal::Account(Refusal::Unknown)), vec![])
    );
    assert_eq!(world.domain.next_number(), 22);
}
#[test]
fn a_provider_must_offer_a_program() {
    let mut world = World::with(providers(vec![provider(PROVIDER, true, true)]), &[]);
    world.step(Event::Want);
    assert_eq!(
        declare(&mut world, HOLDER, PROVIDER, None, b"personal"),
        (Err(programs::DeclarationRefusal::Account(Refusal::Provider { provider: PROVIDER })), vec![])
    );
}
#[test]
fn invalid_names_allocate_nothing() {
    for name in [b"".as_slice(), b"Upper", b"a_b", &[b'a'; 33]] {
        let mut world = fresh();
        assert_eq!(
            declare(&mut world, HOLDER, PROVIDER, None, name),
            (Err(programs::DeclarationRefusal::Name), vec![])
        );
        assert_eq!(world.domain.next_number(), 0);
    }
}
#[test]
fn names_accept_the_exact_length_bound() {
    let mut world = fresh();
    assert_eq!(declare(&mut world, HOLDER, PROVIDER, None, &[b'a'; 32]).0, Ok(1));
}
#[test]
fn a_live_name_cannot_be_declared_twice() {
    let mut world = fresh();
    assert_eq!(declare(&mut world, HOLDER, PROVIDER, None, b"personal").0, Ok(1));
    assert_eq!(
        declare(&mut world, OTHER, PROVIDER, None, b"personal"),
        (Err(programs::DeclarationRefusal::Issued), vec![])
    );
    assert_eq!(world.domain.next_number(), 2);
}
#[test]
fn retired_names_survive_restart() {
    let mut world = World::with(configured(), &[issued(50_000, b"retired")]);
    world.step(Event::Want);
    assert_eq!(
        declare(&mut world, HOLDER, PROVIDER, None, b"retired"),
        (Err(programs::DeclarationRefusal::Issued), vec![])
    );
    assert_eq!(declare(&mut world, OTHER, PROVIDER, None, b"fresh").0, Ok(50_002));
}
#[test]
fn the_lifetime_name_bound_refuses_before_allocating() {
    let rows = (0..LIMITS.program_names).map(|n| issued(n, format!("old-{n}").as_bytes())).collect::<Vec<_>>();
    let mut world = World::with(configured(), &rows);
    world.step(Event::Want);
    assert_eq!(
        declare(&mut world, HOLDER, PROVIDER, None, b"fresh"),
        (Err(programs::DeclarationRefusal::Names), vec![])
    );
    assert_eq!(world.domain.accounts(), 0);
    assert_eq!(world.domain.next_number(), LIMITS.program_names);
}
#[test]
fn a_restored_source_must_match_its_reserved_holder() {
    let mut program = source(20, 21, 0, Kind::Program);
    if let Stored::Source { name, .. } = &mut program {
        *name = b"personal".as_slice().into();
    }
    let mut world = World::with(configured(), &[account(20, PROVIDER, OTHER, b""), program, issued(21, b"personal")]);
    world.step(Event::Want);
    assert!(world.domain.source(21).is_none());
    assert_eq!(world.domain.unrestored(), 2);
    assert_eq!(declare(&mut world, HOLDER, PROVIDER, None, b"personal").0, Err(programs::DeclarationRefusal::Issued));
}

#[test]
fn restoring_a_program_reserves_its_existing_directory_name() {
    let mut program = source(20, 21, 0, Kind::Program);
    if let Stored::Source { name, .. } = &mut program {
        *name = b"personal".as_slice().into();
    }
    let mut world = World::with(configured(), &[account(20, PROVIDER, HOLDER, b""), program]);
    assert!(world.step(Event::Want).contains(&Request::Save { record: issued(21, b"personal") }));
    assert!(world.domain.source(21).is_some());
    assert_eq!(
        declare(&mut world, OTHER, PROVIDER, None, b"personal"),
        (Err(programs::DeclarationRefusal::Issued), vec![])
    );
}
