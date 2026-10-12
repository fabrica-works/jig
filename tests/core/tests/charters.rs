use jig_core_world::{effects::World, world};
use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{ReplyTo, Token};

#[test]
fn a_delegate_on_another_charter_runs_its_instructions_and_account() {
    let (config, limits) = world::two_charters(141);
    let mut world = World::configured(141, false, config, limits);
    assert_eq!(world.assigned.as_ref().expect("chat").charter, 1);
    world.grant_account(2);
    world.delegate_agent(2, b"find the regression");
    let assignment = world.assigned.as_ref().expect("delegate assigned");
    assert_eq!(assignment.charter, 2);
    assert_eq!(assignment.run.policy.instructions.as_ref(), b"Research the question.");
    assert_eq!(assignment.grant.account, 2);
}

#[test]
fn a_task_waits_while_its_charters_account_is_unusable() {
    let (config, limits) = world::two_charters(142);
    let mut world = World::configured(142, false, config, limits);
    let chat = world.assigned.as_ref().expect("chat").task;
    world.delegate_agent(2, b"wait for the account");
    assert_eq!(world.assigned.as_ref().expect("chat stays assigned").task, chat);
    assert!(world.store.rows.values().any(|row| matches!(row,
        root::Record::Core(core::Record::Tasks(tasks::Stored::Live(task)))
        if task.executor == tasks::Executor::Agent { charter: 2 } && task.attempt == 0)));
    world.grant_account(2);
    assert_eq!(world.assigned.as_ref().expect("delegate after grant").charter, 2);
}

#[test]
fn a_restart_restores_a_task_on_the_second_charter() {
    let (config, limits) = world::two_charters(143);
    let mut world = World::configured(143, false, config, limits);
    world.delegate_agent(2, b"research across restart");
    let task = world
        .store
        .rows
        .values()
        .find_map(|row| match row {
            root::Record::Core(core::Record::Tasks(tasks::Stored::Live(task)))
                if task.executor == tasks::Executor::Agent { charter: 2 } =>
            {
                Some(task.number)
            }
            root::Record::Core(_) | root::Record::Connector { .. } => None,
        })
        .expect("durable delegate");
    world.restart_with(world::two_charters(143).0);
    world.grant_account(2);
    rejoin(&mut world);
    let assignment = world.assigned.as_ref().expect("restored delegate assigned");
    assert_eq!(assignment.task, task);
    assert_eq!(assignment.charter, 2);
    assert_eq!(assignment.run.policy.instructions.as_ref(), b"Research the question.");
    assert_eq!(assignment.grant.account, 2);
}

fn rejoin(world: &mut World) {
    world.send(root::Event::Core(core::Event::Fleet(jig_domain_fleet::Event::Hello {
        channel: skein_lib::Token::new(7),
        hello: jig_domain_fleet::Hello {
            bounds: jig_core_world::fixtures::bounds(),
            stop_bound: skein_lib::Duration::from_millis(100),
            slots: 2,
            workstreams: Box::new([]),
            hosting: Box::new([]),
        },
    })));
}

fn set_goal(world: &mut World, charter: u32) {
    world.send(root::Event::Core(core::Event::People(people::Event::Ask {
        reply_to: ReplyTo::new(Token::new(9000)),
        sign_in: world.sign_in.expect("signed in party"),
        key: [31; 16],
        ask: people::Ask::SetGoal {
            project: 1,
            spec: b"Research this goal".as_slice().into(),
            executor: people::GoalExecutor::Charter(charter),
            budget: 20,
            priority: 1,
            deadline: None,
        },
    })));
}

#[test]
fn a_goal_on_the_second_charter_is_admitted_and_runs_it() {
    let (config, limits) = world::two_charters(144);
    let mut world = World::configured(144, false, config, limits);
    world.grant_account(2);
    set_goal(&mut world, 2);
    assert!(
        world
            .people_answers
            .iter()
            .any(|reply| matches!(reply, people::Reply::Outcome(people::Outcome::GoalStarted { .. })))
    );
    let assignment = world.assigned.as_ref().expect("goal assigned");
    assert_eq!(assignment.charter, 2);
    assert_eq!(assignment.run.policy.instructions.as_ref(), b"Research the question.");
    assert_eq!(assignment.grant.account, 2);
}

#[test]
fn a_goal_naming_an_unoffered_charter_is_refused_not_offered() {
    let (config, limits) = world::two_charters(145);
    let mut world = World::configured(145, false, config, limits);
    set_goal(&mut world, 9);
    assert!(
        world.people_answers.iter().any(|reply| matches!(
            reply,
            people::Reply::Outcome(people::Outcome::Refused(people::Refusal::NotOffered))
        ))
    );
}
