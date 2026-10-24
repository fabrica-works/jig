//! Replayable fault scenarios for `domain/testing.md`, 4, 7. Configuration
//! comes from the scenario, choices from a seed, and the referee observes every
//! drain. Conformance can reuse the independent peers and the chosen cuts.
use crate::{effects::World, peers::Peers};
use jig_fake_parties::{Act, Ask, Party, Role, Task};
use jig_fake_workers::{Fault, Script, Worker};
use skein_lib::{Duration, Time};

/// One owner, one to three workers and optionally one permanent engine slot.
/// Every activation waits for a message before taking its turn and parking.
#[must_use]
pub fn workers(seed: u64, count: u32, engine: bool) -> World {
    workers_at(seed, count, engine, None)
}

/// The same actors with a crash cut armed before startup.
#[must_use]
pub fn workers_at(seed: u64, count: u32, engine: bool, cut: Option<crate::effects::Cut>) -> World {
    assert!((1..=3).contains(&count));
    let (config, limits) = crate::peers::fixture(seed, u32::from(engine));
    let script = |read| {
        Box::new([
            Script::Wait,
            Script::Turn { body: b"kept whole turn".as_slice().into(), cost: 2, read: Some(read) },
            Script::Park,
        ]) as Box<[Script]>
    };
    let mut hosts: Vec<_> =
        (0..count).map(|index| Worker::new(7 + u64::from(index), 1, vec![script(1), script(2)])).collect();
    if engine {
        hosts.insert(0, Worker::new(0, 1, vec![script(1), script(2)]));
    }
    World::scripted_at(
        config,
        limits,
        Peers::new(
            hosts,
            vec![Party::new(
                1,
                Role::Owner,
                10_000,
                Box::new([
                    Act::SignIn { provider: 0, subject: 7_u64.to_be_bytes().into() },
                    Act::Request { key: [1; 16], ask: Ask::Chat { words: b"work".as_slice().into() } },
                ]),
            )],
        ),
        cut,
    )
}

/// Resume the owner's last chat, retaining the key for reproducible retries.
pub fn say(world: &mut World, key: u8) {
    world.peers.as_mut().expect("scripted peers").parties[0].extend(Box::new([Act::Request {
        key: [key; 16],
        ask: Ask::Say { task: Task::Last, words: b"continue".as_slice().into(), release: None },
    }]));
    world.drain();
}

/// Apply a link fault to the selected independent host and route its own report.
pub fn worker_fault(world: &mut World, index: usize, fault: Fault) {
    let now = Time::from_nanos(world.wall_time().as_nanos());
    let peers = world.peers.as_mut().expect("scripted peers");
    let reports = peers.workers[index].fault(fault, now);
    let events: Vec<_> = reports.into_iter().map(|up| peers.worker_event(up)).collect();
    for event in events {
        world.send(event);
    }
    world.drain();
}

/// Cold restart with the scenario's original charter and host capacities.
pub fn restart(world: &mut World, seed: u64, engine: bool) {
    world.restart_with(crate::peers::fixture(seed, u32::from(engine)).0);
    // Account usability is an independently refreshed runtime capability.
    world.send(jig_test_domain::Event::Core(jig_domain_core::Event::Account(jig_domain_accounts::Event::Component(
        jig_domain_accounts::FromComponent::Granted {
            form: jig_domain_accounts::Form::Credential(jig_domain_accounts::Scheme::Bearer),
            account: 1,
            generation: 1,
            valid: Duration::from_secs(60),
        },
    ))));
}

/// The plan fixture with a story's own document row bound, set before the
/// world prices its heap.
#[must_use]
pub fn plan_fixture_with_document_rows(seed: u64, rows: u32) -> (jig_test_domain::Config, jig_test_domain::Limits) {
    let (configuration, mut limits) = plan_fixture(seed);
    limits.core.documents.rows = rows;
    (configuration, limits)
}

/// The bounded two-level plan fixture used by resource and failure stories.
#[must_use]
pub fn plan_fixture(seed: u64) -> (jig_test_domain::Config, jig_test_domain::Limits) {
    let (mut configuration, mut limits) = crate::effects::fixture(seed, false);
    limits.core.tasks.tasks = 5;
    limits.core.tasks.project_tasks = 5;
    limits.core.tasks.tree_tasks = 5;
    limits.core.tasks.depth = 2;
    limits.core.tasks.delegates = 3;
    limits.core.tasks.funders = 16;
    limits.core.tasks.inbox_bytes = 512;
    limits.core.tasks.inbox_messages = 7;
    limits.core.views.runs = 5;
    limits.core.fleet.slots = 3;
    limits.core.fleet.attempts = 5;
    limits.core.call_records = 8;
    limits.core.people.amendment_bytes = 256;
    limits.core.people.inbox_recipients = 21;
    crate::peers::journal(&mut limits);
    let mut rules = configuration.core.defaults.rules.clone();
    rules.ceiling.delegation.tasks = 6;
    rules.ceiling.delegation.depth = 3;
    rules.maximum_run_spend = 10;
    let mut policy = configuration.core.seed_projects[0].policy.policy.clone();
    policy.ceiling.delegation = rules.ceiling.delegation.clone();
    policy.roles[0].authority.delegation =
        jig_domain_authority::Delegation { kinds: rules.ceiling.delegation.kinds.clone(), tasks: 5, depth: 3 };
    let mut checked = jig_domain_authority::Domain::new(rules, limits.core.authority).expect("bounded plan policy");
    let mut out = skein_lib::Queue::with_capacity(jig_domain_authority::POLICY_MAX_OUT);
    jig_domain_authority::step(&mut checked, jig_domain_authority::Event::Policy { project: 1, policy }, &mut out);
    assert_eq!(out.pop(), Some(jig_domain_authority::PolicyFact::Added { project: 1 }));
    configuration.core.seed_projects[0].policy.policy = checked.policy(1).expect("checked seed policy").clone();
    configuration.core.defaults.rules = (jig_domain_authority::Domain::new(checked.rules().clone(), *checked.limits())
        .expect("checked rules"))
    .rules()
    .clone();
    (configuration, limits)
}
