//! Admission refuses party proposals with no accepting role (QUESTIONS 31).
use super::*;
use jig_core_world::stories::offers::{goal, no_acceptor_fixture, project};

#[test]
fn a_members_goal_beyond_their_pool_is_refused_when_nobody_may_accept_it() {
    let (configuration, limits) = no_acceptor_fixture(180);
    let mut world = World::configured(180, false, configuration, limits);
    let person = world
        .people_answers
        .iter()
        .find_map(|reply| if let people::Reply::SignedIn { person, .. } = reply { Some(*person) } else { None })
        .expect("signed-in person");
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
        executor: jig_domain_people::GoalExecutor::Charter(1),
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

#[test]
fn a_member_is_offered_a_goal_past_their_allotment_as_proposed() {
    use jig_domain_core::{
        acts::{Act, Admission, Beyond, Bounds, Recipient},
        documents::Document,
    };
    let mut world = jig_core_world::stories::offers::world(181, Some(people::Role::Member));
    let Document::Project(project) = project(&world) else {
        panic!("project");
    };
    let offered = project.acts.iter().find(|value| value.act == Act::SetGoal).expect("goal offer");
    let Admission::Bounded {
        within: Bounds::Goal { pool_left, budget, charters, maximum_budget, .. },
        beyond: Beyond::Proposed { to: Recipient::Roles { project, roles } },
    } = &offered.admission
    else {
        panic!("bounded proposal");
    };
    assert_eq!((*pool_left, *budget), (200, 100));
    assert_eq!(charters.as_slice(), &[1]);
    assert!(*maximum_budget >= 600);
    assert_eq!((*project, roles.as_slice()), (1, [0].as_slice()));
    let before = world.assignments.len();
    ask(&mut world, 92, goal(600));
    let Some(people::Reply::Outcome(people::Outcome::GoalProposed { proposal })) = world.people_answers.last() else {
        panic!("goal proposed");
    };
    assert!(world.store.rows.values().any(|row| matches!(row, root::Record::Core(core::Record::Tasks(tasks::Stored::PersonProposal(value))) if value.number == *proposal && value.goal.authority.budget.spend == 600)));
    assert_eq!(world.assignments.len(), before, "no goal runs before acceptance");
    assert_eq!(world.offers.as_ref().expect("offer referee").checked, 1);
    world.watches.as_ref().expect("watch parties").finish();
}

#[test]
fn an_observer_is_offered_nothing_and_told_why() {
    use jig_domain_core::{
        acts::{Act, Admission},
        documents::Document,
    };
    let mut world = jig_core_world::stories::offers::world(182, Some(people::Role::Observer));
    let Document::Project(project) = project(&world) else {
        panic!("project");
    };
    let refused = people::Refusal::Role { held: Some(people::Role::Observer) };
    assert_eq!(project.acts.len(), 7);
    assert!(project.acts.iter().all(|value| value.admission == Admission::Refused(refused)));
    assert!(project.acts.iter().any(|value| value.act == Act::SetGoal));
    let task_rows = world
        .store
        .rows
        .values()
        .filter(|row| matches!(row, root::Record::Core(core::Record::Tasks(_))))
        .cloned()
        .collect::<Vec<_>>();
    ask(&mut world, 93, goal(50));
    assert_eq!(world.people_answers.last(), Some(&people::Reply::Outcome(people::Outcome::Refused(refused))));
    assert_eq!(
        world
            .store
            .rows
            .values()
            .filter(|row| matches!(row, root::Record::Core(core::Record::Tasks(_))))
            .cloned()
            .collect::<Vec<_>>(),
        task_rows
    );
    assert_eq!(world.offers.as_ref().expect("offer referee").checked, 1);
    world.watches.as_ref().expect("watch parties").finish();
}

#[test]
fn a_policy_narrowed_withdraws_an_offer_in_the_next_delivery() {
    use jig_domain_core::{
        acts::{Act, Admission},
        documents::{Document, Subject},
    };
    let mut world = jig_core_world::stories::offers::world(183, None);
    let run = world.assignments[0];
    let mut child = member(&world, tasks::Executor::Agent { charter: 1 }, b"delegated work");
    child.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    let task = delegate(&mut world, run, 7, child);
    jig_core_world::watches::open(&mut world, 91, Subject::Task { task });
    let Document::Task(value) = jig_core_world::watches::latest(&world, 91) else {
        panic!("task");
    };
    assert_eq!(
        value.acts.iter().find(|value| value.act == Act::TakeOver).expect("offered takeover").admission,
        Admission::Allowed
    );
    let first = world.watches.as_ref().expect("watches").deliveries.len();
    let (configuration, _) = fixture(183, false);
    let role = configuration.core.authority.policy(1).expect("policy").roles[0].clone();
    let change = people::PolicyChange::Role(people::PolicyRole {
        number: role.number,
        authority: people::Authority {
            tools: role.authority.tools.0,
            grants: Box::new([people::Grant {
                connector: 1,
                kind: 5,
                pattern: people::Pattern { segments: Box::new([]), last: people::Last::Open(Box::new([])) },
            }]),
            delegation: people::Delegation {
                kinds: Box::new([people::Executor::Charter(1), people::Executor::Procedure(1)]),
                tasks: role.authority.delegation.tasks,
                depth: role.authority.delegation.depth,
            },
            spend: 0,
            deadline: role.authority.budget.deadline,
            notes: role.authority.notes.0,
            note_resources: Box::new([]),
        },
        period_spend: role.period_spend,
        requests: role.requests.0,
        decides: role.decides.0,
    });
    ask(&mut world, 94, people::Ask::ChangePolicy { project: 1, change });
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::PolicyChanged { project: 1 }))
    );
    let deliveries = &world.watches.as_ref().expect("watches").deliveries;
    assert!(deliveries.len() > first, "policy commit replaces the task offers");
    assert!(deliveries.last().expect("replacement").1 >= world.store.applied);
    let Document::Task(value) = jig_core_world::watches::latest(&world, 91) else {
        panic!("task");
    };
    let Admission::Refused(refused) =
        value.acts.iter().find(|value| value.act == Act::TakeOver).expect("withdrawn takeover").admission
    else {
        panic!("withdrawn offer");
    };
    assert!(matches!(refused, people::Refusal::Authority { .. }));
    ask(&mut world, 95, people::Ask::Move { project: 1, task, reason: b"make mine".as_slice().into() });
    assert_eq!(world.people_answers.last(), Some(&people::Reply::Outcome(people::Outcome::Refused(refused))));
    assert_eq!(world.offers.as_ref().expect("offer referee").checked, 1);
    world.watches.as_ref().expect("watch parties").finish();
}
