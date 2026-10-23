use jig_core_world::effects::{Cut, World};
use jig_domain_core as core;
use jig_domain_people as people;
use jig_test_connector as connector;
use jig_test_domain as root;
use sha2::{Digest, Sha256};
use skein_lib::{ReplyTo, Token};

#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

fn ask(sign_in: u64, key: u8, value: connector::ApplicationRequest) -> root::Event {
    let mut bytes = Vec::new();
    bytes.extend(value.object.to_be_bytes());
    bytes.extend(value.revision.to_be_bytes());
    bytes.extend(value.value.to_be_bytes());
    match value.release {
        Some(release) => {
            bytes.push(1);
            bytes.extend(release.task.to_be_bytes());
            bytes.extend(release.revision.to_be_bytes());
            bytes.push(match release.again {
                None => 0,
                Some(false) => 1,
                Some(true) => 2,
            });
        }
        None => bytes.push(0),
    }
    root::Event::Application {
        reply_to: ReplyTo::new(Token::new(600 + u64::from(key))),
        sign_in,
        key: [key; 16],
        scope: jig_domain_people::ApplicationScope::Project(1),
        number: 2,
        digest: Sha256::digest(bytes).into(),
        value,
        carried: None,
    }
}

#[test]
fn admitted_application_routes_and_its_core_part_share_the_commit_barrier() {
    let mut world = World::new(711, false);
    let sign_in = world.sign_in.expect("signed in owner");
    let value = connector::ApplicationRequest { object: 1, revision: 0, value: 18, release: None };
    let commit = world.store.applied + 1;
    world.cut = Some(Cut::Submitted(commit));
    world.send(ask(sign_in, 1, value));
    assert_eq!(world.reached_cut, Some(Cut::Submitted(commit)));
    assert!(world.application_answers.is_empty());
    world.cut = None;
    world.release_commits();
    let changed = connector::ApplicationOutcome::Changed {
        object: 1,
        now: connector::ApplicationObject { revision: 1, value: 18 },
    };
    assert_eq!(world.application_answers[0].1, 2);
    assert_eq!(world.application_answers[0].2, changed);
    let writes = world.store.applied;
    world.send(ask(sign_in, 1, value));
    assert_eq!(world.application_answers.len(), 2);
    assert_eq!(world.application_answers[1].2, changed);
    assert_eq!(world.store.applied, writes, "replay decides nothing");

    let refused = connector::ApplicationRequest {
        object: 1,
        revision: 1,
        value: 25,
        release: Some(connector::ApplicationRelease { task: u64::MAX, revision: 1, again: None }),
    };
    world.send(ask(sign_in, 2, refused));
    assert_eq!(
        world.application_answers[2].2,
        connector::ApplicationOutcome::CoreRefused { why: connector::application_core::Refusal::Unknown }
    );
    assert!(world.store.rows.values().any(|row| matches!(row, root::Record::Connector { number: 2, record: connector::Record::ApplicationObject { object: 1, now } } if *now == connector::ApplicationObject { revision: 1, value: 18 })));
    assert!(world.store.rows.values().any(|row| matches!(
        row,
        root::Record::Core(core::Record::People(people::Stored::Answer {
            outcome: people::Outcome::Applied { connector: 2, .. },
            ..
        }))
    )));
}

#[test]
fn application_replay_after_a_durable_restart_does_not_repeat_a_decision() {
    let mut world = World::new(715, false);
    let sign_in = world.sign_in.expect("signed in owner");
    let value = connector::ApplicationRequest { object: 1, revision: 0, value: 14, release: None };
    let commit = world.store.applied + 1;
    world.cut = Some(Cut::Durable(commit));
    world.send(ask(sign_in, 11, value));
    assert_eq!(world.reached_cut, Some(Cut::Durable(commit)));
    assert!(world.application_answers.is_empty(), "durability alone has not delivered the completion");
    world.restart(715, false);
    let writes = world.store.applied;
    world.send(ask(sign_in, 11, value));
    let now = connector::ApplicationObject { revision: 1, value: 14 };
    assert_eq!(
        world.application_answers.last().expect("retained terminal").2,
        connector::ApplicationOutcome::Changed { object: 1, now }
    );
    assert_eq!(world.store.applied, writes, "restored replay decides nothing");

    let task = world.assigned.as_ref().expect("first person's task").task;
    world.send(root::Event::Core(core::Event::SignIn {
        held: None,
        reply_to: ReplyTo::new(Token::new(901)),
        identity: people::Identity {
            key: people::IdentityKey { provider: 0, subject: Box::from([8]) },
            login: Box::from(&b"other"[..]),
            name: Box::from(&b"Other"[..]),
        },
    }));
    let other_sign_in = world.sign_in.expect("signed in second party");
    assert_ne!(other_sign_in, sign_in);
    world.send(root::Event::Core(core::Event::People(people::Event::Roles {
        project: 1,
        holdings: Box::new([
            people::Holding { person: 1, role: people::Role::Owner },
            people::Holding { person: 2, role: people::Role::Member },
        ]),
    })));
    world.send(ask(
        other_sign_in,
        12,
        connector::ApplicationRequest {
            object: 1,
            revision: 1,
            value: 90,
            release: Some(connector::ApplicationRelease { task, revision: 1, again: None }),
        },
    ));
    assert_eq!(
        world.application_answers.last().expect("whole refusal").2,
        connector::ApplicationOutcome::CoreRefused { why: connector::application_core::Refusal::Unknown }
    );
    assert!(world.store.rows.values().any(|row| matches!(row, root::Record::Connector { number: 2, record: connector::Record::ApplicationObject { object: 1, now: saved } } if *saved == now)), "an unauthorized core part changes no own object");
}
