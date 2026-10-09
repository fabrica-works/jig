//! Connector values through the real root and independently observed store.
use jig_core::{self as core, documents::ApplicationScope};
use jig_core_world::{
    effects::{Cut, World},
    reads, watches,
};
use jig_test_connector::{self as connector, ClientDocument, ClientSubject};
use jig_test_domain as root;
use skein_lib::{ReplyTo, Token};

fn world(seed: u64) -> World {
    let mut world = World::new(seed, false);
    world.watches = Some(watches::Parties::new(seed, world.store.applied, false));
    world.reads = Some(reads::Parties::new(world.store.applied));
    world.send(root::Event::Connector {
        number: 1,
        event: connector::Event::Adopt {
            project: 1,
            resource: jig_test_connector_world::path(1, 1),
            role: connector::ResourceRole::Owned,
        },
    });
    let path = jig_test_connector_world::path(1, 1);
    world.systems[0].other_hand(&path, 0);
    let present = world.systems[0].answer(
        connector::SystemRequest::ReadFact { resource: path, observed: skein_lib::Wall::from_nanos(1) },
        jig_test_system::Fault::None,
    );
    world.send(root::Event::Connector { number: 1, event: connector::Event::System(present) });
    world
}

fn change(world: &mut World, revision: u8, value: u64) {
    world.send(root::Event::Application {
        reply_to: ReplyTo::new(Token::new(40_000 + u64::from(revision))),
        sign_in: world.sign_in.expect("story fixture supplies this value"),
        key: [revision + 1; 16],
        project: 1,
        number: 1,
        digest: [revision + 1; 32],
        value: connector::ApplicationRequest { object: 1, revision: u64::from(revision), value, release: None },
    });
}

fn received(world: &World, watcher: u64) -> Vec<&ClientDocument> {
    world
        .watches
        .as_ref()
        .expect("story fixture supplies this value")
        .application_deliveries
        .iter()
        .filter_map(|(id, _, number, document)| {
            assert_eq!(*number, 1);
            (*id == watcher).then_some(document)
        })
        .collect()
}

#[test]
fn a_watched_connector_subject_is_delivered_whole_after_each_change() {
    let mut world = world(881);
    let subject = ClientSubject::Object { project: 1, object: 1 };
    for watcher in [51, 52] {
        watches::application(&mut world, watcher, ApplicationScope::Project(1), 1, subject);
    }
    assert_eq!(
        received(&world, 51),
        [&ClientDocument::Object { object: 1, now: connector::ApplicationObject { revision: 0, value: 0 } }]
    );
    world.send(root::Event::Core(core::Event::Delivered { watcher: Token::new(51) }));
    for (revision, value) in [(0, 11), (1, 22)] {
        world.store_delay = 3;
        change(&mut world, revision, value);
        assert_eq!(
            received(&world, 51).last().expect("story fixture supplies this value"),
            &&ClientDocument::Object {
                object: 1,
                now: connector::ApplicationObject { revision: u64::from(revision) + 1, value }
            }
        );
        world.send(root::Event::Core(core::Event::Delivered { watcher: Token::new(51) }));
    }
    assert_eq!(received(&world, 51).len(), 3);
    assert_eq!(received(&world, 52).len(), 1, "a slow party keeps one delivery in flight");
    world.send(root::Event::Core(core::Event::Delivered { watcher: Token::new(52) }));
    assert_eq!(
        received(&world, 52),
        [
            &ClientDocument::Object { object: 1, now: connector::ApplicationObject { revision: 0, value: 0 } },
            &ClientDocument::Object { object: 1, now: connector::ApplicationObject { revision: 2, value: 22 } }
        ]
    );
    world.watches.as_ref().expect("story fixture supplies this value").finish();
}

#[test]
fn a_panel_read_waits_for_the_commit_it_reflects() {
    let mut world = world(882);
    let task = world.assigned.as_ref().expect("story fixture supplies this value").task;
    let path = jig_test_connector_world::path(1, 2);
    world.send(root::Event::Connector {
        number: 1,
        event: connector::Event::Adopt { project: 1, resource: path.clone(), role: connector::ResourceRole::Owned },
    });
    let commit = world.store.applied + 1;
    world.store.fault(jig_fake_store::Fault::Hold { commits: 1 });
    world.cut = Some(Cut::Submitted(commit));
    world.send(root::Event::Connector {
        number: 1,
        event: connector::Event::Names { task, project: 1, resources: Box::new([path.clone()]) },
    });
    assert_eq!(world.reached_cut, Some(Cut::Submitted(commit)));
    assert!(world.store.applied < commit);
    watches::application(&mut world, 54, ApplicationScope::Project(1), 1, ClientSubject::Panel { project: 1, task });
    assert!(received(&world, 54).is_empty(), "the watched panel also waits for the owner commit");
    reads::application(&mut world, 1, ApplicationScope::Project(1), 1, ClientSubject::Panel { project: 1, task });
    assert!(world.reads.as_ref().expect("story fixture supplies this value").application_receipts.is_empty());
    let durable = world.store.release_held().expect("story fixture supplies this value");
    world.send(root::Event::Committed { number: durable });
    let ClientDocument::Panel { resources: watched_resources, .. } = received(&world, 54)[0] else {
        panic!("watched connector panel")
    };
    assert_eq!(watched_resources.get(0).expect("named resource").path, path);
    let receipt = &world.reads.as_ref().expect("story fixture supplies this value").application_receipts[0];
    assert_eq!(receipt.1, commit);
    let Ok(ClientDocument::Panel { task: named, resources, .. }) = &receipt.3 else { panic!("connector task panel") };
    assert_eq!(*named, task);
    assert_eq!(resources.len(), 1);
    assert_eq!(resources.get(0).expect("story fixture supplies this value").path, path);
    reads::application(&mut world, 2, ApplicationScope::Project(1), 1, ClientSubject::Panel { project: 2, task });
    assert_eq!(
        world.reads.as_ref().expect("story fixture supplies this value").application_receipts[1].3,
        Err(connector::DocumentMissing::Unknown)
    );
    world.restart(882, false);
    assert_eq!(
        world.reads.as_ref().expect("story fixture supplies this value").application_receipts.len(),
        2,
        "the party keeps its durable answers across restart"
    );
}

#[test]
fn a_connector_subject_ends_when_its_object_does() {
    let mut world = world(883);
    watches::application(
        &mut world,
        51,
        ApplicationScope::Project(1),
        1,
        ClientSubject::Object { project: 1, object: 1 },
    );
    world.send(root::Event::Core(core::Event::Delivered { watcher: Token::new(51) }));
    let path = jig_test_connector_world::path(1, 1);
    world.systems[0].remove(&path);
    let gone = world.systems[0].answer(
        connector::SystemRequest::ReadFact { resource: path.clone(), observed: skein_lib::Wall::from_nanos(100) },
        jig_test_system::Fault::None,
    );
    world.send(root::Event::Connector { number: 1, event: connector::Event::System(gone) });
    assert_eq!(world.systems[0].state(&path), None);
    assert!(
        world
            .watches
            .as_ref()
            .expect("story fixture supplies this value")
            .endings
            .contains(&(51, jig_core_views::End::Gone))
    );
    assert_eq!(received(&world, 51).len(), 1, "the missing object never produces another document");
    world.watches.as_ref().expect("story fixture supplies this value").finish();
}
