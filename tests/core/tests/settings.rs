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
    ready_host(&mut world);
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

fn client_read(world: &mut World, signed: u64, read: reads::Read) -> reads::Answer {
    world.send(root::Event::Core(core::Event::Read { reply: ReplyTo::new(Token::new(SEED)), sign_in: signed, read }));
    let (_, root::Delivery::Core(core::Held::Read { answer, .. })) =
        world.client_peer.as_mut().expect("settings story fixture").delivered.pop().expect("read terminal")
    else {
        panic!("settings read answer")
    };
    answer
}
fn inventory(world: &mut World, signed: u64) -> documents::Settings {
    let reads::Answer::Document(document) = client_read(world, signed, reads::Read::Settings) else {
        panic!("settings inventory")
    };
    let documents::Document::Settings(document) = *document else { panic!("settings document") };
    *document
}
fn released_history() -> World {
    let (config, mut limits) = configuration();
    limits.core.settings.history = 2;
    limits.core.documents.rows = 2;
    peers::journal(&mut limits);
    let mut world = World::first_start_with_accounts(config, &limits);
    world.client_peer = Some(ClientPeer::default());
    for grace in 6..=10 {
        let mut config = configuration().0;
        config.core.defaults.templates[0].grace = Some(Duration::from_secs(grace));
        restart(&mut world, config);
    }
    world
}
#[test]
fn settings_inventory_tracks_each_defaults_revision_across_a_release() {
    let mut world = start(2);
    let signed = sign_in(&mut world);
    let first = inventory(&mut world, signed);
    assert_eq!(first.newest, 1);
    assert!(first.settings.iter().all(|line| line.follows == documents::Follows::Default && line.last == 1));
    assert_eq!(first.held.get(0).expect("settings story fixture").holders, 0);
    let mut config = configuration().0;
    config.core.defaults.templates[0].grace = Some(Duration::from_secs(6));
    restart(&mut world, config);
    let signed = sign_in(&mut world);
    let second = inventory(&mut world, signed);
    assert_eq!(second.newest, 2);
    for line in &second.settings {
        assert_eq!(line.last, if line.setting == settings::Setting::Template(1) { 2 } else { 1 });
    }
}
#[test]
fn settings_inventory_is_unknown_without_a_deployment_role() {
    let mut world = jig_core_world::stories::reads::world(SEED);
    jig_core_world::reads::ask(&mut world, 1, reads::Read::Settings);
    assert_eq!(jig_core_world::reads::latest(&world), &reads::Answer::None(reads::Why::Unknown));
    jig_core_world::reads::ask(&mut world, 2, reads::Read::Deployment);
    assert_eq!(jig_core_world::reads::latest(&world), &reads::Answer::None(reads::Why::Unknown));
}
#[test]
fn a_setting_reads_whole_only_from_the_newest() {
    let mut world = released_history();
    let signed = sign_in(&mut world);
    let reads::Answer::Setting(value) =
        client_read(&mut world, signed, reads::Read::Setting { setting: settings::Setting::Template(1), revision: 6 })
    else {
        panic!("whole template")
    };
    let settings::Value::Template(template) = value.value else { panic!("template value") };
    assert_eq!(template.grace, Some(Duration::from_secs(10)));
    assert_eq!(
        client_read(&mut world, signed, reads::Read::Setting { setting: settings::Setting::Template(1), revision: 5 }),
        reads::Answer::None(reads::Why::Unknown)
    );
    assert_eq!(
        client_read(
            &mut world,
            signed,
            reads::Read::Setting { setting: settings::Setting::Template(999), revision: 6 }
        ),
        reads::Answer::None(reads::Why::Unknown)
    );
}
#[test]
fn revision_reads_load_held_and_history_records_and_name_the_forgotten_cutoff() {
    let mut world = released_history();
    let signed = sign_in(&mut world);
    for number in [6, 5, 4] {
        let reads::Answer::Revision(value) = client_read(&mut world, signed, reads::Read::Revision { number }) else {
            panic!("revision read")
        };
        let reads::RevisionAnswer::Revision { revision, .. } = *value else { panic!("kept revision") };
        assert_eq!(revision.head.number, number);
        assert_eq!(revision.head.by, settings::Decider::Release);
        assert_eq!(revision.items.len(), 1);
    }
    assert_eq!(
        client_read(&mut world, signed, reads::Read::Revision { number: 3 }),
        reads::Answer::Revision(Box::new(reads::RevisionAnswer::Forgotten { oldest: 4 }))
    );
    assert_eq!(
        client_read(&mut world, signed, reads::Read::Revision { number: 7 }),
        reads::Answer::Revision(Box::new(reads::RevisionAnswer::Unknown))
    );
    world.erase(key(settings::Key::History(5)));
    assert_eq!(
        client_read(&mut world, signed, reads::Read::Revision { number: 5 }),
        reads::Answer::Revision(Box::new(reads::RevisionAnswer::Forgotten { oldest: 4 }))
    );
}
#[test]
fn revisions_pages_run_newest_first_and_end_with_forgotten() {
    let mut world = released_history();
    let signed = sign_in(&mut world);
    let reads::Answer::Revisions(first) = client_read(&mut world, signed, reads::Read::Revisions { before: None })
    else {
        panic!("history page")
    };
    assert_eq!(first.rows.iter().map(|row| row.head.number).collect::<Vec<_>>(), [6, 5]);
    assert_eq!(first.next, Some(5));
    assert!(!first.forgotten);
    let reads::Answer::Revisions(last) = client_read(&mut world, signed, reads::Read::Revisions { before: first.next })
    else {
        panic!("last history page")
    };
    assert_eq!(last.rows.iter().map(|row| row.head.number).collect::<Vec<_>>(), [4]);
    assert_eq!(last.next, None);
    assert!(last.forgotten);
}
fn ready_host(world: &mut World) {
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
}
fn watched_holders(world: &World) -> Vec<u32> {
    world
        .client_peer
        .as_ref()
        .expect("settings story fixture")
        .delivered
        .iter()
        .filter_map(|(_, delivery)| match delivery {
            root::Delivery::Core(core::Held::Delivery { document, .. }) => match document.as_ref() {
                documents::Document::Settings(value) => {
                    Some(value.held.get(0).expect("settings story fixture").holders)
                }
                _ => None,
            },
            _ => None,
        })
        .collect()
}
#[test]
fn settings_watch_marks_when_an_agent_task_is_added_and_ends() {
    const WATCHER: Token = Token::new(37_002);
    let mut world = start(2);
    let signed = sign_in(&mut world);
    world.send(root::Event::Core(core::Event::Watch {
        watcher: WATCHER,
        sign_in: signed,
        subject: documents::Watched::Settings,
        window: documents::Window { rows: 1, after: None },
    }));
    assert_eq!(watched_holders(&world), [0]);
    world.send(root::Event::Core(core::Event::Delivered { watcher: WATCHER }));
    world.send(root::Event::Core(core::Event::Period { project: 1, period: 1, budget: 100, next: None }));
    ready_host(&mut world);
    let chat = fixtures::chat(&world, 1, Box::from(b"hello".as_slice()));
    world.send(root::Event::Core(core::Event::People(people::Event::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(SEED)),
        sign_in: signed,
        key: [61; 16],
        ask: chat,
    })));
    assert_eq!(watched_holders(&world), [0, 1]);
    world.send(root::Event::Core(core::Event::Delivered { watcher: WATCHER }));
    let task = world.assigned.as_ref().expect("live agent task").task;
    world.send(root::Event::Core(core::Event::People(people::Event::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(SEED)),
        sign_in: signed,
        key: [62; 16],
        ask: people::Ask::Cancel { project: 1, task, reason: Box::from(b"finished".as_slice()), grace: None },
    })));
    world.end(jig_domain_tasks::End::Parked, 0);
    assert_eq!(watched_holders(&world), [0, 1, 0]);
}

#[test]
fn a_removed_setting_keeps_its_inventory_line_and_reads_unknown() {
    let mut world = start(2);
    world.seed(Box::new([record(settings::Record::Changed {
        setting: settings::Setting::Template(2),
        changed: settings::Changed { value: None, revision: 1 },
    })]));
    restart(&mut world, configuration().0);
    let signed = sign_in(&mut world);
    let document = inventory(&mut world, signed);
    let line = document
        .settings
        .iter()
        .find(|line| line.setting == settings::Setting::Template(2))
        .expect("settings story fixture");
    assert_eq!(line.follows, documents::Follows::Change);
    assert_eq!(line.last, 1);
    assert_eq!(
        client_read(&mut world, signed, reads::Read::Setting { setting: settings::Setting::Template(2), revision: 1 }),
        reads::Answer::None(reads::Why::Unknown)
    );
}
#[test]
fn revision_item_previews_state_how_many_were_cut() {
    let (config, mut limits) = configuration();
    limits.core.documents.lines = 1;
    let mut world = World::first_start_with_accounts(config, &limits);
    world.client_peer = Some(ClientPeer::default());
    let mut config = configuration().0;
    for template in &mut config.core.defaults.templates {
        template.name = Box::from(b"new name".as_slice());
    }
    restart(&mut world, config);
    let signed = sign_in(&mut world);
    let reads::Answer::Revisions(page) = client_read(&mut world, signed, reads::Read::Revisions { before: None })
    else {
        panic!("revision page")
    };
    let row = page.rows.get(0).expect("settings story fixture");
    assert_eq!(row.head.number, 2);
    assert_eq!(row.items.len(), 1);
    assert_eq!(row.more, 1);
    assert_eq!(page.next, None);
    assert!(!page.forgotten);
}
#[test]
fn revision_loads_answer_busy_when_the_load_slab_is_full() {
    let mut world = start(2);
    let signed = sign_in(&mut world);
    world.sign_in = Some(signed);
    world.client_peer = None;
    let mut parties = jig_core_world::reads::Parties::new(world.store.applied);
    parties.hold_loads = true;
    world.reads = Some(parties);
    // The small profile has four load slots (configuration.md, section 6).
    for number in 1..=5 {
        jig_core_world::reads::ask(&mut world, number, reads::Read::Revision { number: 1 });
    }
    assert_eq!(jig_core_world::reads::latest(&world), &reads::Answer::None(reads::Why::Busy));
    jig_core_world::reads::complete(&mut world);
    assert_eq!(world.reads.as_ref().expect("settings story fixture").receipts.len(), 5);
}

#[test]
fn a_revision_item_past_the_single_read_budget_answers_its_encoded_size() {
    use jig_client_protocol_wire::v19 as w;
    use jig_protocol_engine_clients::translate::settings as wire;
    const SINGLE_READ_BYTES: u64 = 4096;
    const NAME_BYTES: usize = 4096;
    // v19 SettingValue tag + this fixture's Template: 46 fixed bytes + its 4096-byte name.
    const ENCODED_TEMPLATE_BYTES: u64 = 4142;
    let (config, mut limits) = configuration();
    limits.core.load_bytes = SINGLE_READ_BYTES;
    let mut world = World::first_start_with_accounts(config, &limits);
    world.client_peer = Some(ClientPeer::default());
    let mut config = configuration().0;
    config.core.defaults.templates[0].name = vec![b'x'; NAME_BYTES].into_boxed_slice();
    restart(&mut world, config);
    let signed = sign_in(&mut world);
    let reads::Answer::Revision(answer) = client_read(&mut world, signed, reads::Read::Revision { number: 2 }) else {
        panic!("revision read")
    };
    let reads::RevisionAnswer::Revision { value_bytes, .. } = answer.as_ref() else { panic!("revision") };
    assert_eq!(*value_bytes, SINGLE_READ_BYTES);
    let w::RevisionAnswer::Revision(document) = wire::revision(&answer, &[], &w::CEILINGS).expect("revision answer")
    else {
        panic!("revision record")
    };
    let mut pieces = skein_lib::Reader::new(document.items());
    let length = pieces.u32().expect("item length");
    let bytes = pieces.bytes(length).expect("item bytes");
    let item = w::RevisionItem::decode(&w::CEILINGS, &mut skein_lib::Reader::new(bytes)).expect("revision item");
    let w::RevisionEdit::TooLarge(size) = item.edit() else { panic!("item size") };
    assert_eq!(size.bytes(), ENCODED_TEMPLATE_BYTES);
}
