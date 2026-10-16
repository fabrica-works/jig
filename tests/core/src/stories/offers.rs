//! Starting facts shared by the delivered-offer stories.
use crate::effects::{World, fixture};
use jig_domain_authority as authority;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_test_domain as root;
use skein_lib::{Queue, ReplyTo, Token};

/// The same fixed goal form and authority used by the offer world, for cold restoration.
#[must_use]
pub fn world_configuration(seed: u64) -> root::Config {
    let (mut configuration, _) = fixture(seed, false);
    crate::world::edit_role(&mut configuration.core, |role| {
        role.authority.delegation.tasks = 2;
        role.authority.delegation.depth = 2;
    });
    configuration
}

#[must_use]
pub fn world(seed: u64, role: Option<people::Role>) -> World {
    let (mut configuration, mut limits) = fixture(seed, false);
    if let Some(held) = role {
        limits.core.authority.roles = 2;
        let mut policy = configuration.core.authority.policy(1).expect("policy").clone();
        let mut added = policy.roles[0].clone();
        added.number = held.number();
        added.authority.budget.spend = 100;
        added.requests = if held == people::Role::Observer { authority::Requests(0) } else { authority::Requests::ALL };
        added.decides = authority::Proposals(0);
        policy.roles = Box::new([policy.roles[0].clone(), added]);
        let mut domain = authority::Domain::new(configuration.core.authority.rules().clone(), limits.core.authority)
            .expect("role bounds");
        let mut facts = Queue::with_capacity(authority::POLICY_MAX_OUT);
        authority::step(&mut domain, authority::Event::Policy { project: 1, policy }, &mut facts);
        assert_eq!(facts.pop(), Some(authority::PolicyFact::Added { project: 1 }));
        configuration.core.authority = domain;
    }
    crate::world::edit_role(&mut configuration.core, |role| {
        role.authority.delegation.tasks = 2;
        role.authority.delegation.depth = 2;
    });
    let mut world = World::configured(seed, false, configuration, limits);
    let person = world
        .people_answers
        .iter()
        .find_map(|reply| if let people::Reply::SignedIn { person, .. } = reply { Some(*person) } else { None })
        .expect("signed-in party");
    if let Some(role) = role {
        world.send(jig_test_domain::Event::Core(core::Event::People(people::Event::Ask {
            reply_to: ReplyTo::new(Token::new(6000)),
            sign_in: world.sign_in.expect("sign in"),
            key: [0; 16],
            ask: people::Ask::SetRoles { project: 1, holdings: Box::new([people::Holding { person, role }]) },
        })));
        assert_eq!(
            world.people_answers.last(),
            Some(&people::Reply::Outcome(people::Outcome::RolesSet { project: 1 }))
        );
    }
    let mut referee = crate::referee::offers::Referee::default();
    referee.register(world.sign_in.expect("signed in"), person, world.store.applied);
    world.offers = Some(referee);
    world.watches =
        Some(crate::watches::Parties::new(seed, world.store.applied, true, world.sign_in.expect("signed in")));
    world.reads = Some(crate::reads::Parties::new(world.store.applied));
    crate::reads::ask(&mut world, 91, core::reads::Read::Project { project: 1 });
    world
}

#[must_use]
pub fn no_acceptor_fixture(seed: u64) -> (root::Config, root::Limits) {
    let (mut configuration, mut limits) = fixture(seed, false);
    limits.core.authority.roles = 2;
    let mut policy = configuration.core.authority.policy(1).expect("configured policy").clone();
    let mut member = policy.roles[0].clone();
    member.number = people::Role::Member.number();
    member.authority.budget.spend = 100;
    member.decides = authority::Proposals(0);
    policy.roles[0].decides = authority::Proposals(16);
    policy.roles = Box::new([policy.roles[0].clone(), member]);
    let mut domain = authority::Domain::new(configuration.core.authority.rules().clone(), limits.core.authority)
        .expect("valid roles");
    let mut out = Queue::with_capacity(authority::POLICY_MAX_OUT);
    authority::step(&mut domain, authority::Event::Policy { project: 1, policy }, &mut out);
    assert_eq!(out.pop(), Some(authority::PolicyFact::Added { project: 1 }));
    configuration.core.authority = domain;
    (configuration, limits)
}

/// A member's roots have no holder until policy gives that role the decision.
#[must_use]
pub fn no_holder_fixture(seed: u64) -> (root::Config, root::Limits) {
    let (mut configuration, limits) = no_acceptor_fixture(seed);
    let mut policy = configuration.core.authority.policy(1).expect("configured policy").clone();
    policy.escalation_role = None;
    let mut out = Queue::with_capacity(authority::POLICY_MAX_OUT);
    authority::step(&mut configuration.core.authority, authority::Event::Policy { project: 1, policy }, &mut out);
    assert_eq!(out.pop(), Some(authority::PolicyFact::Changed { project: 1 }));
    (configuration, limits)
}

#[must_use]
pub fn goal(budget: u64) -> people::Ask {
    people::Ask::SetGoal {
        project: 1,
        spec: b"offered goal".as_slice().into(),
        executor: jig_domain_people::GoalExecutor::Charter(1),
        budget,
        priority: 1,
        deadline: None,
    }
}

#[must_use]
pub fn project(world: &World) -> &core::documents::Document {
    let core::reads::Answer::Document(value) = crate::reads::latest(world) else {
        panic!("project read");
    };
    value
}
