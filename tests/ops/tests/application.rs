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
        key: [3; 16],
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
