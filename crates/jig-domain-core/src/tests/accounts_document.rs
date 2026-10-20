//! The accounts document copies only its holder's facts and its asker's sign-ins.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "unit-test fixtures are ordinary Rust under programming-model.md section 10.2"
)]
use super::*;
use documents::{AccountLine, Accounts, LastEnd, SignInLine, SourceLine};

const ALICE: u64 = 1;
const BOB: u64 = 2;
const ALICE_SIGN_IN: u64 = 10;
const BOB_SIGN_IN: u64 = 11;
const PROVIDER: u32 = 3;
const OWN: u32 = 20;
const OTHER: u32 = 30;
const DEPLOYMENT: u32 = 40;
const FIRST_SOURCE: u32 = 21;
const LAST_SOURCE: u32 = 22;
// A new source is numbered first, its new account second.
const MADE_ACCOUNT: u32 = 1;
const UNKNOWN: u64 = 99;

pub(super) fn fixture(records: Vec<accounts::Stored>) -> (Core, Env<Limits>) {
    let mut limits = limits();
    // Three holders, plus a second own account for the cut test.
    limits.accounts.accounts = 4;
    limits.accounts.holder_accounts = 2;
    let mut config = config(73);
    config.owners = Box::new([people::InitialOwner {
        project: 0,
        identity: people::IdentityKey { provider: 0, subject: ALICE.to_be_bytes().into() },
    }]);
    config.settings.llm_providers = accounts::Providers {
        providers: Box::new([accounts::Provider {
            name: b"Fake".as_slice().into(),
            signed_in_plan: true,
            manage_usage: Some(b"https://usage.example/".as_slice().into()),
            number: PROVIDER,
            models: Box::new([]),
            offers: accounts::Offers { sign_in: true, key: true, programs: Box::new([]) },
        }]),
    };
    let mut core = Core::new(config, &limits);
    core.connectors = Box::new([]);
    core.accounts = accounts::Domain::new(&limits.accounts, core.settings.llm_providers.clone());
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut out = Queue::with_capacity(people::max_out(&limits.people));
    let people_env = Env { now: env.now, wall: env.wall, limits: limits.people };
    for (person, sign_in) in [(ALICE, ALICE_SIGN_IN), (BOB, BOB_SIGN_IN)] {
        let identity = people::Identity {
            key: people::IdentityKey { provider: 0, subject: person.to_be_bytes().into() },
            login: b"party".as_slice().into(),
            name: b"Party".as_slice().into(),
        };
        for record in [
            people::Stored::Person { number: person, identity },
            people::Stored::SignIn {
                number: sign_in,
                person,
                expires: Wall::from_nanos(Duration::from_secs(60).as_nanos()),
            },
        ] {
            people::step(&mut core.people, &people_env, people::Event::Restore { record }, &mut out);
        }
    }
    people::step(
        &mut core.people,
        &people_env,
        people::Event::Restore {
            record: people::Stored::Roles {
                project: 0,
                holdings: Box::new([people::Holding { person: ALICE, role: people::Role::Owner }]),
            },
        },
        &mut out,
    );
    people::step(&mut core.people, &people_env, people::Event::Restored, &mut out);
    for record in records {
        drop(step(&mut core, &env, Event::Account(accounts::Event::Restore { record })));
    }
    drop(step(&mut core, &env, Event::Account(accounts::Event::Want)));
    (core, env)
}
pub(super) fn account(number: u32, holder: accounts::Holder) -> accounts::Stored {
    accounts::Stored::Account {
        place: 0,
        uses: None,
        number,
        provider: PROVIDER,
        holder,
        subject: number.to_be_bytes().into(),
        display: b"Name".as_slice().into(),
    }
}
pub(super) fn source(account: u32, number: u32, place: u32, kind: accounts::Kind) -> accounts::Stored {
    accounts::Stored::Source { account, number, place, kind, name: Box::new([]), held: None }
}
fn own_records() -> Vec<accounts::Stored> {
    vec![
        account(OWN, accounts::Holder::Party(ALICE)),
        source(OWN, LAST_SOURCE, 7, accounts::Kind::Key),
        source(OWN, FIRST_SOURCE, 2, accounts::Kind::SignIn),
    ]
}
fn mixed() -> Vec<accounts::Stored> {
    let mut records = own_records();
    records.extend([
        account(OTHER, accounts::Holder::Party(BOB)),
        source(OTHER, 31, 0, accounts::Kind::SignIn),
        account(DEPLOYMENT, accounts::Holder::Deployment),
        source(DEPLOYMENT, 41, 0, accounts::Kind::SignIn),
    ]);
    records
}
fn document(core: &Core, env: &Env<Limits>, party: u64) -> Accounts {
    let Document::Accounts(document) =
        make(core, &env.limits, party, &Subject::Accounts { party }, &window(), env.wall).expect("own accounts")
    else {
        panic!("accounts document")
    };
    *document
}
fn ask(core: &mut Core, env: &Env<Limits>, by: u64, key: u8, holder: people::AccountHolder) -> u32 {
    let sign_in = if by == ALICE { ALICE_SIGN_IN } else { BOB_SIGN_IN };
    let Requests::Out(out) = step(
        core,
        env,
        Event::People(people::Event::Ask {
            models: Box::new([]),
            reply_to: ReplyTo::new(Token::new(u64::from(key))),
            sign_in,
            key: [key; 16],
            ask: people::Ask::AccountSignIn { holder, provider: PROVIDER },
        }),
    );
    let source = core.accounts.pending().get(0).unwrap_or_else(|| panic!("one sign-in under way: {out:?}")).source;
    core.reclaim();
    source
}
fn finish(core: &mut Core, env: &Env<Limits>, source: u32) {
    drop(step(
        core,
        env,
        Event::Account(accounts::Event::SignedIn {
            source,
            subject: b"new".as_slice().into(),
            display: b"Name".as_slice().into(),
            declined: false,
        }),
    ));
}
fn line(place: u32, number: u32, kind: accounts::Kind) -> SourceLine {
    SourceLine {
        place,
        number,
        kind,
        state: AccountState::Asking,
        usable: false,
        expiring: false,
        attention: false,
        spent_until: None,
    }
}

#[test]
fn a_partys_accounts_document_lists_its_own_accounts_with_their_sources_in_order() {
    let (core, env) = fixture(mixed());
    assert_eq!(
        document(&core, &env, ALICE).accounts.as_slice(),
        &[AccountLine {
            provider_name: Some(Text { bytes: b"Fake".as_slice().into(), cut: false }),
            manage_usage: Some(b"https://usage.example/".as_slice().into()),
            place: 0,
            uses: None,
            plan: true,
            number: OWN,
            provider: Some(PROVIDER),
            display: Text { bytes: b"Name".as_slice().into(), cut: false },
            holder: accounts::Holder::Party(ALICE),
            sources: list([line(0, FIRST_SOURCE, accounts::Kind::SignIn), line(1, LAST_SOURCE, accounts::Kind::Key)]),
            cut: false,
        }]
    );
}
fn list<T>(items: impl IntoIterator<Item = T>) -> List<T> {
    let items: Vec<_> = items.into_iter().collect();
    let mut result = List::with_capacity(u32::try_from(items.len()).expect("small fixture"));
    for item in items {
        result.push(item).ok().expect("fixture bound");
    }
    result
}

#[test]
fn a_sign_in_under_way_is_shown_to_the_party_that_asked_and_unsettles_it() {
    let (mut core, env) = fixture(vec![]);
    let source = ask(&mut core, &env, ALICE, 1, people::AccountHolder::Deployment);
    let underway = document(&core, &env, ALICE);
    assert_eq!(
        underway.signing.as_slice(),
        &[SignInLine { source, provider: PROVIDER, holder: accounts::Holder::Deployment }]
    );
    assert!(!underway.settled);
    let other = document(&core, &env, BOB);
    assert!(other.signing.is_empty());
    assert!(other.settled);
    finish(&mut core, &env, source);
    let finished = document(&core, &env, ALICE);
    assert!(finished.signing.is_empty());
    assert!(finished.settled);
}

#[test]
fn the_document_keeps_how_the_last_sign_in_ended() {
    let (mut core, env) = fixture(vec![]);
    let source = ask(&mut core, &env, ALICE, 1, people::AccountHolder::Own);
    finish(&mut core, &env, source);
    assert_eq!(
        document(&core, &env, ALICE).last_end,
        Some(LastEnd { source, end: accounts::End::Made { account: MADE_ACCOUNT, declined: false } })
    );
    let source = ask(&mut core, &env, ALICE, 2, people::AccountHolder::Own);
    finish(&mut core, &env, source);
    core.drain_facts(&env.limits);
    assert_eq!(
        document(&core, &env, ALICE).last_end,
        Some(LastEnd { source, end: accounts::End::Renewed { account: MADE_ACCOUNT, declined: false } })
    );
    assert_eq!(document(&core, &env, BOB).last_end, None);
    let source = ask(&mut core, &env, BOB, 3, people::AccountHolder::Own);
    finish(&mut core, &env, source);
    assert_eq!(
        document(&core, &env, BOB).last_end,
        Some(LastEnd {
            source,
            end: accounts::End::Refused { refusal: accounts::Refusal::Held { provider: PROVIDER } }
        })
    );
}

fn grant(core: &mut Core, env: &Env<Limits>, source: u32) {
    drop(step(
        core,
        env,
        Event::Account(accounts::Event::Component(accounts::FromComponent::Granted {
            form: accounts::Form::Credential(accounts::Scheme::Bearer),
            account: source,
            generation: 1,
            valid: Duration::from_secs(60),
        })),
    ));
}
#[test]
fn usable_follows_a_source_the_party_or_the_deployment_holds() {
    let mut records = own_records();
    records.extend([account(DEPLOYMENT, accounts::Holder::Deployment), source(DEPLOYMENT, 41, 0, accounts::Kind::Key)]);
    let (mut core, env) = fixture(records);
    assert!(!document(&core, &env, ALICE).usable);
    grant(&mut core, &env, FIRST_SOURCE);
    assert!(document(&core, &env, ALICE).usable);
    assert!(!document(&core, &env, BOB).usable);
    grant(&mut core, &env, 41);
    assert!(document(&core, &env, BOB).usable);
}

#[test]
fn another_partys_accounts_subject_is_not_allowed() {
    let (core, env) = fixture(vec![]);
    assert_eq!(
        make(&core, &env.limits, ALICE, &Subject::Accounts { party: BOB }, &window(), env.wall),
        Err(Missing::NotAllowed)
    );
    assert_eq!(
        make(&core, &env.limits, UNKNOWN, &Subject::Accounts { party: UNKNOWN }, &window(), env.wall),
        Err(Missing::Unknown)
    );
}

#[test]
fn a_later_state_is_a_wall_time_only_with_a_clock() {
    let (mut core, env) = fixture(own_records());
    drop(step(
        &mut core,
        &env,
        Event::Account(accounts::Event::Component(accounts::FromComponent::Failed {
            account: FIRST_SOURCE,
            failure: accounts::Failure::Later { after: Some(Duration::from_nanos(5)) },
        })),
    ));
    assert_eq!(
        document(&core, &env, ALICE).accounts.get(0).unwrap().sources.get(0).unwrap().state,
        AccountState::Later { until: None }
    );
    let Document::Accounts(clock) = make_with_clock(
        &core,
        &env.limits,
        ALICE,
        &Subject::Accounts { party: ALICE },
        &window(),
        DocumentClock { now: env.now, wall: Wall::from_nanos(100) },
    )
    .unwrap() else {
        panic!("accounts document")
    };
    assert_eq!(
        clock.accounts.get(0).unwrap().sources.get(0).unwrap().state,
        AccountState::Later { until: Some(Wall::from_nanos(105)) }
    );
}

#[test]
fn the_accounts_document_is_cut_at_the_line_bound() {
    let mut records = own_records();
    records.extend([account(OTHER, accounts::Holder::Party(ALICE)), source(OTHER, 31, 0, accounts::Kind::SignIn)]);
    let (core, mut env) = fixture(records);
    env.limits.documents.lines = 1;
    let doc = document(&core, &env, ALICE);
    assert_eq!(doc.accounts.len(), 1);
    assert!(doc.cut);
    assert_eq!(doc.accounts.get(0).unwrap().sources.len(), 1);
    assert!(doc.accounts.get(0).unwrap().cut);
}

#[test]
fn the_deployment_document_lists_only_the_deployments_accounts_with_their_sources() {
    let (core, mut env) = fixture(mixed());
    env.limits.documents.lines = 1;
    let Document::Deployment(doc) = make(&core, &env.limits, ALICE, &Subject::Deployment, &window(), env.wall).unwrap()
    else {
        panic!("deployment document")
    };
    assert_eq!(doc.accounts.len(), 1);
    let account = doc.accounts.get(0).unwrap();
    assert_eq!(account.number, DEPLOYMENT);
    assert_eq!(account.provider, Some(PROVIDER));
    assert_eq!(account.display, Text { bytes: b"Name".as_slice().into(), cut: false });
    assert_eq!(account.holder, accounts::Holder::Deployment);
    assert_eq!(account.sources.as_slice(), &[line(0, 41, accounts::Kind::SignIn)]);
    assert!(!doc.cut, "the two parties' accounts do not count against deployment's bound");
}

fn change_account(core: &mut Core, env: &Env<Limits>, holder: accounts::Holder, change: accounts::Change) {
    let mut out = Queue::with_capacity(accounts::max_out(&env.limits.accounts));
    accounts::step(
        &mut core.accounts,
        &Env { now: env.now, wall: env.wall, limits: env.limits.accounts },
        accounts::Event::Change { request: Token::new(70), holder, change },
        &mut out,
    );
    let mut answered = false;
    while let Some(request) = out.pop() {
        if let accounts::Request::Answer { answer: accounts::Answer::Done, .. } = request {
            answered = true;
        }
    }
    assert!(answered);
}
#[test]
fn the_accounts_document_shows_uses_and_order() {
    let mut records = own_records();
    records.extend([
        account(25, accounts::Holder::Party(ALICE)),
        source(25, 26, 0, accounts::Kind::Key),
        account(DEPLOYMENT, accounts::Holder::Deployment),
        source(DEPLOYMENT, 41, 0, accounts::Kind::Key),
    ]);
    let (mut core, env) = fixture(records);
    let uses = Box::new([Use::Party(BOB), Use::Project(0)]);
    change_account(
        &mut core,
        &env,
        accounts::Holder::Party(ALICE),
        accounts::Change::SetUses { account: 25, uses: Some(uses) },
    );
    change_account(
        &mut core,
        &env,
        accounts::Holder::Party(ALICE),
        accounts::Change::MoveAccount { account: 25, place: 0 },
    );
    let result = document(&core, &env, ALICE);
    let rows = result.accounts.as_slice();
    assert_eq!(
        rows.iter().map(|r| (r.number, r.place, r.plan)).collect::<Vec<_>>(),
        vec![(25, 0, false), (OWN, 1, true)]
    );
    assert_eq!(rows[0].uses.as_ref().expect("explicit uses").as_slice(), &[Use::Party(BOB), Use::Project(0)]);
    assert_eq!(rows[1].uses, None);
    change_account(
        &mut core,
        &env,
        accounts::Holder::Deployment,
        accounts::Change::SetUses { account: DEPLOYMENT, uses: Some(Box::new([Use::Party(ALICE)])) },
    );
    let Document::Deployment(deployment) =
        make(&core, &env.limits, ALICE, &Subject::Deployment, &window(), env.wall).expect("owner deployment")
    else {
        panic!("deployment")
    };
    let row = deployment.accounts.get(0).expect("deployment account");
    assert_eq!((row.number, row.place, row.plan), (DEPLOYMENT, 0, false));
    assert_eq!(row.uses.as_ref().expect("deployment uses").as_slice(), &[Use::Party(ALICE)]);
    let mut short = env;
    short.limits.documents.lines = 1;
    let cut = document(&core, &short, ALICE);
    assert!(cut.cut);
    assert_eq!(cut.accounts.get(0).expect("first account").number, 25);
}
#[test]
fn serves_work_follows_the_uses() {
    let (mut core, env) = fixture(vec![
        account(OWN, accounts::Holder::Party(ALICE)),
        source(OWN, FIRST_SOURCE, 0, accounts::Kind::Key),
        account(DEPLOYMENT, accounts::Holder::Deployment),
        source(DEPLOYMENT, 41, 0, accounts::Kind::Key),
    ]);
    for source in [FIRST_SOURCE, 41] {
        drop(step(
            &mut core,
            &env,
            Event::Account(accounts::Event::Component(accounts::FromComponent::Granted {
                form: accounts::Form::Credential(accounts::Scheme::Key),
                account: source,
                generation: 1,
                valid: Duration::from_secs(60),
            })),
        ));
    }
    change_account(
        &mut core,
        &env,
        accounts::Holder::Deployment,
        accounts::Change::SetUses { account: DEPLOYMENT, uses: Some(Box::new([])) },
    );
    for (uses, alice, bob) in [
        (None, true, false),
        (Some(vec![Use::Party(BOB)]), false, true),
        (Some(vec![Use::Project(0)]), false, false),
        (Some(vec![]), false, false),
    ] {
        change_account(
            &mut core,
            &env,
            accounts::Holder::Party(ALICE),
            accounts::Change::SetUses { account: OWN, uses: uses.map(Vec::into_boxed_slice) },
        );
        assert_eq!((document(&core, &env, ALICE).usable, document(&core, &env, BOB).usable), (alice, bob));
    }
    change_account(
        &mut core,
        &env,
        accounts::Holder::Deployment,
        accounts::Change::SetUses { account: DEPLOYMENT, uses: None },
    );
    assert!(document(&core, &env, ALICE).usable && document(&core, &env, BOB).usable);
    change_account(
        &mut core,
        &env,
        accounts::Holder::Deployment,
        accounts::Change::SetUses { account: DEPLOYMENT, uses: Some(Box::new([Use::Party(ALICE)])) },
    );
    assert!(document(&core, &env, ALICE).usable);
    assert!(!document(&core, &env, BOB).usable);
    drop(step(&mut core, &env, Event::Account(accounts::Event::Limited { account: 41, reset: None })));
    assert!(!document(&core, &env, ALICE).usable);
}

#[test]
fn needs_sign_in_retains_party_and_deployment_source_lines() {
    let (mut core, env) = fixture(mixed());
    for source in [FIRST_SOURCE, 41] {
        grant(&mut core, &env, source);
        drop(step(
            &mut core,
            &env,
            Event::Account(accounts::Event::Component(accounts::FromComponent::Failed {
                account: source,
                failure: accounts::Failure::SignInAgain,
            })),
        ));
    }
    let party = document(&core, &env, ALICE);
    let Document::Deployment(deployment) =
        make(&core, &env.limits, ALICE, &Subject::Deployment, &window(), env.wall).unwrap()
    else {
        panic!("deployment document")
    };
    for (source, line) in [
        (FIRST_SOURCE, party.accounts.get(0).unwrap().sources.get(0).unwrap()),
        (41, deployment.accounts.get(0).unwrap().sources.get(0).unwrap()),
    ] {
        assert_eq!(line.number, source);
        assert_eq!(line.kind, accounts::Kind::SignIn);
        assert_eq!(line.state, AccountState::SignInAgain);
        assert!(line.attention);
        assert!(!line.usable);
    }
    assert_eq!(party.accounts.len(), 1);
    assert_eq!(deployment.accounts.len(), 1);
}

#[test]
fn the_providers_uses_rule_checks_people_and_projects_for_plans_and_llm_keys() {
    const SERVICE: u64 = 3;
    for kind in [accounts::Kind::SignIn, accounts::Kind::Key] {
        let (mut core, env) =
            fixture(vec![account(OWN, accounts::Holder::Party(ALICE)), source(OWN, FIRST_SOURCE, 0, kind)]);
        let mut limits = env.limits.people;
        limits.people = 3;
        limits.projects = 3;
        limits.holdings = 3;
        core.people = people::Domain::new(&limits, Box::new([]), core.settings.deployment_provider);
        let penv = Env { now: env.now, wall: env.wall, limits };
        let mut out = Queue::with_capacity(people::max_out(&limits));
        for (party, provider) in [(ALICE, 0), (BOB, 0), (SERVICE, core.settings.deployment_provider)] {
            people::step(
                &mut core.people,
                &penv,
                people::Event::Restore {
                    record: people::Stored::Person {
                        number: party,
                        identity: people::Identity {
                            key: people::IdentityKey { provider, subject: party.to_be_bytes().into() },
                            login: Box::new([]),
                            name: Box::new([]),
                        },
                    },
                },
                &mut out,
            );
        }
        for (project, parties) in [(0, vec![ALICE, SERVICE]), (1, vec![ALICE, BOB]), (2, vec![])] {
            people::step(
                &mut core.people,
                &penv,
                people::Event::Restore {
                    record: people::Stored::Roles {
                        project,
                        holdings: parties
                            .into_iter()
                            .map(|person| people::Holding { person, role: people::Role::Member })
                            .collect(),
                    },
                },
                &mut out,
            );
        }
        people::step(&mut core.people, &penv, people::Event::Restored, &mut out);
        assert!(core.people.ready());
        let plan = kind == accounts::Kind::SignIn;
        for (payer, expected) in [
            (people::Use::Party(ALICE), Ok(())),
            (people::Use::Party(SERVICE), Ok(())),
            (people::Use::Party(BOB), if plan { Err(people::Refusal::Provider) } else { Ok(()) }),
            (people::Use::Project(0), Ok(())),
            (people::Use::Project(1), if plan { Err(people::Refusal::Provider) } else { Ok(()) }),
            (people::Use::Project(2), Ok(())),
            (people::Use::Party(UNKNOWN), Err(people::Refusal::Unknown)),
            (people::Use::Project(99), Err(people::Refusal::Unknown)),
        ] {
            let result = routing::account_requests::change(
                &core,
                accounts::Holder::Party(ALICE),
                people::AccountChange::SetUses { account: OWN, uses: Some(Box::new([payer])) },
            );
            assert_eq!(result.map(|_| ()), expected, "{kind:?}, {payer:?}");
        }
        let empty: Box<[people::Use]> = Box::new([]);
        for uses in [None, Some(empty)] {
            routing::account_requests::change(
                &core,
                accounts::Holder::Party(ALICE),
                people::AccountChange::SetUses { account: OWN, uses },
            )
            .expect("the holder may keep default or empty uses");
        }
        assert_eq!(
            routing::account_requests::change(
                &core,
                accounts::Holder::Party(BOB),
                people::AccountChange::SetUses { account: OWN, uses: None }
            )
            .err(),
            Some(people::Refusal::Unknown)
        );
    }
    assert_eq!(people::USES, accounts::USES);
}

mod catalogue;
