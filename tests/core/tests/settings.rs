//! Settings travel through the real restart, store and commit barrier (engine.md, 17.7).
use jig_core_world::{client_peer::ClientPeer, effects::World, fixtures, peers, world};
use jig_domain_core::{self as core, documents, reads, settings};
use jig_domain_fleet as fleet;
use jig_domain_people as people;
use jig_test_domain as root;
use skein_lib::{Duration, ReplyTo, Token};

const SEED: u64 = 37_001;
const OWNER: u64 = 1;
fn configuration() -> (root::Config, root::Limits) {
    let document = jig_protocol_config::document(
        include_bytes!("settings/engine.json"),
        &jig_protocol_config::DocumentLimits { bytes: 16384, tokens: 4096, depth: 16 },
    )
    .expect("settings story fixture");
    let mut engine = jig_protocol_config::engine(&document, 0, &jig_protocol_config::Directories::working())
        .expect("settings story fixture");
    engine.charters[0].policy.instructions = Box::from(b"Answer within the granted authority.".as_slice());
    let mut config = world::config(SEED);
    config.core = jig_protocol_config::core(&engine, [31; 16], SEED);
    let mut limits = world::limits();
    limits.core = engine.limits.core;
    peers::journal(&mut limits);
    (config, limits)
}
fn start(history: u32) -> World {
    let (config, mut limits) = configuration();
    limits.core.settings.history = history;
    let mut world = World::first_start_with_accounts(config, &limits);
    world.client_peer = Some(ClientPeer::default());
    assert!(world.domain.ready());
    world
}
fn sign_in(world: &mut World) -> u64 {
    world.send(root::Event::Core(core::Event::SignIn {
        held: None,
        reply_to: ReplyTo::new(Token::new(SEED)),
        identity: people::Identity {
            key: people::IdentityKey { provider: 1, subject: Box::from(b"owner-a".as_slice()) },
            login: Box::new([]),
            name: Box::new([]),
        },
    }));
    let (
        _,
        root::Delivery::Core(core::Held::PeopleReply {
            reply: people::Reply::SignedIn { person, sign_in, .. }, ..
        }),
    ) = world.client_peer.as_mut().expect("settings story fixture").delivered.pop().expect("settings story fixture")
    else {
        panic!("owner signs in")
    };
    assert_eq!(person, OWNER);
    sign_in
}
fn ask(world: &mut World, signed: u64, key: u8, ask: people::Ask) -> people::Reply {
    world.send(root::Event::Core(core::Event::People(people::Event::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(SEED)),
        sign_in: signed,
        key: [key; 16],
        ask,
    })));
    let (_, root::Delivery::Core(core::Held::PeopleReply { reply, .. })) =
        world.client_peer.as_mut().expect("settings story fixture").delivered.pop().expect("settings story fixture")
    else {
        panic!("request reply")
    };
    reply
}
fn frame(world: &mut World, signed: u64) -> documents::Frame {
    world.send(root::Event::Core(core::Event::Read {
        reply: ReplyTo::new(Token::new(SEED)),
        sign_in: signed,
        read: reads::Read::Frame,
    }));
    let (_, root::Delivery::Core(core::Held::Read { answer: reads::Answer::Document(document), .. })) =
        world.client_peer.as_mut().expect("settings story fixture").delivered.pop().expect("settings story fixture")
    else {
        panic!("me reply")
    };
    let documents::Document::Frame(frame) = *document else { panic!("me document") };
    frame
}
fn key(key: settings::Key) -> root::Key {
    root::Key::Core(core::Key::Core(core::CoreKey::Settings(key)))
}
fn record(record: settings::Record) -> root::Record {
    root::Record::Core(core::Record::Core(core::CoreRecord::Settings(record)))
}
fn revision(world: &World, number: u32, history: bool) -> &settings::Revision {
    let address = if history { settings::Key::History(number) } else { settings::Key::Held(number) };
    let root::Record::Core(core::Record::Core(core::CoreRecord::Settings(settings::Record::Revision {
        revision, ..
    }))) = &world.store.rows[&key(address)]
    else {
        panic!("settings revision")
    };
    revision
}
fn restart(world: &mut World, config: root::Config) {
    world.restart_with(config);
    assert!(world.domain.ready());
}

#[test]
fn the_document_bridge_starts_once_and_preserves_templates_policies_rules_and_models() {
    let mut world = start(64);
    assert_eq!(revision(&world, 1, false).head.by, settings::Decider::FirstStart);
    assert!(revision(&world, 1, false).items.is_empty());
    let signed = sign_in(&mut world);
    let before = frame(&mut world, signed);
    assert_eq!(
        before.projects.get(0).expect("settings story fixture").templates.as_slice(),
        configuration().0.core.defaults.templates.as_ref()
    );
    let policy = configuration().0.core.defaults.policies.get(&1).expect("settings story fixture").policy.clone();
    let reply = ask(&mut world, signed, 1, people::Ask::MakeProject { name: Box::from("Made"), policy: 1 });
    assert!(matches!(reply, people::Reply::Outcome(people::Outcome::ProjectMade { .. })), "{reply:?}");
    let root::Record::Core(core::Record::People(people::Stored::Policy { value, .. })) =
        &world.store.rows[&root::Key::Core(core::Key::People(people::Key::Policy(2)))]
    else {
        panic!("copied project policy")
    };
    assert_eq!(value.ceiling.spend, policy.ceiling.budget.spend);
    assert_eq!(world.domain.settings().rules(), &configuration().0.core.defaults.rules);
    world.send(root::Event::Core(core::Event::Period { project: 1, period: 1, budget: 100, next: None }));
    world.send(root::Event::Core(core::Event::Fleet(fleet::Event::Hello {
        worker: 0,
        channel: Token::new(7),
        hello: fleet::Hello {
            bounds: fixtures::bounds(),
            stop_bound: Duration::from_millis(1),
            slots: 1,
            workstreams: Box::new([]),
            hosting: Box::new([]),
        },
    })));
    let chat = fixtures::chat(&world, 1, Box::from(b"hello".as_slice()));
    let reply = ask(&mut world, signed, 2, chat);
    assert!(matches!(reply, people::Reply::Outcome(people::Outcome::Created { .. })), "{reply:?}");
    let assignment = world.assigned.as_ref().expect("document charter runs");
    assert_eq!(assignment.run.policy.model.name.as_ref(), b"scripted");
    assert_eq!(assignment.run.policy.instructions.as_ref(), b"Answer within the granted authority.");
    restart(&mut world, configuration().0);
    assert_eq!(world.domain.settings().newest(), 1);
    let signed = sign_in(&mut world);
    assert_eq!(
        frame(&mut world, signed).projects.get(0).expect("settings story fixture").templates,
        before.projects.get(0).expect("settings story fixture").templates
    );
}
#[test]
fn a_release_changes_the_template_once_and_archives_its_predecessor() {
    let mut world = start(64);
    let mut config = configuration().0;
    config.core.defaults.templates[0].grace = Some(Duration::from_millis(6000));
    restart(&mut world, config);
    let newest = revision(&world, 2, false);
    assert_eq!(newest.head.by, settings::Decider::Release);
    assert_eq!(newest.head.against, 1);
    assert!(
        matches!(&newest.items[..], [settings::Item { setting: settings::Setting::Template(1), edit: settings::Edit::Set(settings::Value::Template(value)) }] if value.grace == Some(Duration::from_millis(6000)))
    );
    assert_eq!(revision(&world, 1, true).head.by, settings::Decider::FirstStart);
    let signed = sign_in(&mut world);
    assert_eq!(
        frame(&mut world, signed)
            .projects
            .get(0)
            .expect("settings story fixture")
            .templates
            .get(0)
            .expect("settings story fixture")
            .grace,
        Some(Duration::from_millis(6000))
    );
    let mut config = configuration().0;
    config.core.defaults.templates[0].grace = Some(Duration::from_millis(6000));
    restart(&mut world, config);
    assert_eq!(world.domain.settings().newest(), 2);
}
#[test]
fn a_stored_change_wins_until_reset_and_a_dropped_default_is_no_longer_offered() {
    let mut world = start(64);
    let mut template = configuration().0.core.defaults.templates[0].clone();
    template.grace = Some(Duration::from_millis(7000));
    world.seed(Box::new([record(settings::Record::Changed {
        setting: settings::Setting::Template(1),
        changed: settings::Changed { revision: 1, value: Some(settings::Value::Template(Box::new(template))) },
    })]));
    let mut config = configuration().0;
    config.core.defaults.templates[0].grace = Some(Duration::from_millis(6000));
    restart(&mut world, config);
    assert_eq!(world.domain.settings().newest(), 1);
    assert_eq!(
        world.domain.settings().template(1).expect("settings story fixture").grace,
        Some(Duration::from_millis(7000))
    );
    world.erase(key(settings::Key::Changed(settings::Setting::Template(1))));
    let mut config = configuration().0;
    config.core.defaults.templates[0].grace = Some(Duration::from_millis(6000));
    restart(&mut world, config);
    assert_eq!(world.domain.settings().newest(), 1);
    assert_eq!(
        world.domain.settings().template(1).expect("settings story fixture").grace,
        Some(Duration::from_millis(6000))
    );
    let mut config = configuration().0;
    config.core.defaults.templates = Box::new([config.core.defaults.templates[0].clone()]);
    restart(&mut world, config);
    let signed = sign_in(&mut world);
    assert_eq!(frame(&mut world, signed).projects.get(0).expect("settings story fixture").templates.len(), 1);
}
#[test]
fn history_keeps_two_predecessors_after_three_releases() {
    let mut world = start(2);
    for grace in 6..=8 {
        let mut config = configuration().0;
        config.core.defaults.templates[0].grace = Some(Duration::from_secs(grace));
        restart(&mut world, config);
    }
    assert_eq!(world.domain.settings().newest(), 4);
    assert_eq!(world.domain.settings().oldest(), 2);
    assert!(!world.store.rows.contains_key(&key(settings::Key::History(1))));
    for number in 2..=3 {
        assert_eq!(revision(&world, number, true).head.number, number);
    }
    assert_eq!(revision(&world, 4, false).head.number, 4);
}
#[test]
fn a_stored_charter_with_an_unconfigured_model_refuses_the_start_naming_the_change() {
    let mut world = start(64);
    let mut charter = configuration().0.core.defaults.charters[0].clone();
    charter.policy.model.name = Box::from(b"no-longer-configured".as_slice());
    world.seed(Box::new([record(settings::Record::Changed {
        setting: settings::Setting::Charter(1),
        changed: settings::Changed { revision: 1, value: Some(settings::Value::Charter(Box::new(charter))) },
    })]));
    world.restart_with(configuration().0);
    assert_eq!(
        world.domain.settings_failure(),
        Some(core::RestartRequest::Settings {
            failure: settings::Failure { setting: settings::Setting::Charter(1), fault: settings::Fault::Reference },
            changed: true
        })
    );
    assert!(!world.domain.ready());
    assert!(world.stopped);
}

#[test]
fn a_live_agent_task_on_a_dropped_charter_refuses_the_start_before_a_release_commits() {
    let mut world = start(64);
    let signed = sign_in(&mut world);
    world.send(root::Event::Core(core::Event::Period { project: 1, period: 1, budget: 100, next: None }));
    let chat = fixtures::chat(&world, 1, Box::from(b"still live".as_slice()));
    let reply = ask(&mut world, signed, 3, chat);
    assert!(matches!(reply, people::Reply::Outcome(people::Outcome::Created { .. })), "{reply:?}");
    let mut config = configuration().0;
    config.core.defaults.charters = Box::new([]);
    config.core.defaults.templates = Box::new([]);
    config.core.defaults.policies = skein_lib::Map::with_capacity(0);
    config.core.defaults.rules.ceiling.delegation.kinds = Box::new([]);
    world.restart_with(config);
    assert_eq!(
        world.domain.settings_failure(),
        Some(core::RestartRequest::Settings {
            failure: settings::Failure { setting: settings::Setting::Charter(1), fault: settings::Fault::Reference },
            changed: false
        })
    );
    assert!(world.stopped);
    assert_eq!(revision(&world, 1, false).head.number, 1, "refused release was never committed");
    assert!(!world.store.rows.contains_key(&key(settings::Key::Held(2))));
}

#[test]
fn changes_past_their_kinds_bound_refuse_before_the_overflow_value_is_retained() {
    let mut world = start(64);
    let config = configuration().0;
    let bound = configuration().1.core.tasks.charters;
    let extra = bound.checked_add(1).expect("one past the charter count");
    let mut rows = skein_lib::List::with_capacity(extra);
    for number in 1..=extra {
        let mut charter = config.core.defaults.charters[0].clone();
        charter.number = number;
        rows.push(record(settings::Record::Changed {
            setting: settings::Setting::Charter(number),
            changed: settings::Changed { revision: 1, value: Some(settings::Value::Charter(Box::new(charter))) },
        }))
        .expect("bounded fixture rows");
    }
    world.seed(rows.into_boxed());
    world.restart_with(config);
    assert!(!world.domain.ready());
    assert!(!world.domain.settings().changed(settings::Setting::Charter(extra)));
    assert_eq!(
        world.domain.settings_failure(),
        Some(core::RestartRequest::Settings {
            failure: settings::Failure { setting: settings::Setting::Charter(extra), fault: settings::Fault::Count },
            changed: true
        })
    );
    assert_eq!(revision(&world, 1, false).head.number, 1);
}
