//! Readiness consumes child selection; its source/host matrix stays in child worlds.
use super::*;
use documents::{PayerReadiness, Readiness};

fn serving() -> (Core, Env<Limits>) {
    serving_records(own_records())
}
fn serving_records(records: Vec<accounts::Stored>) -> (Core, Env<Limits>) {
    hosted_records(records, true)
}
fn hosted_records(records: Vec<accounts::Stored>, granted: bool) -> (Core, Env<Limits>) {
    let (mut core, env) = fixture(records);
    core.settings.models[0].name = b"scripted".as_slice().into();
    core.settings.llm_providers.providers[0].models = Box::new([b"scripted".as_slice().into()]);
    if granted {
        grant(&mut core, &env, FIRST_SOURCE);
    }
    let mut out = Queue::with_capacity(fleet::max_out(&env.limits.fleet));
    let host_env = Env { now: env.now, wall: env.wall, limits: env.limits.fleet };
    fleet::step(
        &mut core.fleet,
        &host_env,
        fleet::Event::Hello {
            worker: 0,
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
    let logins = core
        .fleet
        .program_names()
        .iter()
        .map(|name| fleet::Login { name: *name, state: fleet::LoginState::NotSignedIn })
        .collect();
    fleet::step(&mut core.fleet, &host_env, fleet::Event::Logins { channel: Token::new(77), logins }, &mut out);
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
fn program_offers_without_an_account_are_unavailable() {
    let (mut core, env) = fixture(vec![]);
    core.settings.llm_providers.providers[0].offers.programs = Box::new([b"arbitrary".as_slice().into()]);
    assert_eq!(own(&core, &env, ALICE).readiness, Readiness::Unavailable);
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
    assert_eq!(offer.key_bytes, frame.key_bytes);
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

#[test]
fn readiness_keeps_providers_separate() {
    let (mut core, env) = serving();
    let mut other = core.settings.llm_providers.providers[0].clone();
    other.number = 4;
    core.settings.llm_providers.providers = Box::new([core.settings.llm_providers.providers[0].clone(), other]);
    let doc = document(&core, &env, ALICE);
    assert_eq!(doc.readiness.get(0).unwrap().readiness, Readiness::Usable);
    assert_eq!(doc.readiness.get(1).unwrap().readiness, Readiness::Unavailable);
}

#[test]
fn a_pending_inventory_keeps_readiness_unknown() {
    let (mut core, env) = serving();
    drop(step(
        &mut core,
        &env,
        Event::Fleet(fleet::Event::Sources {
            sources: Box::new([fleet::SourceName::new(b"program").expect("bounded fixture name")]),
        }),
    ));
    assert_eq!(own(&core, &env, ALICE).readiness, Readiness::Unknown);
}

#[test]
fn a_limited_candidate_does_not_hide_a_program_host_wait() {
    let mut records = own_records();
    records.extend([
        account(23, accounts::Holder::Party(ALICE)),
        accounts::Stored::Source {
            account: 23,
            number: 18,
            place: 0,
            kind: accounts::Kind::Program,
            name: b"program".as_slice().into(),
            held: None,
        },
    ]);
    let (mut core, env) = serving_records(records);
    core.settings.models[0].program = Some(b"claude_code".as_slice().into());
    drop(step(&mut core, &env, Event::Account(accounts::Event::Limited { account: FIRST_SOURCE, reset: None })));
    let coverage = core.account_coverage(tasks::Payer::Party(ALICE));
    assert_eq!(
        core.accounts.candidates(Use::Party(ALICE), PROVIDER, None, None, None, coverage.as_slice()).get(0),
        Some(&OWN)
    );
    assert_eq!(own(&core, &env, ALICE).readiness, Readiness::Unknown);
}
#[test]
fn a_restored_source_awaiting_its_grant_reads_unknown() {
    let (core, env) = hosted_records(own_records(), false);
    assert_eq!(own(&core, &env, ALICE).readiness, Readiness::Unknown);
}
#[test]
fn a_restored_source_awaiting_its_grant_is_unsettled() {
    let (core, env) = hosted_records(own_records(), false);
    assert_eq!(own(&core, &env, ALICE).settled, Some(false));
}
#[test]
fn a_pending_personal_sign_in_leaves_project_settlement_alone() {
    let (mut core, env) = fixture(vec![]);
    ask(&mut core, &env, ALICE, 1, people::AccountHolder::Own);
    assert_eq!(document(&core, &env, ALICE).readiness.get(1).unwrap().settled, Some(true));
}

fn catalogue_host(holder: accounts::Holder, program_first: bool) -> (Core, Env<Limits>) {
    let (mut core, limits) = run_choice::placement_fixture(accounts::Holder::Party(ALICE), holder, program_first);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    change_account(
        &mut core,
        &env,
        accounts::Holder::Party(ALICE),
        accounts::Change::SetUses { account: 0, uses: Some(Box::new([Use::Party(ALICE)])) },
    );
    (core, env)
}
#[test]
fn readiness_ignores_another_provider_s_logins() {
    let (core, env) = catalogue_host(accounts::Holder::Deployment, false);
    assert_eq!(own(&core, &env, ALICE).readiness, Readiness::Usable);
}
#[test]
fn readiness_reads_a_usable_program_login() {
    let (mut core, env) = catalogue_host(accounts::Holder::Party(ALICE), true);
    drop(step(&mut core, &env, Event::Account(accounts::Event::Limited { account: 1, reset: None })));
    assert_eq!(own(&core, &env, ALICE).readiness, Readiness::Usable);
}

#[test]
fn readiness_requires_one_models_source_and_host_together() {
    let (mut core, env) = serving();
    let mut direct = core.settings.models[0].clone();
    direct.name = b"direct".as_slice().into();
    core.settings.models[0].program = Some(b"claude_code".as_slice().into());
    core.settings.models = Box::new([core.settings.models[0].clone(), direct]);
    core.settings.llm_providers.providers[0].models =
        Box::new([b"scripted".as_slice().into(), b"direct".as_slice().into()]);
    core.settings.llm_providers.providers[0].offers.programs = Box::new([b"claude_code".as_slice().into()]);
    core.fleet = run_choice::placement_fleet(&core, &env.limits);
    run_choice::host(&mut core, &env.limits, Token::new(7), &[b"scripted"]);
    // The program model has a host but no source; the direct model has a source but no host.
    assert_eq!(own(&core, &env, ALICE).readiness, Readiness::Unavailable);
    drop(step(&mut core, &env, Event::Fleet(fleet::Event::Lost { channel: Token::new(7) })));
    core.fleet.reclaim();
    run_choice::host(&mut core, &env.limits, Token::new(11), &[b"scripted", b"direct"]);
    assert_eq!(own(&core, &env, ALICE).readiness, Readiness::Usable);
    drop(step(&mut core, &env, Event::Account(accounts::Event::Limited { account: FIRST_SOURCE, reset: None })));
    assert_eq!(own(&core, &env, ALICE).readiness, Readiness::Unavailable);
}
