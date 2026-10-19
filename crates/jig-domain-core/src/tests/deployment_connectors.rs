//! Configured deployment connector references are supplied independently of their
//! owners' values (`client/domain/pages.md`, 7; `client/domain/root.md`, 6).
use super::*;
use crate::documents::Deployment;

fn source(connectors: &[u16], lines: u32) -> (Core, Limits) {
    let mut limits = limits();
    limits.connectors = u32::try_from(connectors.len()).unwrap();
    limits.documents.lines = lines;
    let mut config = config(71);
    config.connectors = connectors.into();
    let mut core = Core::new(config, &limits);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.people };
    let mut out = Queue::with_capacity(people::max_out(&limits.people));
    for record in [
        people::Stored::Person {
            number: 1,
            identity: people::Identity {
                key: people::IdentityKey { provider: 0, subject: Box::new([1]) },
                login: b"owner".as_slice().into(),
                name: b"A Person".as_slice().into(),
            },
        },
        people::Stored::Person {
            number: 2,
            identity: people::Identity {
                key: people::IdentityKey { provider: 0, subject: Box::new([2]) },
                login: b"outsider".as_slice().into(),
                name: b"Another Person".as_slice().into(),
            },
        },
        people::Stored::Roles {
            project: 0,
            holdings: Box::new([people::Holding { person: 1, role: people::Role::Owner }]),
        },
    ] {
        people::step(&mut core.people, &env, people::Event::Restore { record }, &mut out);
        assert_eq!(out.pop(), None, "authentic deployment membership restored");
    }
    people::step(&mut core.people, &env, people::Event::Restored, &mut out);
    assert_eq!(out.pop(), None);
    (core, limits)
}

#[expect(clippy::wildcard_enum_match_arm, reason = "ordinary proof helper receives a Deployment document")]
fn deployment(core: &Core, limits: &Limits) -> Deployment {
    match make(core, limits, 1, &Subject::Deployment, &window(), Wall::EPOCH).unwrap() {
        Document::Deployment(value) => *value,
        _ => panic!("deployment document"),
    }
}

fn assert_original_facts(before: &Deployment, after: &Deployment) {
    assert_eq!(after.hosts, before.hosts);
    assert_eq!(after.attempts, before.attempts);
    assert_eq!(after.accounts, before.accounts);
    assert_eq!(after.tasks, before.tasks);
    assert_eq!(after.workers, before.workers);
    assert_eq!(after.attempts_capacity, before.attempts_capacity);
    assert_eq!(after.waiting_attempts, before.waiting_attempts);
    assert_eq!(after.proposals, before.proposals);
    assert_eq!(after.projects, before.projects);
    assert_eq!(after.grace_bound, before.grace_bound);
    assert_eq!(after.hello_refusals, before.hello_refusals);
    assert_eq!(after.hosts_forgotten, before.hosts_forgotten);
    assert_eq!(after.hello_refusals_forgotten, before.hello_refusals_forgotten);
    assert_eq!(after.person_room_refusals, before.person_room_refusals);
    assert_eq!(after.person_room_refusals_forgotten, before.person_room_refusals_forgotten);
    assert_eq!(after.person_answer_retention, before.person_answer_retention);
    assert_eq!(after.task_room_refusals, before.task_room_refusals);
    assert_eq!(after.task_room_refusals_forgotten, before.task_room_refusals_forgotten);
    assert_eq!(after.cut, before.cut || after.connectors_cut);
}

#[test]
fn deployment_connector_references_keep_full_width_identifiers_and_configured_order_without_activity() {
    let (core, limits) = source(&[u16::MAX - 1, 9, 0, 1], 5);
    let value = deployment(&core, &limits);
    assert_eq!(value.connectors.as_slice(), &[u16::MAX - 1, 9, 0, 1]);
    assert_eq!(value.connectors.capacity(), 5);
    assert!(!value.connectors_cut);
    assert!(!value.cut);
    assert!(value.hosts.is_empty());
    assert!(value.attempts.is_empty());
    assert_eq!(value.tasks.used, 0);
    let (empty, empty_limits) = source(&[], 5);
    assert_original_facts(&deployment(&empty, &empty_limits), &value);
    assert!(core.people.identity(2).is_some());
    assert_eq!(core.people.role(2, 0), None);
    assert_eq!(core.people.identity(3), None);
    assert_eq!(make(&core, &limits, 2, &Subject::Deployment, &window(), Wall::EPOCH), Err(Missing::NotAllowed));
    assert_eq!(make(&core, &limits, 3, &Subject::Deployment, &window(), Wall::EPOCH), Err(Missing::Unknown));
}

#[test]
fn deployment_supplied_empty_connector_registry_is_complete_and_reserves_whole_document_bytes() {
    let (core, limits) = source(&[], 3);
    let value = deployment(&core, &limits);
    assert!(value.connectors.is_empty());
    assert_eq!(value.connectors.capacity(), 3);
    assert!(!value.connectors_cut);
    assert!(!value.cut);
    let allocated = u64::try_from(size_of::<Document>()).unwrap()
        + List::<Host>::worst_case(value.hosts.capacity()).unwrap()
        + List::<HostRun>::worst_case(value.attempts.capacity()).unwrap()
        + List::<Account>::worst_case(value.accounts.capacity()).unwrap()
        + List::<ProjectCapacity>::worst_case(value.projects.capacity()).unwrap()
        + List::<HelloRefusalLine>::worst_case(value.hello_refusals.capacity()).unwrap()
        + List::<people::CapacityRefusal>::worst_case(value.person_room_refusals.capacity()).unwrap()
        + List::<tasks::CapacityRefusal>::worst_case(value.task_room_refusals.capacity()).unwrap()
        + List::<u16>::worst_case(value.connectors.capacity()).unwrap();
    assert!(List::<u16>::worst_case(value.connectors.capacity()).unwrap() > 0);
    assert!(document_worst_case(&limits.documents).unwrap() >= allocated);
}

#[test]
fn deployment_connector_reference_loss_is_independent_and_included_in_the_aggregate_cut() {
    let (core, limits) = source(&[9, u16::MAX - 1, 1], 1);
    let value = deployment(&core, &limits);
    assert_eq!(value.connectors.as_slice(), &[9]);
    assert_eq!(value.connectors.capacity(), 1);
    assert!(value.connectors_cut);
    assert!(value.cut);
    let (empty, empty_limits) = source(&[], 1);
    let before = deployment(&empty, &empty_limits);
    assert!(!before.cut);
    assert_original_facts(&before, &value);
}

#[test]
fn deployment_complete_connector_registry_does_not_hide_original_refusal_history_loss_or_epoch() {
    let (mut core, limits) = source(&[u16::MAX - 1], 2);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.fleet };
    let mut out = Queue::with_capacity(fleet::max_out(&limits.fleet));
    for channel in [9, 10, 11] {
        fleet::step(
            &mut core.fleet,
            &env,
            fleet::Event::Hello {
                channel: Token::new(channel),
                hello: fleet::Hello {
                    bounds: fixtures::bounds(),
                    stop_bound: limits.fleet.grace,
                    slots: 1,
                    workstreams: Box::new([]),
                    hosting: Box::new([]),
                },
            },
            &mut out,
        );
        let Some(fleet::Request::Refuse { channel: refused }) = out.pop() else {
            panic!("hello refused");
        };
        assert!(refused == Token::new(channel));
        assert_eq!(out.pop(), None);
    }
    let value = deployment(&core, &limits);
    assert_eq!(value.connectors.as_slice(), &[u16::MAX - 1]);
    assert_eq!(value.connectors.capacity(), 2);
    assert!(!value.connectors_cut);
    assert!(value.cut);
    assert_eq!(value.hello_refusals_forgotten, 1);
    assert_eq!(value.hello_refusals.len(), 2);
    assert_eq!(value.hello_refusals.get(0).unwrap().channel, 11);
    assert_eq!(value.hello_refusals.get(1).unwrap().channel, 10);
    for row in &value.hello_refusals {
        assert_eq!(row.at, Wall::EPOCH);
        assert_eq!(row.slots, 1);
        assert_eq!(row.stop_bound, limits.fleet.grace);
        assert_eq!(row.why, fleet::HelloRefusal::StopBound { grace: limits.fleet.grace });
    }
}

#[test]
fn the_deployment_document_shows_the_last_fill_and_the_mark() {
    let (mut core, mut limits) = source(&[1], 2);
    limits.fill_mark = 9012;
    core.fill = 6789;
    assert_eq!(deployment(&core, &limits).store, StoreFill { bytes: 6789, mark: 9012 });
}

#[test]
fn a_refused_hello_shows_its_agent_shortfall() {
    let (mut core, limits) = source(&[1], 2);
    let mut bounds = fixtures::bounds();
    bounds.models = Box::new([]);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.fleet };
    let mut out = Queue::with_capacity(fleet::max_out(&limits.fleet));
    fleet::step(
        &mut core.fleet,
        &env,
        fleet::Event::Hello {
            channel: Token::new(9),
            hello: fleet::Hello {
                bounds,
                slots: 1,
                stop_bound: Duration::ZERO,
                workstreams: Box::new([]),
                hosting: Box::new([]),
            },
        },
        &mut out,
    );
    assert_eq!(out.pop(), Some(fleet::Request::Refuse { channel: Token::new(9) }));
    let value = deployment(&core, &limits);
    assert_eq!(
        value.hello_refusals.get(0).unwrap().why,
        fleet::HelloRefusal::Agent { why: fleet::AgentShortfall::Model { charter: 1, model: 0 } }
    );
}

fn waiting_account() -> (Core, Limits) {
    let (mut core, limits) = source(&[1], 2);
    let env = Env { now: Time::from_nanos(100_000_000_000), wall: Wall::from_nanos(1_000_000_000_000), limits };
    drop(step(&mut core, &env, Event::Account(accounts::Event::Want)));
    drop(step(
        &mut core,
        &env,
        Event::Account(accounts::Event::Component(accounts::FromComponent::Failed {
            account: 1,
            failure: accounts::Failure::Later { after: Some(Duration::from_secs(30)) },
        })),
    ));
    (core, limits)
}

#[test]
fn an_account_waiting_to_ask_again_carries_the_wall_it_asks_at() {
    let (core, limits) = waiting_account();
    let clock = DocumentClock { now: Time::from_nanos(110_000_000_000), wall: Wall::from_nanos(2_000_000_000_000) };
    let Document::Deployment(value) =
        make_with_clock(&core, &limits, 1, &Subject::Deployment, &window(), clock).unwrap()
    else {
        panic!("deployment document");
    };
    assert_eq!(
        value.accounts.get(0).unwrap().state,
        AccountState::Later { until: Some(Wall::from_nanos(2_020_000_000_000)) }
    );
}

#[test]
fn an_account_waiting_to_ask_again_has_no_wall_without_a_document_clock() {
    let (core, limits) = waiting_account();
    assert_eq!(deployment(&core, &limits).accounts.get(0).unwrap().state, AccountState::Later { until: None });
}
