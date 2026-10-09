//! Admission refuses party proposals with no accepting role (QUESTIONS 31).
use super::*;

fn no_acceptor_fixture(seed: u64) -> (root::Config, root::Limits) {
    let (mut configuration, mut limits) = fixture(seed, false);
    limits.core.authority.roles = 2;
    let mut policy = configuration.core.authority.policy(1).unwrap().clone();
    let mut member = policy.roles[0].clone();
    member.number = people::Role::Member.number();
    member.authority.budget.spend = 100;
    member.decides = authority::Proposals(0);
    policy.roles[0].decides = authority::Proposals(16);
    policy.roles = Box::new([policy.roles[0].clone(), member]);
    let mut domain =
        authority::Domain::new(configuration.core.authority.rules().clone(), limits.core.authority).unwrap();
    let mut out = Queue::with_capacity(authority::POLICY_MAX_OUT);
    authority::step(&mut domain, authority::Event::Policy { project: 1, policy }, &mut out);
    assert_eq!(out.pop(), Some(authority::PolicyFact::Added { project: 1 }));
    configuration.core.authority = domain;
    (configuration, limits)
}

#[test]
fn a_members_goal_beyond_their_pool_is_refused_when_nobody_may_accept_it() {
    let (configuration, limits) = no_acceptor_fixture(180);
    let mut world = World::configured(180, false, configuration, limits);
    let person = world
        .people_answers
        .iter()
        .find_map(|reply| if let people::Reply::SignedIn { person, .. } = reply { Some(*person) } else { None })
        .unwrap();
    ask(
        &mut world,
        90,
        people::Ask::SetRoles {
            project: 1,
            holdings: Box::new([people::Holding { person, role: people::Role::Member }]),
        },
    );
    assert_eq!(world.people_answers.last(), Some(&people::Reply::Outcome(people::Outcome::RolesSet { project: 1 })));
    let tasks_before: Vec<_> = world
        .store
        .rows
        .values()
        .filter(|row| matches!(row, root::Record::Core(core::Record::Tasks(_))))
        .cloned()
        .collect();
    let goal = people::Ask::SetGoal {
        project: 1,
        spec: b"beyond my pool".as_slice().into(),
        charter: 1,
        budget: 600,
        priority: 1,
        deadline: None,
    };
    let refused = people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Authority {
        lacking: people::Lack::Approver,
        proposable: Some(false),
    }));
    ask(&mut world, 91, goal.clone());
    assert_eq!(world.people_answers.last(), Some(&refused));
    let tasks_after: Vec<_> = world
        .store
        .rows
        .values()
        .filter(|row| matches!(row, root::Record::Core(core::Record::Tasks(_))))
        .cloned()
        .collect();
    assert_eq!(tasks_after, tasks_before, "no task, proposal or funding change is made");
    ask(&mut world, 91, goal.clone());
    assert_eq!(world.people_answers.last(), Some(&refused), "the keyed refusal replays");
    world.restart_with(no_acceptor_fixture(180).0);
    ask(&mut world, 91, goal);
    assert_eq!(world.people_answers.last(), Some(&refused), "the fact survives cold restoration");
    assert!(
        !world
            .store
            .rows
            .values()
            .any(|row| matches!(row, root::Record::Core(core::Record::Tasks(tasks::Stored::PersonProposal(_)))))
    );
}
