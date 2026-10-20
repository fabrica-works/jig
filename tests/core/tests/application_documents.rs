//! Connector values through the real root and independently observed store.
use jig_core_world::{
    effects::{Cut, World},
    reads, watches,
};
use jig_domain_core as core;
use jig_domain_people::ApplicationScope;
use jig_test_connector::{self as connector, ClientDocument, ClientSubject};
use jig_test_domain as root;
use skein_lib::{ReplyTo, Token};

const PROJECT: u32 = 1;
const CONNECTOR: u16 = 1;
const OBJECT: u64 = 1;
const OWN_REQUEST: u64 = 40001;
const OWN_KEY: u8 = 17;

const OWN_WATCHER: u64 = 56;
const DEPLOYMENT_WATCHER: u64 = 57;
const UNKNOWN_SUBJECT: u64 = 900;

fn world(seed: u64) -> World {
    let mut world = World::new(seed, false);
    world.watches = Some(watches::Parties::new(seed, world.store.applied, false, world.sign_in.expect("signed in")));
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
        scope: jig_domain_people::ApplicationScope::Project(1),
        number: 1,
        digest: [revision + 1; 32],
        value: connector::ApplicationRequest { object: 1, revision: u64::from(revision), value, release: None },
        carried: None,
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
    world.store.fault(jig_core_world::Fault::Hold { commits: 1 });
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
    world.send(root::Event::Committed { number: durable, fill: 0 });
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
            .contains(&(51, jig_domain_views::End::Gone))
    );
    assert_eq!(received(&world, 51).len(), 1, "the missing object never produces another document");
    world.watches.as_ref().expect("story fixture supplies this value").finish();
}

#[test]
fn an_own_scope_watch_is_admitted_on_the_sign_in_alone() {
    let mut world = world(884);
    world.send(root::Event::Core(core::Event::People(jig_domain_people::Event::Roles {
        project: 1,
        holdings: Box::new([]),
    })));
    watches::application(&mut world, OWN_WATCHER, ApplicationScope::Own, 1, ClientSubject::Deployment);
    let [ClientDocument::Deployment { outbox, .. }] = received(&world, OWN_WATCHER).as_slice() else {
        panic!("own document")
    };
    assert!(outbox.is_empty());
    world.watches.as_ref().expect("watch parties").finish();
}

#[test]
fn a_deployment_scope_watch_without_a_deployment_role_is_refused() {
    let mut world = world(885);
    let env = skein_lib::Env {
        now: skein_lib::Time::ZERO,
        wall: skein_lib::Wall::EPOCH,
        limits: jig_core_world::effects::fixture(885, false).1,
    };
    root::step(
        &mut world.domain,
        &env,
        root::Event::Core(core::Event::Watch {
            watcher: Token::new(DEPLOYMENT_WATCHER),
            sign_in: world.sign_in.expect("signed in"),
            subject: core::documents::Watched::Application {
                scope: ApplicationScope::Deployment,
                connector: 1,
                subject: Token::new(UNKNOWN_SUBJECT),
            },
            window: core::documents::Window { rows: 1, after: None },
        }),
    );
    let mut out = skein_lib::Queue::with_capacity(16);
    root::release(&mut world.domain, &env, &mut out);
    let mut refusals = Vec::new();
    while let Some(request) = out.pop() {
        if let root::Request::Now(core::Now::WatchRefused { watcher, why }) = request {
            refusals.push((watcher, why));
        }
    }
    assert_eq!(refusals, [(Token::new(DEPLOYMENT_WATCHER), core::WatchRefusal::NotAllowed)]);
}

#[test]
fn an_application_read_carries_its_window_to_the_connector() {
    let mut world = world(886);
    let path = jig_test_connector_world::path(1, 2);
    world.send(root::Event::Connector {
        number: 1,
        event: connector::Event::Adopt { project: 1, resource: path.clone(), role: connector::ResourceRole::Owned },
    });
    reads::application_window(
        &mut world,
        1,
        ApplicationScope::Project(1),
        1,
        ClientSubject::Project { project: 1 },
        core::documents::Window { rows: 1, after: Some(core::documents::Cursor::Number { number: 1 }) },
    );
    let receipt = &world.reads.as_ref().expect("read parties").application_receipts[0];
    let Ok(ClientDocument::Project { resources, .. }) = &receipt.3 else { panic!("project rows") };
    assert_eq!(resources.len(), 1);
    assert_eq!(resources.get(0).expect("second resource").path, path);
}

#[test]
fn an_own_scope_application_request_reaches_its_connector_with_no_role() {
    let mut world = world(887);
    world.send(root::Event::Core(core::Event::People(jig_domain_people::Event::Roles {
        project: PROJECT,
        holdings: Box::new([]),
    })));
    world.send(root::Event::Application {
        reply_to: ReplyTo::new(Token::new(OWN_REQUEST)),
        sign_in: world.sign_in.expect("signed in"),
        key: [OWN_KEY; 16],
        scope: ApplicationScope::Own,
        number: CONNECTOR,
        digest: [8; 32],
        value: connector::ApplicationRequest { object: OBJECT, revision: 0, value: 42, release: None },
        carried: None,
    });
    assert_eq!(
        world.application_answers,
        vec![(
            Token::new(OWN_REQUEST),
            CONNECTOR,
            connector::ApplicationOutcome::Changed {
                object: OBJECT,
                now: connector::ApplicationObject { revision: 1, value: 42 }
            }
        )]
    );
    world.watches.as_ref().expect("watch parties").finish();
}

#[test]
fn a_task_document_keeps_a_held_name_past_the_short_list_bound() {
    let (mut world, task, path) = jig_core_world::fixtures::long_resource_world(886);
    reads::ask(&mut world, OWN_REQUEST, core::reads::Read::Task { task });
    let core::reads::Answer::Document(document) = reads::latest(&world) else { panic!("task document") };
    let core::documents::Document::Task(value) = document.as_ref() else { panic!("task document") };
    let resource = value.resources.get(0).expect("held resource");
    let core::documents::PartKey::Path(segments) = &resource.part.key else { panic!("resource path") };
    assert_eq!(segments.as_slice(), path.segments());
}

const CARRIED: u64 = 1 << 63;

/// Sends the own-scope request under its one key, carrying a kept value,
/// and returns how many answers were delivered before it.
fn carry(world: &mut World, reply: u64, digest: u8, number: u64) -> usize {
    let before = world.people_answers.len() + world.application_answers.len();
    world.send(root::Event::Application {
        reply_to: ReplyTo::new(Token::new(reply)),
        sign_in: world.sign_in.expect("signed in"),
        key: [OWN_KEY; 16],
        scope: ApplicationScope::Own,
        number: CONNECTOR,
        digest: [digest; 32],
        value: connector::ApplicationRequest { object: OBJECT, revision: 0, value: 42, release: None },
        carried: Some(core::Carried { number, digest: [digest; 32] }),
    });
    before
}

#[test]
fn a_carried_value_the_core_leaves_unused_is_named_before_its_answer() {
    let mut world = world(888);
    let changed = connector::ApplicationOutcome::Changed {
        object: OBJECT,
        now: connector::ApplicationObject { revision: 1, value: 42 },
    };
    carry(&mut world, OWN_REQUEST, 8, CARRIED);
    assert!(world.carried_unused.is_empty(), "an admitted request uses its value");
    let replayed = carry(&mut world, OWN_REQUEST + 1, 8, CARRIED + 1);
    assert_eq!(
        world.application_answers,
        vec![(Token::new(OWN_REQUEST), CONNECTOR, changed), (Token::new(OWN_REQUEST + 1), CONNECTOR, changed)],
        "the retained answer is replayed"
    );
    let conflicting = carry(&mut world, OWN_REQUEST + 2, 9, CARRIED + 2);
    assert_eq!(
        world.people_answers.last(),
        Some(&jig_domain_people::Reply::Refused(jig_domain_people::Refusal::KeyConflict))
    );
    world.stop();
    let stopping = carry(&mut world, OWN_REQUEST + 3, 8, CARRIED + 3);
    assert_eq!(world.people_answers.last(), Some(&jig_domain_people::Reply::Again(jig_domain_people::Again::NotReady)));
    assert_eq!(
        world.carried_unused,
        vec![(replayed, CARRIED + 1), (conflicting, CARRIED + 2), (stopping, CARRIED + 3)],
        "each unused value is named once, before its request's answer"
    );
}
