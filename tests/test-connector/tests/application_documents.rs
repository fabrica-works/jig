//! Whole connector-owned parts, observed beside their durable records.
use jig_test_connector::{self as connector, ClientDocument, ClientScope, ClientSubject, Event, Request};
use jig_test_connector_world::{World, kind, path};
use skein_lib::{Duration, Token, Wall};

fn world(seed: u8) -> World {
    let mut world = World::new(seed);
    world.event(Event::Adopt { project: 1, resource: path(seed, 1), role: connector::ResourceRole::Owned });
    world.system.other_hand(&path(seed, 1), 0);
    let fact = world.system.answer(
        connector::SystemRequest::ReadFact { resource: path(seed, 1), observed: Wall::from_nanos(1) },
        jig_test_system::Fault::None,
    );
    world.event(Event::System(fact));
    world
}

fn prepare(world: &mut World, number: u64, subject: ClientSubject) -> Token {
    let token = Token::new(number);
    world.event(Event::DocumentPrepare { token, subject });
    token
}

fn document(world: &mut World, token: Token) -> Result<ClientDocument, connector::DocumentMissing> {
    let requests = world.event(Event::Document {
        watcher: Token::new(50),
        scope: ClientScope::Project(1),
        subject: token,
        party: 1,
    });
    let [Request::Document { watcher, subject, document }] = requests.as_slice() else { panic!("one whole document") };
    assert_eq!(*watcher, Token::new(50));
    assert_eq!(*subject, token);
    document.clone()
}

fn marked(requests: &[Request], token: Token) -> bool {
    requests
        .iter()
        .any(|request| matches!(request, Request::Marked { subjects } if subjects.as_slice().contains(&token)))
}

#[test]
fn a_watched_connector_subject_is_delivered_whole_after_each_change() {
    let mut world = world(81);
    let subject = prepare(&mut world, 1, ClientSubject::Object { project: 1, object: 1 });
    assert_eq!(
        document(&mut world, subject),
        Ok(ClientDocument::Object { object: 1, now: connector::ApplicationObject { revision: 0, value: 0 } })
    );
    for (revision, value) in [(0, 13), (1, 27)] {
        let owner = Token::new(100 + revision);
        world.event(Event::ApplicationPrepare {
            owner,
            value: connector::ApplicationRequest { object: 1, revision, value, release: None },
        });
        let requests = world.event(Event::Application {
            request: owner,
            token: owner,
            owner,
            person: 1,
            project: 1,
            role: 0,
            retention: Duration::from_nanos(100),
        });
        assert!(marked(&requests, subject));
        assert_eq!(
            world.records.get(&connector::RecordKey::ApplicationObject(1)),
            Some(&connector::Record::ApplicationObject {
                object: 1,
                now: connector::ApplicationObject { revision: revision + 1, value },
            })
        );
        assert_eq!(
            document(&mut world, subject),
            Ok(ClientDocument::Object {
                object: 1,
                now: connector::ApplicationObject { revision: revision + 1, value },
            })
        );
    }
}

#[test]
fn a_panel_read_waits_for_the_commit_it_reflects() {
    let mut world = world(82);
    let panel = prepare(&mut world, 1, ClientSubject::Panel { project: 1, task: 4 });
    let requests = world.event(Event::Names { task: 4, project: 1, resources: Box::new([path(82, 1)]) });
    assert!(marked(&requests, panel));
    assert!(world.records.contains_key(&connector::RecordKey::Task(4)), "the world journals names before answering");
    let commits = world.commits;
    let requests = world.event(Event::ReadDocument { scope: ClientScope::Project(1), read: panel, party: 1 });
    let [
        Request::ReadAnswered {
            read,
            answer: Ok(ClientDocument::Panel { task, resources, procedure, result, effects }),
        },
    ] = requests.as_slice()
    else {
        panic!("whole task panel")
    };
    assert_eq!(*read, panel);
    assert_eq!(*task, 4);
    assert_eq!(resources.get(0).expect("story fixture supplies this value").path, path(82, 1));
    assert_eq!(
        resources
            .get(0)
            .expect("story fixture supplies this value")
            .fact
            .expect("story fixture supplies this value")
            .state,
        Some(0)
    );
    assert!(procedure.is_none());
    assert!(result.is_none());
    assert!(effects.is_empty());
    assert_eq!(world.commits, commits, "reading does not save or erase state");
    assert_eq!(
        document(&mut world, panel),
        Err(connector::DocumentMissing::Unknown),
        "a read consumes its owner binding"
    );
}

#[test]
fn a_connector_subject_ends_when_its_object_does() {
    let mut world = world(83);
    let token = prepare(&mut world, 1, ClientSubject::Object { project: 1, object: 1 });
    assert!(document(&mut world, token).is_ok());
    world.system.remove(&path(83, 1));
    let gone = world.system.answer(
        connector::SystemRequest::ReadFact { resource: path(83, 1), observed: Wall::from_nanos(2) },
        jig_test_system::Fault::None,
    );
    assert!(marked(&world.event(Event::System(gone)), token));
    assert_eq!(document(&mut world, token), Err(connector::DocumentMissing::Unknown));
    world.event(Event::DocumentForget { subject: token });
    world.system.other_hand(&path(83, 1), 9);
    let restored = world.system.answer(
        connector::SystemRequest::ReadFact { resource: path(83, 1), observed: Wall::from_nanos(3) },
        jig_test_system::Fault::None,
    );
    assert!(!marked(&world.event(Event::System(restored)), token), "an ended watch leaves no subject binding");
    assert_eq!(document(&mut world, token), Err(connector::DocumentMissing::Unknown));
}

#[test]
#[expect(clippy::too_many_lines, reason = "one independent owner fixture verifies every part and its lifecycle")]
fn every_part_is_owned_scoped_and_read_without_mutating_its_records() {
    let mut world = world(84);
    let watched = [
        ClientSubject::Panel { project: 1, task: 4 },
        ClientSubject::Result { project: 1, task: 4 },
        ClientSubject::Effect { project: 1, proposal: 7 },
        ClientSubject::Hold { project: 1, task: 4 },
        ClientSubject::Project { project: 1 },
        ClientSubject::Deployment,
        ClientSubject::Object { project: 1, object: 1 },
    ];
    for (index, subject) in watched.into_iter().enumerate() {
        prepare(&mut world, 40 + index as u64, subject);
    }
    world.event(Event::Names { task: 4, project: 1, resources: Box::new([path(84, 1)]) });
    let procedure = connector::ProcedureState {
        task: 4,
        number: kind(84, 1),
        resource: path(84, 1),
        index: 3,
        steps: 3,
        awaiting: false,
        deadline: Some(Wall::from_nanos(9)),
    };
    world.event(Event::Restore { record: connector::Record::Procedure(procedure.clone()) });
    world.event(Event::Restore { record: connector::Record::Result { task: 4, code: 9 } });
    let effect = connector::Effect {
        kind: kind(84, 1),
        resources: Box::new([path(84, 1)]),
        purpose: 7,
        condition: None,
        target: 5,
        state: 5,
    };
    world.event(Event::Describe { token: Token::new(80), effect: effect.clone() });
    let kept = world.event(Event::KeepProposal { token: Token::new(80), number: 7, task: 4 });
    assert!(marked(&kept, Token::new(42)), "saving a proposal marks its description");
    world.event(Event::Describe { token: Token::new(81), effect: effect.clone() });
    let kept = world.event(Event::Keep { token: Token::new(81), entry: 1, task: 4, key: world.key(4, 7) });
    for token in [40, 43, 45] {
        assert!(marked(&kept, Token::new(token)), "outbox entries mark the panel, hold and deployment");
    }
    let records = world.records.clone();
    let subjects = [
        ClientSubject::Panel { project: 1, task: 4 },
        ClientSubject::Result { project: 1, task: 4 },
        ClientSubject::Effect { project: 1, proposal: 7 },
        ClientSubject::Hold { project: 1, task: 4 },
        ClientSubject::Project { project: 1 },
        ClientSubject::Deployment,
    ];
    let mut documents = Vec::new();
    for (index, subject) in subjects.into_iter().enumerate() {
        let token = prepare(&mut world, 20 + index as u64, subject);
        let scope =
            if subject == ClientSubject::Deployment { ClientScope::Deployment } else { ClientScope::Project(1) };
        let requests = world.event(Event::ReadDocument { scope, read: token, party: 1 });
        let [Request::ReadAnswered { answer: Ok(document), .. }] = requests.as_slice() else {
            panic!("known owner part")
        };
        documents.push(document.clone());
    }
    let ClientDocument::Panel { task, procedure: now, result, resources, effects } = &documents[0] else {
        panic!("panel")
    };
    assert_eq!(*task, 4);
    assert_eq!(now.as_ref(), Some(&procedure));
    assert_eq!(*result, Some(9));
    assert_eq!(resources.len(), 1);
    assert_eq!(effects.len(), 1);
    assert_eq!(documents[1], ClientDocument::Result { task: 4, code: 9 });
    let ClientDocument::Effect { proposal, task, effect: payload, description } = &documents[2] else {
        panic!("effect")
    };
    assert_eq!((*proposal, *task), (7, 4));
    assert_eq!(payload, &effect);
    assert_eq!(description.kind, kind(84, 1));
    assert_eq!(description.price, Some(3));
    let ClientDocument::Hold { task, resources: held, effects: pending } = &documents[3] else { panic!("hold") };
    assert_eq!(*task, 4);
    assert_eq!(held, resources);
    assert_eq!(pending, effects);
    let ClientDocument::Project { project, resources: adopted } = &documents[4] else { panic!("project") };
    assert_eq!(*project, 1);
    assert_eq!(adopted, resources);
    let ClientDocument::Deployment { outbox, subject_limit, staged_limit, .. } = &documents[5] else {
        panic!("deployment")
    };
    assert_eq!(outbox, effects);
    assert_eq!(*subject_limit, jig_test_connector_world::LIMITS.values);
    assert_eq!(*staged_limit, jig_test_connector_world::LIMITS.staged);
    assert_eq!(world.records, records, "every part reads its source without new persistence");
    let token = prepare(&mut world, 30, ClientSubject::Panel { project: 2, task: 4 });
    let requests = world.event(Event::ReadDocument { scope: ClientScope::Project(2), read: token, party: 1 });
    assert!(
        matches!(requests.as_slice(), [Request::ReadAnswered { answer: Err(connector::DocumentMissing::Unknown), .. }]),
        "the typed scope must also match the actual owner record"
    );
    let ended = world.event(Event::DropProposal { number: 7 });
    assert!(marked(&ended, Token::new(42)));
    assert_eq!(document(&mut world, Token::new(42)), Err(connector::DocumentMissing::Unknown));
    let unnamed = world.event(Event::Unnamed { task: 4 });
    for token in [40, 41, 43] {
        assert!(marked(&unnamed, Token::new(token)), "forgetting a task ends each of its parts");
        assert_eq!(document(&mut world, Token::new(token)), Err(connector::DocumentMissing::Unknown));
    }
}

#[test]
fn an_outbox_part_uses_the_retained_entry_and_marks_its_removal() {
    let mut world = world(85);
    world.event(Event::Names { task: 4, project: 1, resources: Box::new([path(85, 1)]) });
    let subject = prepare(&mut world, 1, ClientSubject::Outbox { project: 1, entry: 3 });
    let effect = connector::Effect {
        kind: kind(85, 1),
        resources: Box::new([path(85, 1)]),
        purpose: 7,
        condition: None,
        target: 5,
        state: 5,
    };
    world.event(Event::Describe { token: Token::new(80), effect: effect.clone() });
    let key = world.key(4, 7);
    let kept = world.event(Event::Keep { token: Token::new(80), entry: 3, task: 4, key });
    assert!(marked(&kept, subject));
    let records = world.records.clone();
    let ClientDocument::Outbox { entry, description } = document(&mut world, subject).expect("retained entry") else {
        panic!("outbox")
    };
    assert_eq!((entry.number, entry.task, entry.key), (3, 4, key));
    assert_eq!(entry.effect, effect);
    assert_eq!(description.price, Some(3));
    assert_eq!(world.records, records, "description does not stage an effect");
    let wrong = prepare(&mut world, 2, ClientSubject::Outbox { project: 2, entry: 3 });
    let read = world.event(Event::ReadDocument { scope: ClientScope::Project(2), read: wrong, party: 1 });
    assert!(matches!(
        read.as_slice(),
        [Request::ReadAnswered { answer: Err(connector::DocumentMissing::Unknown), .. }]
    ));
    let settled = world.release(&kept, &mut std::collections::VecDeque::new());
    assert!(marked(&settled, subject));
    assert_eq!(document(&mut world, subject), Err(connector::DocumentMissing::Unknown));
}

mod deployment_diagnostics;
