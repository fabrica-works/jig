//! Configuration seeds project policies and first owners once (configuration.md, 3).

use jig_core_world::{client_peer::ClientPeer, effects::World, world};
use jig_domain_authority as authority;
use jig_domain_core::{self as core, documents, reads};
use jig_domain_people as people;
use jig_test_domain as root;
use skein_lib::{Queue, ReplyTo, Token};

fn cold() -> World {
    let mut world = World::first_start(world::config(11_001), world::limits());
    world.client_peer = Some(ClientPeer::default());
    world
}

fn sign_in(world: &mut World, subject: u64) -> (u64, u64) {
    world.send(root::Event::Core(core::Event::SignIn {
        reply_to: ReplyTo::new(Token::new(11_001)),
        identity: people::Identity {
            key: people::IdentityKey { provider: 0, subject: subject.to_be_bytes().into() },
            login: b"owner".as_slice().into(),
            name: b"First owner".as_slice().into(),
        },
    }));
    let (
        _,
        root::Delivery::Core(core::Held::PeopleReply {
            reply: people::Reply::SignedIn { person, sign_in, .. }, ..
        }),
    ) = world.client_peer.as_mut().expect("client peer").delivered.pop().expect("signed in reply")
    else {
        panic!("configured owner signs in")
    };
    (person, sign_in)
}

fn project(world: &mut World, sign_in: u64) -> documents::Project {
    world.send(root::Event::Core(core::Event::Read {
        reply: ReplyTo::new(Token::new(11_002)),
        sign_in,
        read: reads::Read::Project { project: 1 },
    }));
    let (_, root::Delivery::Core(core::Held::Read { answer: reads::Answer::Document(document), .. })) =
        world.client_peer.as_mut().expect("client peer").delivered.pop().expect("project read reply")
    else {
        panic!("project document reply")
    };
    let documents::Document::Project(project) = *document else { panic!("project policy document") };
    *project
}

#[test]
fn a_first_start_seeds_the_policy_and_owner_before_the_owner_signs_in() {
    let mut world = cold();
    assert!(world.domain.ready());
    assert_eq!(world.store.applied, 1, "one first-start decision");
    assert_eq!(
        world.store.rows[&root::Key::Core(core::Key::People(people::Key::Roles(1)))],
        root::Record::Core(core::Record::People(people::Stored::Roles {
            project: 1,
            holdings: Box::new([people::Holding { person: 1, role: people::Role::Owner }]),
        }))
    );
    assert!(world.store.rows.contains_key(&root::Key::Core(core::Key::People(people::Key::Policy(1)))));
    assert!(world.store.rows.contains_key(&root::Key::Core(core::Key::People(people::Key::Person(1)))));
    let (person, sign_in) = sign_in(&mut world, 7);
    assert_eq!(person, 1, "sign-in resolves the person already seeded");
    let project = project(&mut world, sign_in);
    assert_eq!(project.policy.period_spend, 1000);
    assert_eq!(project.members.len(), 1);
    assert_eq!(project.members.as_slice()[0].role, people::Role::Owner);
    assert!(project.resources.is_empty());
}

#[test]
fn a_restart_seeds_nothing_and_keeps_the_saved_policy_and_owner() {
    let mut world = cold();
    let rows = world.store.rows.clone();
    let commits = world.commits.len();
    let mut configuration = world::config(11_002);
    let mut policy = configuration.core.authority.policy(1).expect("configured policy").clone();
    policy.period_spend = 700;
    let mut facts = Queue::with_capacity(authority::POLICY_MAX_OUT);
    authority::step(&mut configuration.core.authority, authority::Event::Policy { project: 1, policy }, &mut facts);
    assert_eq!(facts.pop(), Some(authority::PolicyFact::Changed { project: 1 }));
    configuration.core.owners = Box::new([]);
    configuration.core.projects.clear();
    world.restart_with(configuration);
    assert!(world.domain.ready());
    assert_eq!(world.store.rows, rows);
    assert_eq!(world.store.applied, 1);
    assert_eq!(world.commits.len(), commits);
    let (person, sign_in) = sign_in(&mut world, 7);
    assert_eq!(person, 1);
    let project = project(&mut world, sign_in);
    assert_eq!(project.members.len(), 1);
    assert_eq!(project.members.as_slice()[0].role, people::Role::Owner);
    assert_eq!(project.policy.period_spend, 1000);
}

#[test]
fn a_restart_does_not_grant_new_configured_owners() {
    let mut world = cold();
    let roles = root::Key::Core(core::Key::People(people::Key::Roles(1)));
    let expected = world.store.rows[&roles].clone();
    let mut configuration = world::config(11_003);
    configuration.core.owners[0].identity.subject = 8_u64.to_be_bytes().into();
    world.restart_with(configuration);
    let (person, _) = sign_in(&mut world, 8);
    assert_eq!(person, 2);
    assert_eq!(world.store.rows[&roles], expected);
}
