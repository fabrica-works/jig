//! Settings travel through the real restart, store and commit barrier (engine.md, 17.7).
use jig_core_world::{client_peer::ClientPeer, effects::World, peers, world};
use jig_domain_core::{self as core, documents, reads, settings};
use jig_domain_people as people;
use jig_test_domain as root;
use skein_lib::{Duration, ReplyTo, Token};

pub(super) const SEED: u64 = 37_001;
pub(super) const OWNER: u64 = 1;
pub(super) fn configuration() -> (root::Config, root::Limits) {
    let document = jig_protocol_config::document(
        include_bytes!("engine.json"),
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
pub(super) fn start(history: u32) -> World {
    let (config, mut limits) = configuration();
    limits.core.settings.history = history;
    let mut world = World::first_start_with_accounts(config, &limits);
    world.client_peer = Some(ClientPeer::default());
    assert!(world.domain.ready());
    world
}
pub(super) fn sign_in(world: &mut World) -> u64 {
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
pub(super) fn ask(world: &mut World, signed: u64, key: u8, ask: people::Ask) -> people::Reply {
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
pub(super) fn frame(world: &mut World, signed: u64) -> documents::Frame {
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
pub(super) fn key(key: settings::Key) -> root::Key {
    root::Key::Core(core::Key::Core(core::CoreKey::Settings(key)))
}
pub(super) fn record(record: settings::Record) -> root::Record {
    root::Record::Core(core::Record::Core(core::CoreRecord::Settings(record)))
}
pub(super) fn revision(world: &World, number: u32, history: bool) -> &settings::Revision {
    let address = if history { settings::Key::History(number) } else { settings::Key::Held(number) };
    let root::Record::Core(core::Record::Core(core::CoreRecord::Settings(settings::Record::Revision {
        revision, ..
    }))) = &world.store.rows[&key(address)]
    else {
        panic!("settings revision")
    };
    revision
}
pub(super) fn restart(world: &mut World, config: root::Config) {
    world.restart_with(config);
    assert!(world.domain.ready());
}

pub(super) fn change(
    world: &mut World,
    signed: u64,
    key_number: u8,
    against: u32,
    items: Box<[settings::Item]>,
) -> people::Reply {
    world.send(root::Event::Core(core::Event::ChangeSettings {
        reply_to: ReplyTo::new(Token::new(SEED)),
        sign_in: signed,
        key: [key_number; 16],
        digest: [key_number; 32],
        change: Box::new(settings::Change { against, items, running: settings::Running::Stays }),
    }));
    let (_, root::Delivery::Core(core::Held::PeopleReply { reply, .. })) =
        world.client_peer.as_mut().expect("client").delivered.pop().expect("settings reply")
    else {
        panic!("settings reply")
    };
    reply
}
pub(super) fn template_edit(world: &World, grace: u64) -> settings::Item {
    let mut value = world.domain.settings().template(1).expect("chat template").clone();
    value.grace = Some(Duration::from_secs(grace));
    settings::Item {
        setting: settings::Setting::Template(1),
        edit: settings::Edit::Set(settings::Value::Template(Box::new(value))),
    }
}
pub(super) fn changed(reply: &people::Reply, revision: u32) {
    assert_eq!(reply, &people::Reply::Outcome(people::Outcome::SettingsChanged { revision }));
}

pub(super) fn settings_rows(world: &World) -> Vec<(root::Key, root::Record)> {
    world
        .store
        .rows
        .iter()
        .filter(|(key, _)| matches!(key, root::Key::Core(core::Key::Core(core::CoreKey::Settings(_)))))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect()
}
