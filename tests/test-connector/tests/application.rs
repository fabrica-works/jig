use jig_test_connector as connector;
use jig_test_connector_world::World;
use sha2::{Digest, Sha256};
use skein_lib::{Duration, Token};

fn decide(world: &mut World, token: u64, role: u32, value: connector::ApplicationRequest) -> Vec<connector::Request> {
    let mut bytes = Vec::new();
    bytes.extend(value.object.to_be_bytes());
    bytes.extend(value.revision.to_be_bytes());
    bytes.extend(value.value.to_be_bytes());
    if let Some(release) = value.release {
        bytes.push(1);
        bytes.extend(release.task.to_be_bytes());
        bytes.extend(release.revision.to_be_bytes());
        bytes.push(match release.again {
            None => 0,
            Some(false) => 1,
            Some(true) => 2,
        });
    } else {
        bytes.push(0);
    }
    let digest = Sha256::digest(bytes);
    let owner = Token::new(u64::from_be_bytes(digest[..8].try_into().expect("digest prefix")));
    world.event(connector::Event::ApplicationPrepare { owner, value });
    world.event(connector::Event::Application {
        request: Token::new(token),
        token: Token::new(token),
        owner,
        person: 1,
        scope: connector::ClientScope::Project(1),
        role: Some(role),
        retention: Duration::from_nanos(100),
    })
}

#[test]
fn an_application_request_naming_a_moved_object_is_refused_by_its_connector() {
    let mut world = World::new(71);
    let value = connector::ApplicationRequest { object: 1, revision: 0, value: 12, release: None };
    decide(&mut world, 1, 2, value);
    let now = connector::ApplicationObject { revision: 1, value: 12 };
    assert_eq!(
        world.domain.application_outcome(Token::new(1)),
        Some(connector::ApplicationOutcome::Changed { object: 1, now })
    );
    let stale = decide(&mut world, 2, 2, connector::ApplicationRequest { value: 19, ..value });
    assert_eq!(
        world.domain.application_outcome(Token::new(2)),
        Some(connector::ApplicationOutcome::Moved { object: 1, now })
    );
    assert!(!stale.iter().any(|request| matches!(
        request,
        connector::Request::Save { record: connector::Record::ApplicationObject { .. } }
    )));
    decide(&mut world, 3, 3, connector::ApplicationRequest { revision: 1, ..value });
    assert_eq!(world.domain.application_outcome(Token::new(3)), Some(connector::ApplicationOutcome::Role { held: 3 }));
}

#[test]
fn an_application_request_sent_again_after_a_restart_is_given_its_kept_outcome() {
    let mut world = World::new(72);
    decide(&mut world, 7, 0, connector::ApplicationRequest { object: 1, revision: 0, value: 18, release: None });
    let outcome = world.domain.application_outcome(Token::new(7));
    world.cold_restart();
    let commits = world.commits;
    assert_eq!(world.domain.application_outcome(Token::new(7)), outcome);
    assert_eq!(world.commits, commits, "a replay is a lookup of the saved terminal");
    assert_eq!(
        world.records.values().filter(|record| matches!(record, connector::Record::ApplicationObject { .. })).count(),
        1
    );
}

#[test]
fn a_request_whose_core_part_the_party_may_not_make_is_refused_whole() {
    let mut world = World::new(73);
    let requests = decide(
        &mut world,
        1,
        2,
        connector::ApplicationRequest {
            object: 1,
            revision: 0,
            value: 18,
            release: Some(connector::ApplicationRelease { task: 4, revision: 1, again: None }),
        },
    );
    assert!(requests.iter().any(|request| matches!(request, connector::Request::ApplicationRelease { .. })));
    assert!(world.domain.application_outcome(Token::new(1)).is_none());
    let why = connector::application_core::Refusal::Role { held: Some(2) };
    world.event(connector::Event::ApplicationCore {
        request: Token::new(1),
        answer: connector::ApplicationCoreAnswer::Refused(why),
    });
    assert_eq!(
        world.domain.application_outcome(Token::new(1)),
        Some(connector::ApplicationOutcome::CoreRefused { why })
    );
    assert!(!world.records.values().any(|record| matches!(record, connector::Record::ApplicationObject { .. })));
    world.cold_restart();
    assert_eq!(
        world.domain.application_outcome(Token::new(1)),
        Some(connector::ApplicationOutcome::CoreRefused { why })
    );
}
