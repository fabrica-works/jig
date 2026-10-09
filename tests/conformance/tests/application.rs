use jig_core as core;
use jig_core_people as people;
use jig_core_world::effects::{Cut, World};
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
        project: 1,
        number: 2,
        digest: Sha256::digest(bytes).into(),
        value,
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
