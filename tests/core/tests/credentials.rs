use jig_core_world::{
    effects::World,
    fixtures::{CREDENTIAL_RESOURCE, WORKSPACE_CREDENTIAL, ask_as},
    world,
};
use jig_domain_accounts as accounts;
use jig_domain_core as core;
use jig_domain_fleet as fleet;
use jig_domain_people as people;
use jig_test_domain as root;
use skein_lib::{Duration, Token};

fn configured(seed: u64) -> World {
    let (config, limits) = world::with_workspace_credential(seed);
    let mut world = World::configured(seed, false, config, limits);
    let (now, wall) = world.clocks();
    world.set_clock(now, wall);
    world
}
fn grant(world: &mut World, generation: u64) {
    world.send(root::Event::Credential { number: WORKSPACE_CREDENTIAL, generation, valid: Duration::from_secs(60) });
}
fn delegated(seed: u64) -> World {
    let mut world = configured(seed);
    grant(&mut world, 3);
    world.delegate_on_resource(CREDENTIAL_RESOURCE);
    world
}
fn hello(world: &mut World) {
    let hosting = world
        .assignments
        .iter()
        .map(|(task, attempt)| fleet::Hosted {
            run: Token::new(*task),
            attempt: Token::new(*attempt),
            phase: fleet::Phase::Active,
        })
        .collect();
    world.send(root::Event::Core(core::Event::Fleet(fleet::Event::Hello {
        channel: Token::new(7),
        hello: fleet::Hello {
            bounds: jig_core_world::fixtures::bounds(),
            stop_bound: Duration::from_millis(100),
            slots: 2,
            workstreams: Box::new([]),
            hosting,
        },
    })));
}
fn received(world: &World) -> (Token, fleet::Grant) {
    let (_, task, _, grant) = world.received_grant().expect("host received a credential grant");
    (task, grant)
}
#[test]
fn a_run_needing_a_credential_waits_until_it_is_granted() {
    let mut world = configured(150);
    world.delegate_on_resource(CREDENTIAL_RESOURCE);
    assert_eq!(world.assignments.len(), 1);
    grant(&mut world, 3);
    assert_eq!(world.assignments.len(), 2);
}
#[test]
fn the_assignment_carries_the_credential_named_by_its_table() {
    let world = delegated(151);
    let assignment = world.assigned.as_ref().expect("the delegate is assigned");
    assert!(assignment.grants.contains(&accounts::Grant {
        table: accounts::GrantTable::Application,
        account: WORKSPACE_CREDENTIAL,
        generation: 3,
        valid: Duration::from_secs(60),
    }));
}
#[test]
fn a_new_credential_generation_reaches_the_live_run() {
    let mut world = delegated(152);
    let task = world.assigned.as_ref().expect("delegate assigned").task;
    grant(&mut world, 4);
    assert_eq!(
        received(&world),
        (
            Token::new(task),
            fleet::Grant {
                table: fleet::GrantTable::Application,
                account: WORKSPACE_CREDENTIAL,
                generation: 4,
                valid: Duration::from_secs(60),
            }
        )
    );
}
#[test]
fn an_adopted_run_is_sent_its_credential_again() {
    let mut world = delegated(153);
    let task = world.assigned.as_ref().expect("delegate assigned").task;
    world.restart_with(world::with_workspace_credential(153).0);
    grant(&mut world, 4);
    hello(&mut world);
    assert_eq!(
        received(&world),
        (
            Token::new(task),
            fleet::Grant {
                table: fleet::GrantTable::Application,
                account: WORKSPACE_CREDENTIAL,
                generation: 4,
                valid: Duration::from_secs(60),
            }
        )
    );
}

/// The core world with a provider offering a sign-in, and room for a party's account beside the
/// configured one.
fn signing(seed: u64) -> World {
    let (mut config, mut limits) = jig_core_world::effects::fixture(seed, false);
    config.core.settings.llm_providers = accounts::Providers {
        providers: Box::new([accounts::Provider {
            number: PROVIDER,
            models: Box::new([]),
            offers: accounts::Offers { sign_in: true, key: false, programs: Box::new([]) },
        }]),
    };
    limits.core.accounts.accounts = 2;
    // The accounts story also opens the deployment watch, beside the project role.
    limits.core.people.projects = 2;
    jig_core_world::peers::journal(&mut limits);
    World::configured(seed, false, config, limits)
}
const PROVIDER: u32 = 3;
fn ask(world: &mut World, key: u8, ask: people::Ask) -> Option<&people::Reply> {
    ask_as(world, world.sign_in.expect("a signed-in party"), key, ask);
    world.people_answers.last()
}
fn signed_in(world: &mut World) -> (u32, u32) {
    let sign_in = people::Ask::AccountSignIn { holder: people::AccountHolder::Own, provider: PROVIDER };
    let Some(people::Reply::Outcome(people::Outcome::AccountSignInStarted { source, address })) =
        ask(world, 40, sign_in).cloned()
    else {
        panic!("the sign-in started: {:?}", world.people_answers.last());
    };
    assert_eq!(&*address, format!("https://issuer.test/visit/{source}").as_bytes());
    let account = world.domain.accounts().source(source).expect("the signed-in source").account;
    (account, source)
}
fn row(key: accounts::Key) -> root::Key {
    root::Key::Core(core::Key::Accounts(key))
}
#[test]
fn a_partys_sign_in_reaches_the_accounts_child_and_its_account_is_saved() {
    let mut world = signing(154);
    let (account, source) = signed_in(&mut world);
    let made = world.domain.accounts().account(account).expect("the account made");
    assert!(matches!(made.holder(), accounts::Holder::Party(_)), "the party's own account");
    assert_eq!(made.subject(), b"signed-in");
    assert!(world.store.rows.contains_key(&row(accounts::Key::Account { number: account })));
    assert!(world.store.rows.contains_key(&row(accounts::Key::Source { account, number: source })));
}
#[test]
fn a_partys_sign_out_removes_its_source_and_account() {
    let mut world = signing(155);
    let (account, source) = signed_in(&mut world);
    let sign_out = people::Ask::AccountSignOut { holder: people::AccountHolder::Own, source };
    assert_eq!(
        ask(&mut world, 41, sign_out),
        Some(&people::Reply::Outcome(people::Outcome::AccountSignedOut { confirmed: true }))
    );
    assert!(world.domain.accounts().source(source).is_none());
    assert!(world.domain.accounts().account(account).is_none());
    assert!(!world.store.rows.contains_key(&row(accounts::Key::Account { number: account })));
    assert!(!world.store.rows.contains_key(&row(accounts::Key::Source { account, number: source })));
}

const CONFIGURED_SOURCE: u32 = 1;
const OWN_WATCH: u64 = 61_001;
const OTHER_WATCH: u64 = 61_002;
const DEPLOYMENT_WATCH: u64 = 61_003;

fn watch(world: &mut World, sign_in: u64, watcher: u64, subject: core::documents::Watched) {
    world.send(root::Event::Core(core::Event::Watch {
        watcher: Token::new(watcher),
        sign_in,
        subject,
        window: core::documents::Window { rows: 8, after: None },
    }));
}
fn take(world: &mut World, watcher: u64) {
    world.send(root::Event::Core(core::Event::Delivered { watcher: Token::new(watcher) }));
}
fn delivered(world: &World, watcher: u64) -> &core::documents::Document {
    world
        .client_peer
        .as_ref()
        .expect("client peer")
        .delivered
        .iter()
        .rev()
        .find_map(|(_, delivery)| {
            let root::Delivery::Core(core::Held::Delivery { watcher: name, document }) = delivery else { return None };
            if name.raw() == watcher { Some(document.as_ref()) } else { None }
        })
        .expect("the watch delivered")
}
fn accounts_document(world: &World, watcher: u64) -> &core::documents::Accounts {
    let core::documents::Document::Accounts(document) = delivered(world, watcher) else { panic!("accounts document") };
    document
}
fn start_account(world: &mut World) -> u32 {
    let sign_in = world.sign_in.expect("signed in");
    ask_as(world, sign_in, 42, people::Ask::AccountSignIn { holder: people::AccountHolder::Own, provider: PROVIDER });
    // The independent component answers within send; the first watch delivery remains in flight.
    world
        .domain
        .accounts()
        .account_views()
        .iter()
        .find_map(|fact| {
            let account = world.domain.accounts().account(fact.account).expect("account view");
            matches!(account.holder(), accounts::Holder::Party(_)).then_some(fact.source)
        })
        .expect("the fake component made the party's source")
}

#[test]
fn a_partys_accounts_watch_follows_its_sign_in_from_start_to_end() {
    let mut world = signing(156);
    world.send(root::Event::Core(core::Event::Account(accounts::Event::Component(accounts::FromComponent::Failed {
        account: CONFIGURED_SOURCE,
        failure: accounts::Failure::SignInAgain,
    }))));
    let first = world.sign_in.expect("first sign-in");
    let second = jig_core_world::fixtures::second_party(&mut world, b"accounts-observer", people::Role::Observer);
    world.sign_in = Some(first);
    world.client_peer = Some(jig_core_world::client_peer::ClientPeer::default());
    watch(&mut world, first, OWN_WATCH, core::documents::Watched::Accounts);
    watch(&mut world, second, OTHER_WATCH, core::documents::Watched::Accounts);
    let empty = accounts_document(&world, OWN_WATCH);
    assert!(empty.accounts.is_empty());
    assert!(empty.settled);
    assert!(!empty.usable);
    take(&mut world, OWN_WATCH);
    take(&mut world, OTHER_WATCH);
    let source = start_account(&mut world);
    let underway = accounts_document(&world, OWN_WATCH);
    assert_eq!(underway.signing.len(), 1);
    let pending = underway.signing.get(0).expect("the party's sign-in is under way");
    assert_eq!((pending.source, pending.provider), (source, PROVIDER));
    assert!(!underway.settled);
    take(&mut world, OWN_WATCH);
    let finished = accounts_document(&world, OWN_WATCH);
    let account = finished.accounts.get(0).expect("own account");
    let first_source = account.sources.get(0).expect("the account's first source");
    assert_eq!((first_source.number, first_source.place), (source, 0));
    assert!(finished.signing.is_empty());
    assert!(finished.settled);
    assert_eq!(
        finished.last_end,
        Some(core::documents::LastEnd {
            source,
            end: accounts::End::Made { account: account.number, declined: false }
        })
    );
    assert!(accounts_document(&world, OTHER_WATCH).accounts.is_empty());
    assert!(accounts_document(&world, OTHER_WATCH).signing.is_empty());
    assert_eq!(accounts_document(&world, OTHER_WATCH).last_end, None);
    assert!(world.client_peer.as_ref().expect("client peer").delivered.iter().all(|(_, delivery)| {
        let root::Delivery::Core(core::Held::Delivery { watcher, document }) = delivery else { return true };
        watcher.raw() != OTHER_WATCH || matches!(document.as_ref(), core::documents::Document::Accounts(doc) if doc.accounts.is_empty() && doc.signing.is_empty() && doc.last_end.is_none())
    }));
}

#[test]
fn the_deployment_watch_shows_the_deployments_accounts_and_not_a_partys_own() {
    let mut world = signing(157);
    let sign_in = world.sign_in.expect("signed in");
    let person = world
        .store
        .rows
        .values()
        .find_map(|record| {
            let root::Record::Core(core::Record::People(people::Stored::SignIn { number, person, .. })) = record else {
                return None;
            };
            if *number == sign_in { Some(*person) } else { None }
        })
        .expect("the store kept the live sign-in");
    world.send(root::Event::Core(core::Event::People(people::Event::Roles {
        project: 0,
        holdings: Box::new([people::Holding { person, role: people::Role::Owner }]),
    })));
    world.client_peer = Some(jig_core_world::client_peer::ClientPeer::default());
    watch(&mut world, sign_in, OWN_WATCH, core::documents::Watched::Accounts);
    watch(&mut world, sign_in, DEPLOYMENT_WATCH, core::documents::Watched::Deployment);
    take(&mut world, OWN_WATCH);
    take(&mut world, DEPLOYMENT_WATCH);
    let source = start_account(&mut world);
    take(&mut world, OWN_WATCH);
    let own = accounts_document(&world, OWN_WATCH).accounts.get(0).expect("own account").number;
    take(&mut world, DEPLOYMENT_WATCH);
    let core::documents::Document::Deployment(deployment) = delivered(&world, DEPLOYMENT_WATCH) else {
        panic!("deployment document")
    };
    assert_eq!(deployment.accounts.len(), 1);
    assert_eq!(deployment.accounts.get(0).expect("the configured deployment account").number, CONFIGURED_SOURCE);
    assert!(!deployment.accounts.iter().any(|account| account.number == own));
    assert!(deployment.accounts.iter().all(|account| account.holder == accounts::Holder::Deployment));
    take(&mut world, OWN_WATCH);
    ask_as(&mut world, sign_in, 43, people::Ask::AccountSignOut { holder: people::AccountHolder::Own, source });
    assert!(accounts_document(&world, OWN_WATCH).accounts.is_empty());
}
