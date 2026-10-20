//! Readiness consumes child selection; its source/host matrix stays in child worlds.
use super::*;
use documents::{PayerReadiness, Readiness};

fn serving() -> (Core, Env<Limits>) {
    serving_records(own_records())
}
fn serving_records(records: Vec<accounts::Stored>) -> (Core, Env<Limits>) {
    let (mut core, env) = fixture(records);
    core.settings.models[0].name = b"scripted".as_slice().into();
    core.settings.llm_providers.providers[0].models = Box::new([b"scripted".as_slice().into()]);
    grant(&mut core, &env, FIRST_SOURCE);
    let mut out = Queue::with_capacity(fleet::max_out(&env.limits.fleet));
    let host_env = Env { now: env.now, wall: env.wall, limits: env.limits.fleet };
    fleet::step(
        &mut core.fleet,
        &host_env,
        fleet::Event::Hello {
            channel: Token::new(77),
            hello: fleet::Hello {
                bounds: fixtures::bounds(),
                stop_bound: Duration::ZERO,
                slots: 1,
                workstreams: Box::new([]),
                hosting: Box::new([]),
            },
        },
        &mut out,
    );
    fleet::step(
        &mut core.fleet,
        &host_env,
        fleet::Event::Logins { channel: Token::new(77), logins: Box::new([]) },
        &mut out,
    );
    (core, env)
}
fn own(core: &Core, env: &Env<Limits>, party: u64) -> PayerReadiness {
    *document(core, env, party).readiness.get(0).expect("own payer first")
}
#[test]
fn readiness_uses_the_current_payer() {
    let (core, env) = serving();
    assert_eq!(own(&core, &env, ALICE).readiness, Readiness::Usable);
    assert_eq!(own(&core, &env, BOB).readiness, Readiness::Unavailable);
}
#[test]
fn readiness_excludes_explicit_empty_uses() {
    let (mut core, env) = serving();
    change_account(
        &mut core,
        &env,
        accounts::Holder::Party(ALICE),
        accounts::Change::SetUses { account: OWN, uses: Some(Box::new([])) },
    );
    assert_eq!(own(&core, &env, ALICE).readiness, Readiness::Unavailable);
}
#[test]
fn readiness_keeps_an_unknown_reset_unavailable() {
    let (mut core, env) = serving();
    drop(step(&mut core, &env, Event::Account(accounts::Event::Limited { account: FIRST_SOURCE, reset: None })));
    assert_eq!(own(&core, &env, ALICE).readiness, Readiness::Unavailable);
}
#[test]
fn readiness_survives_cut_account_rows() {
    let mut records = vec![account(19, accounts::Holder::Party(ALICE)), source(19, 18, 0, accounts::Kind::Key)];
    records.extend(own_records());
    let (core, mut env) = serving_records(records);
    env.limits.documents.lines = 1;
    let doc = document(&core, &env, ALICE);
    assert!(doc.cut);
    assert_eq!(doc.accounts.get(0).unwrap().number, 19);
    assert_eq!(doc.readiness.get(0).unwrap().readiness, Readiness::Usable);
}
#[test]
fn readiness_excludes_a_disconnected_host() {
    let (mut core, env) = serving();
    drop(step(&mut core, &env, Event::Fleet(fleet::Event::Lost { channel: Token::new(77) })));
    assert_eq!(own(&core, &env, ALICE).readiness, Readiness::Unavailable);
}
#[test]
fn programs_keep_readiness_explicitly_unknown() {
    let (mut core, env) = fixture(vec![]);
    core.settings.llm_providers.providers[0].offers.programs = Box::new([b"arbitrary".as_slice().into()]);
    assert_eq!(own(&core, &env, ALICE).readiness, Readiness::Unknown);
}
#[test]
fn program_settlement_is_unknown_before_check_facts() {
    let (mut core, env) = fixture(vec![]);
    core.settings.llm_providers.providers[0].offers.programs = Box::new([b"arbitrary".as_slice().into()]);
    assert_eq!(own(&core, &env, ALICE).settled, None);
}
#[test]
fn pending_sign_in_unsettles_only_its_provider() {
    let (mut core, env) = fixture(vec![]);
    let mut other = core.settings.llm_providers.providers[0].clone();
    other.number = 4;
    core.settings.llm_providers.providers = Box::new([core.settings.llm_providers.providers[0].clone(), other]);
    ask(&mut core, &env, ALICE, 1, people::AccountHolder::Own);
    let doc = document(&core, &env, ALICE);
    assert_eq!(doc.readiness.get(0).unwrap().settled, Some(false));
    assert_eq!(doc.readiness.get(1).unwrap().settled, Some(true));
}
#[test]
fn catalogue_keeps_public_offers_for_an_ordinary_party() {
    let (mut core, env) = fixture(vec![]);
    core.settings.key_bytes = 123;
    core.settings.llm_providers.providers[0].offers.programs = Box::new([b"arbitrary".as_slice().into()]);
    let Document::Frame(frame) =
        make(&core, &env.limits, BOB, &Subject::Frame { party: BOB }, &window(), env.wall).unwrap()
    else {
        panic!("frame")
    };
    assert!(frame.deployment_role.is_none());
    let offer = frame.providers.get(0).unwrap();
    assert_eq!(offer.provider, PROVIDER);
    assert_eq!(offer.name.bytes.as_ref(), b"Fake");
    assert!(offer.sign_in && offer.key && offer.signed_in_plan && !offer.none);
    assert_eq!(offer.programs.get(0).unwrap().as_ref(), b"arbitrary");
    assert_eq!(offer.manage_usage.as_deref(), Some(b"https://usage.example/".as_slice()));
    assert_eq!(frame.key_bytes, 123);
}

#[test]
fn readiness_rechecks_a_projects_current_members() {
    let (mut core, env) = serving();
    let project = |core: &Core| document(core, &env, ALICE).readiness.get(1).unwrap().readiness;
    assert_eq!(project(&core), Readiness::Usable);
    drop(step(
        &mut core,
        &env,
        Event::People(people::Event::Roles {
            project: 0,
            holdings: Box::new([
                people::Holding { person: ALICE, role: people::Role::Owner },
                people::Holding { person: BOB, role: people::Role::Member },
            ]),
        }),
    ));
    assert_eq!(project(&core), Readiness::Unavailable);
}
