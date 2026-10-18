use jig_core_world::fixtures::two_projects;
use jig_core_world::{effects::World, world};
use jig_domain_authority as authority;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{ReplyTo, Token};

fn ask(world: &mut World, key: u8, ask: people::Ask) {
    world.send(root::Event::Core(core::Event::People(people::Event::Ask {
        reply_to: ReplyTo::new(Token::new(8000 + u64::from(key))),
        sign_in: world.sign_in.expect("signed in owner"),
        key: [key; 16],
        ask,
    })));
}

fn ledger(world: &World, funder: tasks::Funder) -> &tasks::FundingRecord {
    let root::Record::Core(core::Record::Tasks(tasks::Stored::Ledger(row))) =
        &world.store.rows[&root::Key::Core(core::Key::Tasks(tasks::Key::Ledger(funder)))]
    else {
        panic!("funding ledger")
    };
    row
}

#[test]
fn two_projects_with_different_spends_fund_their_own_periods() {
    let (config, limits) = two_projects(601);
    let mut world = World::configured(601, false, config, limits);
    world.send(root::Event::Core(core::Event::Period { project: 1, period: 2, budget: 800, next: None }));
    world.send(root::Event::Core(core::Event::Period { project: 2, period: 9, budget: 600, next: None }));
    world.restart_with(two_projects(601).0);
    ask(&mut world, 1, people::Ask::SetPool { project: 1, person: 1, budget: 100 });
    ask(&mut world, 2, people::Ask::SetPool { project: 2, person: 1, budget: 200 });
    let first = tasks::Funder::Period { project: 1, period: 2 };
    let second = tasks::Funder::Period { project: 2, period: 9 };
    assert_eq!(ledger(&world, first).numbers.budget, 800);
    assert_eq!(ledger(&world, second).numbers.budget, 600);
    assert_eq!(ledger(&world, tasks::Funder::Pool { project: 1, person: 1, period: 2 }).parent, Some(first));
    assert_eq!(ledger(&world, tasks::Funder::Pool { project: 2, person: 1, period: 9 }).parent, Some(second));
}

#[test]
fn a_chat_takes_the_most_that_fits_under_its_persons_role() {
    let mut config = world::config(602);
    world::edit_role(&mut config.core, |role| {
        role.authority.tools = authority::Tools(513);
        role.authority.budget.spend = 80;
        role.period_spend = 90;
        role.authority.delegation.tasks = 3;
    });
    let mut limits = world::limits();
    limits.core.tasks.tree_tasks = 4;
    jig_core_world::peers::journal(&mut limits);
    let mut world = World::configured(602, false, config, limits);
    let remainder_chat = jig_core_world::fixtures::chat(&world, 1, b"use the remainder".as_slice().into());
    ask(&mut world, 6, remainder_chat);
    let task = 2;
    let root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row))) =
        &world.store.rows[&root::Key::Core(core::Key::Tasks(tasks::Key::Live(task)))]
    else {
        panic!("live chat")
    };
    assert_eq!(row.authority.tools, tasks::Tools(513));
    assert_eq!(row.authority.budget.spend, 10);
    assert_eq!(row.authority.delegation.depth, 1);
    assert_eq!(row.authority.delegation.tasks, 2);
    assert_eq!(row.numbers.budget, 10);
}

#[test]
fn a_policy_change_reaches_the_next_pool_budget() {
    let mut world = World::configured(603, false, world::config(603), world::limits());
    let initial = tasks::Funder::Pool { project: 1, person: 1, period: 1 };
    assert_eq!(ledger(&world, initial).numbers.budget, 500);
    ask(&mut world, 3, people::Ask::ChangePolicy { project: 1, change: people::PolicyChange::Role(changed_role()) });
    world.send(root::Event::Core(core::Event::Period { project: 1, period: 2, budget: 1000, next: None }));
    let funded_goal = jig_core_world::fixtures::goal(
        &world,
        1,
        b"next period".as_slice().into(),
        people::TaskExecutor::Agent { charter: 1 },
        jig_core_world::fixtures::Goal { budget: 20, priority: 1, ..Default::default() },
    );
    ask(&mut world, 4, funded_goal);
    assert_eq!(ledger(&world, tasks::Funder::Pool { project: 1, person: 1, period: 2 }).numbers.budget, 175);
    assert_eq!(ledger(&world, initial).numbers.budget, 500);
}

fn changed_role() -> people::PolicyRole {
    people::PolicyRole {
        number: 0,
        authority: people::Authority {
            tools: 1023,
            grants: Box::new([]),
            delegation: people::Delegation {
                kinds: Box::new([people::Executor::Charter(1), people::Executor::Procedure(1)]),
                tasks: 2,
                depth: 2,
            },
            spend: 100,
            deadline: None,
            notes: 0,
            note_resources: Box::new([]),
        },
        period_spend: 175,
        requests: authority::Requests::ALL.0,
        decides: authority::Proposals::ALL.0,
    }
}
