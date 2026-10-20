//! Configured terms classify sources; records hold no duplicate terms facts.
use jig_accounts_world::{World, account, provider, providers, source};
use jig_domain_accounts::{End, Event, Holder, Kind, Request, Signing, Stored};
use skein_lib::Token;
const PROVIDER: u32 = 1;
const PARTY: u64 = 7;
const REQUEST: Token = Token::new(7);
const HOLDER: Holder = Holder::Party(PARTY);
const ACCOUNT: u32 = 20;
const SIGNED: u32 = 21;
const KEY: u32 = 22;
const PROGRAM: u32 = 23;
fn world(plan: bool, records: Vec<Stored>) -> World {
    let mut configured = provider(PROVIDER, true, true);
    configured.signed_in_plan = plan;
    let mut world = World::with(providers(vec![configured]), &[]);
    world.restore(records);
    world.step(Event::Want);
    world
}
fn mixed(program: bool) -> Vec<Stored> {
    let mut records = vec![
        account(ACCOUNT, PROVIDER, HOLDER, b"subject"),
        source(ACCOUNT, SIGNED, 0, Kind::SignIn),
        source(ACCOUNT, KEY, 1, Kind::Key),
    ];
    if program {
        records.push(Stored::Source {
            account: ACCOUNT,
            number: PROGRAM,
            place: 2,
            kind: Kind::Program,
            name: b"helper".as_slice().into(),
            held: None,
        });
    }
    records
}
fn begin(world: &mut World) -> u32 {
    let requests = world.step(Event::SignIn { request: REQUEST, holder: HOLDER, by: PARTY, provider: PROVIDER });
    requests
        .into_iter()
        .find_map(
            |request| if let Request::Signing(Signing::SignIn { source, .. }) = request { Some(source) } else { None },
        )
        .expect("admitted sign-in")
}
fn finish(world: &mut World, source: u32) {
    world.step(Event::SignedIn {
        source,
        subject: b"subject".as_slice().into(),
        display: Box::new([]),
        declined: false,
    });
}
fn sign_in(world: &mut World) -> u32 {
    let source = begin(world);
    finish(world, source);
    let End::Made { account, .. } = world.domain.last_end(PARTY).expect("sign-in ended").end else {
        panic!("made account")
    };
    account
}
#[test]
fn signed_in_accounts_follow_the_configured_plan_flag() {
    for plan in [false, true] {
        let mut world = world(plan, vec![]);
        let number = sign_in(&mut world);
        assert_eq!(world.domain.account(number).expect("made").plan(), plan);
    }
}
#[test]
fn restoring_mixed_sources_recomputes_plan_from_configuration() {
    for (flag, program, expected) in
        [(false, false, false), (true, false, true), (false, true, true), (true, true, true)]
    {
        let world = world(flag, mixed(program));
        assert_eq!(world.domain.account(ACCOUNT).expect("restored").plan(), expected);
    }
}
#[test]
fn removing_a_signed_in_source_leaves_a_key_account() {
    for plan in [false, true] {
        let mut world = world(plan, mixed(false));
        world.step(Event::SignOut { request: REQUEST, holder: HOLDER, source: SIGNED });
        assert!(!world.domain.account(ACCOUNT).expect("key remains").plan());
    }
}
#[test]
fn removing_a_signed_in_source_retains_a_program_plan() {
    let mut world = world(false, mixed(true));
    world.step(Event::SignOut { request: REQUEST, holder: HOLDER, source: SIGNED });
    assert!(world.domain.account(ACCOUNT).expect("program and key remain").plan());
}

#[test]
fn renewing_a_mixed_account_uses_the_configured_signed_in_terms() {
    for (flag, program, expected) in [(false, false, false), (true, false, true), (false, true, true)] {
        let mut world = world(flag, mixed(program));
        let source = begin(&mut world);
        finish(&mut world, source);
        assert_eq!(world.domain.account(ACCOUNT).expect("renewed").plan(), expected);
    }
}

#[test]
fn a_signed_in_source_joins_a_retained_key_account_under_its_terms() {
    for flag in [false, true] {
        let records = vec![account(ACCOUNT, PROVIDER, HOLDER, b"subject"), source(ACCOUNT, KEY, 0, Kind::Key)];
        let mut world = world(flag, records);
        let source = begin(&mut world);
        finish(&mut world, source);
        assert_eq!(world.domain.account(ACCOUNT).expect("joined account").plan(), flag);
        assert_eq!(world.domain.source(KEY).expect("key retained").kind, Kind::Key);
    }
}
