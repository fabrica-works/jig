//! Pure people/accounts joins; no run or core-world routing is needed.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "test tables are ordinary Rust"
)]
use super::accounts_document::{account, fixture, source};
use super::*;
const ALICE: u64 = 1;
const BOB: u64 = 2;
const SERVICE: u64 = 3;
const ACCOUNT: u32 = 20;
const SOURCE: u32 = 21;
const PROJECT: u32 = 0;

fn prepared(plan: bool, holder: accounts::Holder, uses: Option<Vec<Use>>) -> (Core, Env<Limits>) {
    let mut row = account(ACCOUNT, holder);
    if let accounts::Stored::Account { uses: kept, .. } = &mut row {
        *kept = uses.map(Vec::into_boxed_slice);
    }
    let (core, env) =
        fixture(vec![row, source(ACCOUNT, SOURCE, 0, if plan { accounts::Kind::SignIn } else { accounts::Kind::Key })]);
    (core, env)
}
fn serves(core: &Core, payer: Use) -> bool {
    crate::coverage::serves(&core.people, core.accounts.account(ACCOUNT).unwrap(), payer)
}
fn roster(core: &mut Core, env: &Env<Limits>, members: &[u64]) {
    let mut limits = env.limits.people;
    // Holder, another person and one service; the shared roster contains all three.
    limits.people = 3;
    limits.holdings = 3;
    core.people = people::Domain::new(&limits, Box::new([]), core.settings.deployment_provider);
    let people_env = Env { now: env.now, wall: env.wall, limits };
    let mut out = Queue::with_capacity(people::max_out(&limits));
    for party in [ALICE, BOB, SERVICE] {
        let provider = if party == SERVICE { core.settings.deployment_provider } else { 0 };
        let identity = people::Identity {
            key: people::IdentityKey { provider, subject: party.to_be_bytes().into() },
            login: Box::new([]),
            name: Box::new([]),
        };
        people::step(
            &mut core.people,
            &people_env,
            people::Event::Restore { record: people::Stored::Person { number: party, identity } },
            &mut out,
        );
    }
    let holdings =
        members.iter().map(|person| people::Holding { person: *person, role: people::Role::Owner }).collect();
    people::step(
        &mut core.people,
        &people_env,
        people::Event::Restore { record: people::Stored::Roles { project: PROJECT, holdings } },
        &mut out,
    );
    people::step(&mut core.people, &people_env, people::Event::Restored, &mut out);
}

#[test]
fn default_keys_and_plans_cover_only_their_allowed_payers() {
    use accounts::{Holder, Use};
    for (plan, holder, party, project) in [
        (false, Holder::Party(ALICE), false, false),
        (false, Holder::Deployment, true, true),
        (true, Holder::Party(ALICE), false, true),
    ] {
        let (core, _) = prepared(plan, holder, None);
        assert!(serves(&core, Use::Party(ALICE)));
        assert_eq!(serves(&core, Use::Party(BOB)), party);
        assert_eq!(serves(&core, Use::Project(PROJECT)), project);
    }
}

#[test]
fn explicit_uses_replace_defaults_and_never_bypass_plan_rules() {
    use accounts::{Holder, Use};
    for (plan, uses, alice, bob) in [
        (false, vec![], false, false),
        (false, vec![Use::Party(BOB)], false, true),
        (true, vec![Use::Party(BOB)], false, false),
        (true, vec![Use::Party(ALICE)], true, false),
    ] {
        let (core, _) = prepared(plan, Holder::Party(ALICE), Some(uses));
        assert_eq!((serves(&core, Use::Party(ALICE)), serves(&core, Use::Party(BOB))), (alice, bob));
    }
}

#[test]
fn plan_coverage_rechecks_current_project_membership() {
    use accounts::{Holder, Use};
    let (mut core, env) = prepared(true, Holder::Party(ALICE), Some(vec![Use::Project(PROJECT), Use::Party(SERVICE)]));
    roster(&mut core, &env, &[ALICE, SERVICE]);
    assert!(serves(&core, Use::Project(PROJECT)) && serves(&core, Use::Party(SERVICE)));
    roster(&mut core, &env, &[ALICE, SERVICE, BOB]);
    assert!(!serves(&core, Use::Project(PROJECT)));
    assert!(serves(&core, Use::Party(SERVICE)));
}

fn preferred(number: u32) -> tasks::LlmChoice {
    tasks::LlmChoice {
        account: Some(number),
        main: tasks::ModelEffort { model: 0, effort: Some(0) },
        subagents: Box::new([]),
    }
}
fn with_model(core: &mut Core) {
    core.settings.llm_providers.providers[0].models = Box::new([b"scripted".as_slice().into()]);
}

#[test]
fn a_named_account_refusal_keeps_the_number_with_its_reason() {
    let (mut core, _) = prepared(false, accounts::Holder::Party(ALICE), None);
    with_model(&mut core);
    assert_eq!(
        core.account_preference(Some(&preferred(999)), tasks::Payer::Party(ALICE), None),
        Err(tasks::Refusal::Account { account: 999, why: tasks::AccountWhy::Unknown })
    );
    assert_eq!(
        core.account_preference(Some(&preferred(ACCOUNT)), tasks::Payer::Party(BOB), None),
        Err(tasks::Refusal::Account { account: ACCOUNT, why: tasks::AccountWhy::Uses })
    );
    core.settings.llm_providers.providers[0].number = 4;
    assert_eq!(
        core.account_preference(Some(&preferred(ACCOUNT)), tasks::Payer::Party(ALICE), None),
        Err(tasks::Refusal::Account { account: ACCOUNT, why: tasks::AccountWhy::Provider })
    );
}

#[test]
fn an_unavailable_named_account_is_admitted_for_its_payer() {
    let (mut core, _) = prepared(false, accounts::Holder::Party(ALICE), None);
    with_model(&mut core);
    assert!(!core.accounts.account_views().get(0).unwrap().usable);
    assert_eq!(core.account_preference(Some(&preferred(ACCOUNT)), tasks::Payer::Party(ALICE), None), Ok(()));
}

#[test]
fn a_batch_preference_uses_the_prospective_financial_chain() {
    let (mut core, _) = prepared(false, accounts::Holder::Party(ALICE), None);
    with_model(&mut core);
    let first = new_row(&core, 2, tasks::Funder::Pool { project: PROJECT, person: BOB, period: 1 }, None);
    let mut child = new_row(&core, 3, tasks::Funder::Task(2), None);
    child.llm = Some(preferred(ACCOUNT));
    let mut batch = [first, child];
    assert_eq!(
        core.resolve_llm(&mut batch),
        Err(tasks::Problem::new(Some(3), tasks::Refusal::Account { account: ACCOUNT, why: tasks::AccountWhy::Uses }))
    );
    batch[0].funder = tasks::Funder::Pool { project: PROJECT, person: ALICE, period: 1 };
    assert_eq!(core.resolve_llm(&mut batch), Ok(()));
}

#[test]
fn acceptance_rechecks_a_preference_against_the_new_funder() {
    let (mut core, _) = prepared(false, accounts::Holder::Party(ALICE), None);
    with_model(&mut core);
    let alice = tasks::Funder::Pool { project: PROJECT, person: ALICE, period: 1 };
    let mut new = new_row(&core, 2, alice, None);
    new.llm = Some(preferred(ACCOUNT));
    assert_eq!(core.funded_accounts(&[new.clone()], alice), Ok(()));
    let proposal = tasks::PersonProposal {
        number: 1,
        proposer: ALICE,
        project: PROJECT,
        action: tasks::PersonAction::Batch(Box::new([new])),
        reason: Box::new([]),
        state: tasks::PersonProposalState::Pending { since: Wall::EPOCH },
    };
    assert_eq!(
        core.goal_accept_allowed(&proposal, PROJECT, BOB, people::Role::Owner, 1, Wall::EPOCH),
        Err(people::Refusal::Account { account: ACCOUNT, why: people::AccountWhy::Uses })
    );
}

#[test]
fn a_limited_kept_account_refuses_a_new_account_of_its_provider() {
    let (mut core, env) = fixture(vec![
        account(ACCOUNT, accounts::Holder::Party(ALICE)),
        source(ACCOUNT, SOURCE, 0, accounts::Kind::Key),
        account(30, accounts::Holder::Party(ALICE)),
        source(30, 31, 0, accounts::Kind::Key),
    ]);
    with_model(&mut core);
    let mut out = Queue::with_capacity(accounts::max_out(&env.limits.accounts));
    let account_env = Env { now: env.now, wall: env.wall, limits: env.limits.accounts };
    accounts::step(
        &mut core.accounts,
        &account_env,
        accounts::Event::Limited { account: SOURCE, reset: None },
        &mut out,
    );
    assert_eq!(
        core.account_preference(Some(&preferred(30)), tasks::Payer::Party(ALICE), Some(ACCOUNT)),
        Err(tasks::Refusal::Account { account: 30, why: tasks::AccountWhy::Limited })
    );
    accounts::step(
        &mut core.accounts,
        &account_env,
        accounts::Event::Change {
            request: Token::new(1),
            holder: accounts::Holder::Party(ALICE),
            change: accounts::Change::Resume { source: SOURCE },
        },
        &mut out,
    );
    assert_eq!(core.account_preference(Some(&preferred(30)), tasks::Payer::Party(ALICE), Some(ACCOUNT)), Ok(()));
}
