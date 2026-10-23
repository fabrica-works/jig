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
        held: None,
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
    assert_eq!(world.store.applied, 1, "one first-start decision; no account counter changed");
    assert!(!world.store.rows.contains_key(&root::Key::Core(core::Key::Accounts(jig_domain_accounts::Key::Next))));
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
    assert_eq!(world.store.applied, 1, "the restored counter has not moved");
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

fn two_projects(people_projects: u32) -> (root::Config, root::Limits) {
    let mut configuration = world::config(11_020);
    let mut limits = world::limits();
    limits.core.authority.projects = 2;
    limits.core.people.projects = people_projects;
    // Reprice this fixture's journal for the second policy, as world::limits does.
    let room = core::room_max(&limits.core).expect("two-project route room");
    let connectors = jig_test_connector::MAX_OUT * 2;
    limits.journal.writes = room.writes + connectors;
    limits.journal.held = limits.journal.held.max(room.held + connectors);
    let mut authority = authority::Domain::new(configuration.core.authority.rules().clone(), limits.core.authority)
        .expect("two policies");
    let mut projects = skein_lib::List::with_capacity(2);
    let mut facts = Queue::with_capacity(authority::POLICY_MAX_OUT);
    for (project, spend) in [(1, 1000), (2, 700)] {
        let mut policy = configuration.core.authority.policy(1).expect("fixture policy").clone();
        policy.period_spend = spend;
        authority::step(&mut authority, authority::Event::Policy { project, policy }, &mut facts);
        assert_eq!(facts.pop(), Some(authority::PolicyFact::Added { project }));
        projects.push(project).expect("two configured projects");
    }
    configuration.core.authority = authority;
    configuration.core.projects = projects;
    configuration.core.names = skein_lib::Map::with_capacity(limits.core.authority.projects);
    configuration.core.names.insert(1, Box::from("Tally")).expect("first name");
    configuration.core.names.insert(2, Box::from("Project 2")).expect("second name");
    let mut permissions = skein_lib::Map::with_capacity(limits.core.authority.projects);
    for (&project, roles) in &configuration.core.permission_roles {
        permissions.insert(project, roles.clone()).expect("two project permission maps");
    }
    configuration.core.permission_roles = permissions;
    configuration.core.template_overrides = skein_lib::Map::with_capacity(limits.core.authority.projects);
    // The fixture's sole owner belongs to project 1; project 2 still needs a roster.
    (configuration, limits)
}

#[test]
fn all_configured_projects_seed_together_at_or_below_people_capacity() {
    for capacity in [AT_ROSTER_BOUND, AT_ROSTER_BOUND + DEPLOYMENT_ROSTER] {
        let (configuration, limits) = two_projects(capacity);
        let mut world = World::first_start(configuration, limits);
        assert!(world.domain.ready());
        assert_eq!(world.store.applied, 1, "the whole seed is one atomic decision");
        assert_eq!(world.commits.len(), 1);
        for (project, spend, holdings) in
            [(1, 1000, vec![people::Holding { person: 1, role: people::Role::Owner }]), (2, 700, vec![])]
        {
            assert_eq!(
                world.store.rows[&root::Key::Core(core::Key::People(people::Key::Roles(project)))],
                root::Record::Core(core::Record::People(people::Stored::Roles {
                    project,
                    holdings: holdings.into_boxed_slice(),
                }))
            );
            let root::Record::Core(core::Record::People(people::Stored::Policy {
                project: saved_project, value, ..
            })) = &world.store.rows[&root::Key::Core(core::Key::People(people::Key::Policy(project)))]
            else {
                panic!("each configured project has a saved policy")
            };
            assert_eq!((*saved_project, value.period_spend), (project, spend));
        }

        let rows = world.store.rows.clone();
        let commits = world.commits.len();
        let (mut changed, _) = two_projects(capacity);
        changed.core.owners = Box::new([]);
        changed.core.projects.clear();
        world.restart_with(changed);
        assert!(world.domain.ready());
        assert_eq!(world.store.rows, rows, "restart keeps every seeded policy and roster");
        assert_eq!(world.store.applied, 1, "restart does not reseed");
        assert_eq!(world.commits.len(), commits);
    }
}

fn model(name: &[u8]) -> core::Model {
    core::Model { name: name.into(), ..world::scripted_model() }
}

/// The fixture deployment under `models`, its owner's role naming `role` of them.
fn with_models(seed: u64, models: Box<[core::Model]>, role: authority::Models) -> root::Config {
    let mut configuration = world::config(seed);
    configuration.core.settings.models = models;
    let mut policy = configuration.core.authority.policy(1).expect("configured policy").clone();
    let mut roles = policy.roles.into_vec();
    roles[0].authority.models = role;
    policy.roles = roles.into_boxed_slice();
    let mut facts = Queue::with_capacity(authority::POLICY_MAX_OUT);
    authority::step(&mut configuration.core.authority, authority::Event::Policy { project: 1, policy }, &mut facts);
    assert_eq!(facts.pop(), Some(authority::PolicyFact::Changed { project: 1 }));
    configuration
}

// The core’s Frame is the source of the protocol’s me document.
fn frame(world: &mut World, sign_in: u64) -> documents::Frame {
    world.send(root::Event::Core(core::Event::Read {
        reply: ReplyTo::new(Token::new(11_003)),
        sign_in,
        read: reads::Read::Frame,
    }));
    let (_, root::Delivery::Core(core::Held::Read { answer: reads::Answer::Document(document), .. })) =
        world.client_peer.as_mut().expect("client peer").delivered.pop().expect("frame read reply")
    else {
        panic!("frame document reply")
    };
    let documents::Document::Frame(frame) = *document else { panic!("frame document") };
    frame
}

fn frame_models(world: &mut World, sign_in: u64) -> authority::Models {
    frame(world, sign_in).projects.as_slice()[0].models
}

const FIRST_OWNER_SUBJECT: u64 = 7; // world::config’s configured owner.
const OTHER_SUBJECT: u64 = 8;
const CONFIGURED_PROJECTS: u32 = 2; // two_projects seeds 1 and 2.
const DEPLOYMENT_ROSTER: u32 = 1;
const AT_ROSTER_BOUND: u32 = CONFIGURED_PROJECTS + DEPLOYMENT_ROSTER;

fn first_frame(subject: u64) -> documents::Frame {
    let mut world = cold();
    let (_, signed) = sign_in(&mut world, subject);
    frame(&mut world, signed)
}

#[test]
fn a_first_owner_holds_the_deployments_owner_role_from_the_first_start() {
    assert_eq!(first_frame(FIRST_OWNER_SUBJECT).deployment_role, Some(people::Role::Owner));
}

#[test]
fn a_party_who_is_no_first_owner_holds_no_deployment_role() {
    assert_eq!(first_frame(OTHER_SUBJECT).deployment_role, None);
}

#[test]
fn the_frame_and_me_list_no_project_for_the_deployments_roles() {
    let (mut configuration, mut limits) = two_projects(AT_ROSTER_BOUND);
    // The parser expands each configured first owner over both projects.
    let owner = configuration.core.owners[0].clone();
    configuration.core.owners = Box::new([owner.clone(), people::InitialOwner { project: 2, ..owner }]);
    limits.core.people.initial_owners = CONFIGURED_PROJECTS;
    jig_core_world::peers::journal(&mut limits);
    let mut world = World::first_start(configuration, limits);
    world.client_peer = Some(ClientPeer::default());
    let (_, signed) = sign_in(&mut world, FIRST_OWNER_SUBJECT);
    let value = frame(&mut world, signed);
    assert_eq!(value.projects.as_slice().iter().map(|row| row.project).collect::<Vec<_>>(), [1, 2]);
}

#[test]
fn with_no_project_configured_the_first_owner_owns_the_deployment() {
    let mut configuration = world::config(11_030);
    configuration.core.projects.clear();
    // Configuration carries a deployment bootstrap entry when there is no project.
    for owner in &mut configuration.core.owners {
        owner.project = people::DEPLOYMENT;
    }
    let mut world = World::first_start(configuration, world::limits());
    world.client_peer = Some(ClientPeer::default());
    let (_, signed) = sign_in(&mut world, FIRST_OWNER_SUBJECT);
    assert_eq!(frame(&mut world, signed).deployment_role, Some(people::Role::Owner));
}

#[test]
fn a_roles_models_follow_their_names_across_restarts() {
    let named = || Box::new([world::scripted_model(), model(b"research"), model(b"review")]);
    let mut world = World::first_start(with_models(11_004, named(), authority::Models::Named(0b101)), world::limits());
    world.client_peer = Some(ClientPeer::default());
    let (_, signed) = sign_in(&mut world, 7);
    assert_eq!(frame_models(&mut world, signed), authority::Models::Named(0b101), "scripted and review, seeded");
    world.restart_with(with_models(11_005, named(), authority::Models::ALL));
    assert_eq!(
        frame_models(&mut world, signed),
        authority::Models::Named(0b101),
        "the saved role, not configuration's"
    );
    let today = Box::new([model(b"research"), world::scripted_model()]);
    world.restart_with(with_models(11_006, today, authority::Models::ALL));
    assert_eq!(frame_models(&mut world, signed), authority::Models::Named(0b10), "scripted moves; review drops out");
}

fn named_edit(world: &mut World, sign_in: u64, key: u8, name: &[u8]) -> people::Reply {
    let mut member = jig_core_world::fixtures::people_member(b"work", 1);
    member.authority.models = Some(people::Models::Named(1));
    world.send(root::Event::Core(core::Event::People(people::Event::Ask {
        reply_to: ReplyTo::new(Token::new(11_010)),
        sign_in,
        key: [key; 16],
        ask: people::Ask::ChangePolicy {
            project: 1,
            change: people::PolicyChange::Role(people::PolicyRole {
                number: 0,
                authority: member.authority,
                period_spend: 1,
                requests: authority::Requests::ALL.0,
                decides: authority::Proposals::ALL.0,
            }),
        },
        models: Box::new([name.into()]),
    })));
    let (_, root::Delivery::Core(core::Held::PeopleReply { reply, .. })) =
        world.client_peer.as_mut().expect("client peer").delivered.pop().expect("keyed reply")
    else {
        panic!("keyed answer")
    };
    reply
}
#[test]
fn an_original_key_replays_before_model_admission_after_removal() {
    let mut world = World::first_start(
        with_models(11_011, Box::new([world::scripted_model(), model(b"removed")]), authority::Models::ALL),
        world::limits(),
    );
    world.client_peer = Some(ClientPeer::default());
    let (_, sign_in) = sign_in(&mut world, 7);
    let original = named_edit(&mut world, sign_in, 80, b"removed");
    assert_eq!(original, people::Reply::Outcome(people::Outcome::PolicyChanged { project: 1 }));
    world.restart_with(with_models(11_012, Box::new([world::scripted_model(), model(b"new")]), authority::Models::ALL));
    assert_eq!(named_edit(&mut world, sign_in, 80, b"removed"), original);
    assert_eq!(named_edit(&mut world, sign_in, 80, b"new"), people::Reply::Refused(people::Refusal::KeyConflict));
    assert_eq!(
        named_edit(&mut world, sign_in, 81, b"removed"),
        people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Unknown))
    );
}

fn named_projects() -> (root::Config, root::Limits) {
    let (mut configuration, mut limits) = two_projects(AT_ROSTER_BOUND);
    let owner = configuration.core.owners[0].clone();
    configuration.core.owners = Box::new([owner.clone(), people::InitialOwner { project: 2, ..owner }]);
    limits.core.people.initial_owners = CONFIGURED_PROJECTS;
    jig_core_world::peers::journal(&mut limits);
    (configuration, limits)
}

fn project_names(world: &mut World, sign_in: u64) -> Vec<(u32, Box<[u8]>)> {
    frame(world, sign_in)
        .projects
        .into_boxed()
        .into_vec()
        .into_iter()
        .map(|row| (row.project, row.name.bytes))
        .collect()
}

#[test]
fn a_first_start_names_each_project() {
    let (configuration, limits) = named_projects();
    let mut world = World::first_start(configuration, limits);
    world.client_peer = Some(ClientPeer::default());
    let (_, signed) = sign_in(&mut world, FIRST_OWNER_SUBJECT);
    assert_eq!(project_names(&mut world, signed), [(1, Box::from(&b"Tally"[..])), (2, Box::from(&b"Project 2"[..]))]);
}

#[test]
fn a_restart_keeps_each_projects_name() {
    let (configuration, limits) = named_projects();
    let mut world = World::first_start(configuration, limits);
    world.client_peer = Some(ClientPeer::default());
    let (mut changed, _) = named_projects();
    // Even swapped configured names cannot collide with the names being restored.
    changed.core.names.insert(1, Box::from("Project 2")).expect("first name");
    changed.core.names.insert(2, Box::from("Tally")).expect("second name");
    changed.core.projects.clear();
    world.restart_with(changed);
    let (_, signed) = sign_in(&mut world, FIRST_OWNER_SUBJECT);
    assert_eq!(project_names(&mut world, signed), [(1, Box::from(&b"Tally"[..])), (2, Box::from(&b"Project 2"[..]))]);
}
