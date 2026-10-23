use jig_core_world::effects::{World, fixture};
use jig_core_world::fixtures::live_row;
use jig_domain_authority as authority;
use jig_domain_core as core;
use jig_domain_tasks as tasks;
use jig_test_connector as connector;
use jig_test_domain as root;
use skein_lib::{Queue, ReplyTo, Token};

fn topic_world(seed: u64, grant: bool) -> World {
    let (mut configuration, limits) = fixture(seed, false);
    configuration.first.topics = Box::new([connector::TopicSpec {
        topic: 1,
        offered: true,
        resources: Box::new([jig_test_connector_world::path(1, 1)]),
    }]);
    let read_grant = authority::Grant {
        connector: 1,
        kind: 0,
        pattern: authority::Pattern { segments: Box::new([]), last: authority::Last::Open(Box::new([])) },
    };
    let mut rules = configuration.core.authority.rules().clone();
    rules.ceiling.grants = Box::new([read_grant.clone()]);
    let mut policy = configuration.core.seed_projects[0].policy.policy.clone();
    policy.ceiling.grants = Box::new([read_grant.clone()]);
    policy.roles[0].authority.grants = if grant { Box::new([read_grant]) } else { Box::new([]) };
    let mut domain = authority::Domain::new(rules, limits.core.authority).expect("topic rules");
    let mut out = Queue::with_capacity(authority::POLICY_MAX_OUT);
    authority::step(&mut domain, authority::Event::Policy { project: 1, policy }, &mut out);
    assert_eq!(out.pop(), Some(authority::PolicyFact::Added { project: 1 }));
    configuration.core.seed_projects[0].policy.policy = domain.policy(1).expect("checked seed policy").clone();
    configuration.core.authority =
        jig_domain_authority::Domain::new(domain.rules().clone(), *domain.limits()).expect("checked rules");
    World::configured(seed, false, configuration, limits)
}

fn subscribe(world: &mut World, topic: u16) {
    world.send(root::Event::TopicCall { to: ReplyTo::new(Token::new(801)), key: world.call_key(1), number: 1, topic });
}

fn news(world: &mut World, importance: u16) {
    world.send(root::Event::Connector {
        number: 1,
        event: connector::Event::System(connector::SystemEvent::News {
            topic: 1,
            importance,
            origin: connector::Origin::Other,
        }),
    });
}

#[test]
fn a_topic_call_under_a_read_grant_subscribes_the_task() {
    let mut world = topic_world(181, true);
    subscribe(&mut world, 1);
    assert!(matches!(world.answers.last(), Some((_, _, core::CallPart::Subscribed { .. }))));
    let task = world.call_key(1).task;
    assert!(
        live_row(&world, task)
            .subscriptions
            .iter()
            .any(|subscription| { subscription.kind == tasks::SubscriptionKind::Topic { connector: 1, topic: 1 } })
    );
}

#[test]
fn a_topic_call_without_a_read_grant_is_denied_naming_the_grant() {
    let mut world = topic_world(182, false);
    subscribe(&mut world, 1);
    assert!(matches!(world.answers.last(), Some((_, _, core::CallPart::ToolDenied { findings, .. }))
        if findings.contains(&authority::Finding::Grant { source: authority::Source::Task })));
    assert!(live_row(&world, world.call_key(1).task).subscriptions.is_empty());
}

#[test]
fn a_topic_not_offered_to_agents_is_refused_to_the_run() {
    let mut world = topic_world(183, true);
    subscribe(&mut world, 2);
    assert!(matches!(
        world.answers.last(),
        Some((_, _, core::CallPart::SubscriptionRefused(tasks::Problem { why: tasks::Refusal::Subscription, .. })))
    ));
    assert!(live_row(&world, world.call_key(1).task).subscriptions.is_empty());
}

#[test]
fn news_on_a_subscribed_topic_reaches_the_tasks_inbox() {
    let mut world = topic_world(184, true);
    subscribe(&mut world, 1);
    let subscription = live_row(&world, world.call_key(1).task).subscriptions[0].number;
    news(&mut world, 2);
    let message = live_row(&world, world.call_key(1).task)
        .inbox
        .iter()
        .find(|word| word.kind == tasks::MessageKind::News { subscription, class: tasks::NewsClass::Wakes })
        .expect("classified news in task inbox");
    assert_eq!(message.words.as_ref(), b"News on topic 1");
    assert!(
        world
            .inbound
            .iter()
            .any(|word| word.kind == tasks::MessageKind::News { subscription, class: tasks::NewsClass::Wakes })
    );
}

#[test]
fn dropped_news_reaches_no_inbox() {
    let mut world = topic_world(185, true);
    subscribe(&mut world, 1);
    let before = live_row(&world, world.call_key(1).task).inbox.clone();
    news(&mut world, 0);
    assert_eq!(live_row(&world, world.call_key(1).task).inbox, before);
}

#[test]
fn unsubscribing_a_topic_ends_it_at_the_connector() {
    let mut world = topic_world(186, true);
    subscribe(&mut world, 1);
    let subscription = live_row(&world, world.call_key(1).task).subscriptions[0].number;
    world.send(root::Event::Core(core::Event::NamedAction {
        to: ReplyTo::new(Token::new(802)),
        key: world.call_key(2),
        action: core::NamedAction::Unsubscribe { subscription },
    }));
    assert!(matches!(world.answers.last(), Some((_, _, core::CallPart::Unsubscribed))));
    assert!(!world.store.rows.contains_key(&root::Key::Connector {
        number: 1,
        key: connector::RecordKey::Subscription { topic: 1, task: world.call_key(1).task }
    }));
}

#[test]
fn ending_a_task_ends_its_topic_at_the_connector() {
    let mut world = topic_world(187, true);
    subscribe(&mut world, 1);
    let task = world.call_key(1).task;
    world.end(
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"finished".as_slice().into() },
            cancel_delegates: false,
        },
        0,
    );
    assert!(
        !world.store.rows.contains_key(&root::Key::Connector {
            number: 1,
            key: connector::RecordKey::Subscription { topic: 1, task },
        })
    );
}
