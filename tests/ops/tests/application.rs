use jig_conformance::Harness;
use jig_ops_domain as root;
use jig_ops_domain_infrastructure as infrastructure;
use jig_ops_world::conformance::{Config, Ops, Story};
use sha2::{Digest, Sha256};
use skein_lib::{ReplyTo, Token};

#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

fn request(sign_in: u64) -> root::Event {
    let value = infrastructure::ApplicationRequest {
        environment: infrastructure::Environment::new(b"pool".as_slice().into(), b"missing".as_slice().into()),
        revision: 0,
        until: 3600,
    };
    ask(sign_in, 3, value)
}

fn ask(sign_in: u64, key: u8, value: infrastructure::ApplicationRequest) -> root::Event {
    let mut bytes = Vec::new();
    bytes.extend(u64::try_from(value.environment.pool.len()).expect("bounded pool name").to_be_bytes());
    bytes.extend_from_slice(&value.environment.pool);
    bytes.extend(u64::try_from(value.environment.name.len()).expect("bounded environment name").to_be_bytes());
    bytes.extend_from_slice(&value.environment.name);
    bytes.extend(value.revision.to_be_bytes());
    bytes.extend(value.until.to_be_bytes());
    root::Event::Application {
        reply_to: ReplyTo::new(Token::new(600)),
        sign_in,
        key: [key; 16],
        project: 1,
        digest: Sha256::digest(bytes).into(),
        value,
    }
}

#[test]
fn an_application_request_reaches_infrastructure_and_waits_for_its_commit() {
    let mut world = Harness::<Ops>::new(Config::new(713, Story::Night), 713);
    world.drain().expect("ops startup");
    let sign_in = world.peers.sign_in.expect("signed in project owner");
    world.store.fault(jig_fake_store::Fault::Hold { commits: 1 });
    world.send(request(sign_in));
    world.drain().expect("decision waits for durable completion");
    assert!(world.peers.application_answers.is_empty());
    world.release_commits();
    world.drain().expect("release application terminal");
    assert_eq!(world.peers.application_answers, [infrastructure::ApplicationOutcome::Unknown]);
    let applied = world.store.applied;
    world.send(request(sign_in));
    world.drain().expect("replay application terminal");
    assert_eq!(
        world.peers.application_answers,
        [infrastructure::ApplicationOutcome::Unknown, infrastructure::ApplicationOutcome::Unknown]
    );
    assert_eq!(world.store.applied, applied, "the connector did not decide a replay");
}

#[test]
fn moving_an_environments_end_is_durable_revision_checked_and_replayed_after_restart() {
    use jig_ops_fake_production as production;
    let mut world = Harness::<Ops>::new(Config::new(714, Story::Night), 714);
    world.drain().expect("ops startup");
    let sign_in = world.peers.sign_in.expect("signed in owner");
    let environment = infrastructure::Environment::new(b"pool".as_slice().into(), b"preview".as_slice().into());
    let name = production::EnvironmentName::new("pool", "preview");
    let created_by = production::Key::procedure([7; 16], 90, 1);
    world.systems.production.set_pool_quota("pool", 1);
    assert_eq!(
        world.systems.production.create_environment(name.clone(), created_by, 100, 2),
        production::ResultValue::Made
    );
    let fact =
        world.systems.infrastructure(infrastructure::SystemRequest::Environment { environment: environment.clone() });
    world.send(root::Event::Infrastructure(infrastructure::Event::System(fact)));
    world.drain().expect("fresh environment");
    let value = infrastructure::ApplicationRequest { environment: environment.clone(), revision: 0, until: 200 };
    world.store.fault(jig_fake_store::Fault::Hold { commits: 1 });
    world.send(ask(sign_in, 7, value.clone()));
    world.drain().expect("held decision");
    assert!(world.peers.application_answers.is_empty());
    assert_eq!(world.systems.production.read_environment(&name).expect("live environment").until, 100);
    world.release_commits();
    world.drain().expect("durable end and answer");
    let now = infrastructure::EnvironmentEnd { revision: 1, until: 200 };
    let kept = infrastructure::ApplicationOutcome::MovedEnd { environment: environment.clone(), now };
    assert_eq!(world.peers.application_answers.as_slice(), std::slice::from_ref(&kept));
    assert_eq!(world.systems.production.read_environment(&name).expect("live environment").until, 200);
    world.send(ask(sign_in, 8, infrastructure::ApplicationRequest { until: 300, ..value.clone() }));
    world.drain().expect("stale request refused");
    assert_eq!(
        world.peers.application_answers.last(),
        Some(&infrastructure::ApplicationOutcome::Moved { environment: environment.clone(), now })
    );
    let cut = world.store.applied + 1;
    world.cut(jig_conformance::Cut::AfterCommit(cut));
    world.send(ask(sign_in, 9, infrastructure::ApplicationRequest { revision: 1, until: 150, ..value.clone() }));
    world.drain().expect("recover a durable end whose backend call and answer were lost");
    assert_eq!(world.outcome().crashes.last(), Some(&cut));
    assert_eq!(world.peers.application_answers.len(), 2, "the second terminal was lost with its completion");
    assert_eq!(
        world.systems.production.read_environment(&name).expect("saved end reconciled after restart").until,
        150
    );
    world.systems.production.move_environment_end(&name, created_by, 1, 200);
    assert_eq!(world.systems.production.read_environment(&name).expect("late copy preserves latest end").until, 150);
    let made = world.systems.production.end_changes.iter().filter(|change| change.applied).count();
    world.crash().expect("cold restart");
    world.drain().expect("restore kept ends and outcomes");
    let writes = world.store.applied;
    world.send(ask(sign_in, 7, value));
    world.drain().expect("replay original terminal");
    assert_eq!(world.peers.application_answers.last(), Some(&kept));
    assert_eq!(world.store.applied, writes, "replay makes no decision");
    assert_eq!(world.systems.production.end_changes.iter().filter(|change| change.applied).count(), made);
    assert_eq!(world.systems.production.read_environment(&name).expect("replay preserves newer end").until, 150);
}
