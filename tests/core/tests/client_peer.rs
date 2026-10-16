use jig_core_world::Fault;
use jig_core_world::{
    client_peer::ClientPeer,
    effects::World,
    peers::{Peers, fixture},
};
use jig_domain_core::{self as core, documents, reads};
use jig_domain_people as people;
use jig_fake_workers::{Script, Worker};
use jig_test_domain as root;
use skein_lib::{ReplyTo, Token};

fn client_world() -> (World, u64) {
    let (configuration, limits) = fixture(451, 0);
    let mut world = World::scripted(
        configuration,
        limits,
        Peers::new(vec![Worker::new(7, 1, vec![Box::new([Script::Wait, Script::Wait])])], vec![]),
    );
    world.client_peer = Some(ClientPeer::default());
    world.send(root::Event::Core(core::Event::SignIn {
        reply_to: ReplyTo::new(Token::new(45_100)),
        identity: people::Identity {
            key: people::IdentityKey { provider: 0, subject: Box::from(&b"client-peer"[..]) },
            login: Box::from(&b"client"[..]),
            name: Box::from(&b"Client"[..]),
        },
    }));
    let (
        _,
        root::Delivery::Core(core::Held::PeopleReply {
            reply: people::Reply::SignedIn { sign_in, person, .. }, ..
        }),
    ) = world.client_peer.as_mut().expect("client peer").delivered.pop().expect("sign-in terminal")
    else {
        panic!("sign-in succeeded");
    };
    world.send(root::Event::Core(core::Event::People(people::Event::Roles {
        project: 1,
        holdings: Box::new([people::Holding { person, role: people::Role::Owner }]),
    })));
    (world, sign_in)
}

fn chat(world: &mut World, sign_in: u64) {
    world.send(root::Event::Core(core::Event::People(people::Event::Ask {
        reply_to: ReplyTo::new(Token::new(45_101)),
        sign_in,
        key: [45; 16],
        ask: people::Ask::StartChat { project: 1, words: Box::from(&b"hello from a client"[..]) },
    })));
}

#[test]
fn a_client_answer_and_read_wait_for_the_engines_commit_completion() {
    let (mut world, sign_in) = client_world();
    let before = world.store.applied;
    world.store.fault(Fault::Hold { commits: 1 });
    chat(&mut world, sign_in);
    assert!(world.store.applied > before, "the store made the transaction durable");
    assert!(world.client_peer.as_ref().expect("client peer").delivered.is_empty(), "no answer skips the journal");
    world.send(root::Event::Core(core::Event::Read {
        reply: ReplyTo::new(Token::new(45_102)),
        sign_in,
        read: reads::Read::Frame,
    }));
    assert!(world.client_peer.as_ref().expect("client peer").delivered.is_empty(), "reads share the held barrier");
    let number = world.store.release_held().expect("the withheld commit completion");
    world.send(root::Event::Committed { number, fill: 0 });
    let receipts = &world.client_peer.as_ref().expect("client peer").delivered;
    assert!(receipts.iter().any(|(durable, receipt)| matches!(receipt,
        root::Delivery::Core(core::Held::PeopleReply { reply: people::Reply::Outcome(people::Outcome::Started { .. }), .. })
            if *durable >= number)));
    assert!(receipts.iter().any(|(durable, receipt)| matches!(receipt,
        root::Delivery::Core(core::Held::Read { answer: reads::Answer::Document(document), .. })
            if matches!(document.as_ref(), documents::Document::Frame(_)) && *durable >= number)));
}

#[test]
fn a_client_watch_coalesces_until_its_delivery_is_taken() {
    let (mut world, sign_in) = client_world();
    chat(&mut world, sign_in);
    let task = world.assignments.first().expect("the independent worker received a chat").0;
    world.client_peer.as_mut().expect("client peer").delivered.clear();
    let watcher = Token::new(45_103);
    world.send(root::Event::Core(core::Event::Watch {
        watcher,
        sign_in,
        subject: documents::Watched::Task { task },
        window: documents::Window { rows: 8, after: None },
    }));
    assert_eq!(world.client_peer.as_ref().expect("client peer").delivered.len(), 1);
    world.client_peer.as_mut().expect("client peer").delivered.clear();
    world.send(root::Event::Core(core::Event::People(people::Event::Ask {
        reply_to: ReplyTo::new(Token::new(45_104)),
        sign_in,
        key: [46; 16],
        ask: people::Ask::Say { project: 1, task, words: Box::from(&b"new words"[..]), release: false },
    })));
    assert!(!world.client_peer.as_ref().expect("client peer").delivered.iter().any(|(_, receipt)| matches!(receipt,
        root::Delivery::Core(core::Held::Delivery { watcher: delivered, .. }) if *delivered == watcher)));
    world.client_peer.as_mut().expect("client peer").delivered.clear();
    world.send(root::Event::Core(core::Event::Delivered { watcher }));
    assert!(world.client_peer.as_ref().expect("client peer").delivered.iter().any(|(_, receipt)| matches!(receipt,
        root::Delivery::Core(core::Held::Delivery { watcher: delivered, .. }) if *delivered == watcher)));
    world.send(root::Event::Core(core::Event::Unwatch { watcher }));
    assert!(
        !world.client_peer.as_ref().expect("client peer").delivered.iter().any(|(_, receipt)| matches!(receipt,
        root::Delivery::Core(core::Held::WatchEnded { watcher: ended, .. }) if *ended == watcher)),
        "closing preserves the outstanding delivery obligation"
    );
    world.send(root::Event::Core(core::Event::Delivered { watcher }));
    assert!(world.client_peer.as_ref().expect("client peer").delivered.iter().any(|(_, receipt)| matches!(receipt,
        root::Delivery::Core(core::Held::WatchEnded { watcher: ended, .. }) if *ended == watcher)));
}

#[test]
fn a_client_watch_refusal_is_immediate_without_a_store_commit() {
    let (mut world, _) = client_world();
    let before = world.store.applied;
    let watcher = Token::new(45_105);
    world.send(root::Event::Core(core::Event::Watch {
        watcher,
        sign_in: u64::MAX,
        subject: documents::Watched::Frame,
        window: documents::Window { rows: 8, after: None },
    }));
    assert_eq!(world.store.applied, before);
    assert!(world.client_peer.as_ref().expect("client peer").now.iter().any(|receipt| matches!(receipt,
        core::Now::WatchRefused { watcher: refused, .. } if *refused == watcher)));
}

const QUERY_WATCHER: u64 = 45_106;
const CANCEL_REPLY: u64 = 45_107;

#[test]
fn a_watched_task_query_is_delivered_again_when_its_task_ends() {
    let (mut world, sign_in) = client_world();
    chat(&mut world, sign_in);
    let task = world.assignments.first().expect("chat assignment").0;
    let requester = jig_core_world::fixtures::live_row(&world, task).requester;
    let query =
        documents::TaskQuery { project: Some(1), requester: Some(requester), ..jig_core_world::fixtures::OWN_TASKS };
    world.client_peer.as_mut().expect("configured fixture").delivered.clear();
    world.send(root::Event::Core(core::Event::Watch {
        watcher: Token::new(QUERY_WATCHER),
        sign_in,
        subject: documents::Watched::Query { query },
        window: documents::Window { rows: 8, after: None },
    }));
    let (_, root::Delivery::Core(core::Held::Delivery { document, .. })) =
        world.client_peer.as_ref().expect("configured fixture").delivered.last().expect("initial query")
    else {
        panic!("query delivery")
    };
    let documents::Document::Lines(value) = document.as_ref() else { panic!("task lines") };
    assert_eq!(value.lines.get(0).expect("configured fixture").chip.number, task);
    world.send(root::Event::Core(core::Event::Delivered { watcher: Token::new(QUERY_WATCHER) }));
    world.client_peer.as_mut().expect("configured fixture").delivered.clear();
    let before = world.store.applied;
    world.send(root::Event::Core(core::Event::People(people::Event::Ask {
        reply_to: ReplyTo::new(Token::new(CANCEL_REPLY)),
        sign_in,
        key: [47; 16],
        ask: people::Ask::Cancel { project: 1, task, reason: Box::from(&b"finished"[..]) },
    })));
    world.send(root::Event::Core(core::Event::Delivered { watcher: Token::new(QUERY_WATCHER) }));
    let (durable, root::Delivery::Core(core::Held::Delivery { document, .. })) = world
        .client_peer
        .as_ref()
        .expect("configured fixture")
        .delivered
        .iter()
        .rev()
        .find(|(_, value)| {
            matches!(value,
            root::Delivery::Core(core::Held::Delivery { watcher, .. }) if watcher.raw() == QUERY_WATCHER)
        })
        .expect("changed query")
    else {
        panic!("query delivery")
    };
    let documents::Document::Lines(value) = document.as_ref() else { panic!("task lines") };
    assert!(value.lines.is_empty());
    assert!(*durable > before, "the removal followed its commit");
}
