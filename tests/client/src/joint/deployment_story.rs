//! A configured first owner reads Deployment through the public tree.
use super::*;

fn world() -> World<Test, Joint> {
    let mut config = jig_core_world::world::config(8511);
    config.core.owners = Box::new([
        people::InitialOwner {
            project: 1,
            identity: people::IdentityKey { provider: 0, subject: 7_u64.to_be_bytes().into() },
        },
        people::InitialOwner {
            project: 0,
            identity: people::IdentityKey { provider: 0, subject: 7_u64.to_be_bytes().into() },
        },
    ]);
    config.core.projects = crate::test::list([1, 0]);
    let mut owner = jig_core_world::world::limits();
    owner.core.authority.projects = 2;
    let policy = config.core.authority.policy(1).expect("configured project policy").clone();
    let rules = config.core.authority.rules().clone();
    config.core.authority =
        jig_domain_authority::Domain::new(rules, owner.core.authority).expect("two configured projects");
    let mut out = skein_lib::Queue::with_capacity(jig_domain_authority::POLICY_MAX_OUT);
    for project in [1, 0] {
        jig_domain_authority::step(
            &mut config.core.authority,
            jig_domain_authority::Event::Policy { project, policy: policy.clone() },
            &mut out,
        );
        assert_eq!(out.pop(), Some(jig_domain_authority::PolicyFact::Added { project }));
    }
    owner.core.people.initial_owners = 2;
    owner.core.people.projects = 2;
    owner.core.documents.rows = 2;
    owner.core.documents.lines = 2;
    owner.core.documents.chips = 2;
    owner.core.documents.text_bytes = 32;
    let room = jig_domain_core::room_max(&owner.core).expect("fixture core bounds");
    let connectors = jig_test_connector::MAX_OUT.checked_mul(2).expect("two connector outputs");
    owner.journal.writes = owner.journal.writes.max(room.writes.checked_add(connectors).expect("fixture writes"));
    owner.journal.held = owner.journal.held.max(
        room.held
            .checked_add(connectors)
            .expect("fixture deliveries")
            .checked_mul(owner.journal.commits)
            .expect("fixture flights"),
    );
    let driver = Driver::new(config, owner, Peers::new(Vec::new(), Vec::new()), Time::ZERO, clock(Time::ZERO));
    let codec = jig_protocol_transcript::Limits {
        smith: smith_transcript::v3::CEILINGS,
        bytes: 1_048_576,
        turn: client::TurnLimits { messages: 2, blocks: 2, text: 32 },
    };
    let identity = people::Identity {
        key: people::IdentityKey { provider: 0, subject: 7_u64.to_be_bytes().into() },
        login: b"login".as_slice().into(),
        name: b"Person".as_slice().into(),
    };
    let mut front =
        Joint::new(Session::new(driver, crate::test::limits(), codec), BTreeMap::from([(1, identity)]), clock);
    front.unsigned_on_bind();
    World::new(Settings { seed: 8511, ..Settings::default() }, front)
}

#[test]
fn a_configured_deployment_owner_reads_capacity_account_and_connector_facts_from_the_engine() {
    configured_deployment_owner_reads_capacity_account_and_connector_facts();
}

fn configured_deployment_owner_reads_capacity_account_and_connector_facts() {
    let mut world = world();
    let client = world.client(1, jig_client_test_domain::Address::Jig(jig_domain_client::address::Address::Deployment));
    let within = Duration::from_secs(1);
    world.person(
        client,
        vec![
            Step::See { find: Find::named(Role::Button, b"Sign in with Test"), within },
            Step::Press { find: Find::named(Role::Button, b"Sign in with Test") },
            Step::See { find: Find::named(Role::Heading { level: 1 }, b"Deployment"), within },
            Step::See { find: Find::named(Role::Region, b"Tasks Capacity"), within },
            Step::See { find: Find::text(b"Account: 1"), within },
            Step::See { find: Find::text(b"Account generation: 1"), within },
            Step::See { find: Find::text(b"Hosts forgotten: 0"), within },
            Step::See { find: Find::named(Role::Region, b"Configured connectors"), within },
            Step::Pause { span: Duration::from_millis(100) },
            Step::See { find: Find::named(Role::Button, b"Project 1"), within },
            Step::Press { find: Find::named(Role::Button, b"Project 1") },
            Step::See { find: Find::named(Role::Tab, b"Settings"), within },
        ],
    );
    world.run(Until::Time(Time::from_nanos(10_000_000)));
    let tree = world.tree(client);
    let (start, region) = tree
        .nodes()
        .iter()
        .enumerate()
        .find(|(_, node)| node.role == Role::Region && node.name.as_deref() == Some(b"Configured connectors"))
        .expect("the person sees the configured connector catalogue");
    let connectors: Vec<_> = tree
        .nodes()
        .iter()
        .skip(start + 1)
        .take(region.size as usize - 1)
        .filter_map(|node| node.slot)
        .map(|slot| match jig_client_view::Owner::from_opaque(slot.owner) {
            Some(jig_client_view::Owner::Application { number: connector }) => connector,
            other @ (Some(jig_client_view::Owner::Jig) | None) => {
                panic!("connector catalogue slot has its actual side owner: {other:?}")
            }
        })
        .collect();
    assert_eq!(connectors, [1, 2], "both configured connectors retain their source order");
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| world.run(Until::Done)));
    let outcome = match result {
        Ok(value) => value,
        Err(failure) => {
            eprintln!("engine peer trace: {:#?}", world.front().session.driver.world.trace);
            eprintln!("durable store: {:#?}", world.front().session.driver.world.store.rows);
            std::panic::resume_unwind(failure);
        }
    };
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    let rows = &world.front().session.driver.world.store.rows;
    assert_eq!(
        rows.values()
            .filter(|value| matches!(
                value,
                jig_test_domain::Record::Core(jig_domain_core::Record::People(people::Stored::SignIn { .. }))
            ))
            .count(),
        1
    );
    assert_eq!(
        rows.values()
            .filter(|value| matches!(
                value,
                jig_test_domain::Record::Core(jig_domain_core::Record::People(people::Stored::Answer { .. }))
            ))
            .count(),
        0
    );
    assert!(!rows.values().any(|value| matches!(
        value,
        jig_test_domain::Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Turn(_)))
    )));
    assert!(rows.values().any(|value| matches!(value,
        jig_test_domain::Record::Core(jig_domain_core::Record::People(people::Stored::Roles { project: 0, holdings }))
        if holdings.iter().any(|holding| holding.role == people::Role::Owner)
    )));
}
