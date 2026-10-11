//! Actual connector documents fill the procedure, effect and Hold slots.
use super::*;

fn procedure_configuration(configuration: &mut jig_test_domain::Config, owner: &jig_test_domain::Limits) {
    let kinds: Box<[jig_core_authority::Executor]> =
        Box::new([jig_core_authority::Executor::Charter(1), jig_core_authority::Executor::Procedure(1)]);
    configuration.core.settings.chat_authority.delegation.kinds.clone_from(&kinds);
    let mut rules = configuration.core.authority.rules().clone();
    rules.ceiling.delegation.kinds.clone_from(&kinds);
    let mut policy = configuration.core.authority.policy(1).expect("actual procedure policy").clone();
    policy.ceiling.delegation.kinds.clone_from(&kinds);
    for role in &mut policy.roles {
        role.authority.delegation.kinds.clone_from(&kinds);
    }
    let mut checked = jig_core_authority::Domain::new(rules, owner.core.authority).expect("declared executor bounds");
    let mut out = skein_lib::Queue::with_capacity(jig_core_authority::POLICY_MAX_OUT);
    jig_core_authority::step(&mut checked, jig_core_authority::Event::Policy { project: 1, policy }, &mut out);
    assert_eq!(out.pop(), Some(jig_core_authority::PolicyFact::Added { project: 1 }));
    configuration.core.authority = checked;
    configuration.first.procedures = Box::new([jig_test_connector::ProcedureSpec {
        number: 1,
        actions: Box::new([jig_test_connector::ProcedureAction::Stall]),
        max_steps: 1,
        stall: Duration::from_secs(5),
    }]);
}

#[test]
fn a_persons_procedure_goal_shows_the_actual_live_connector_panel_on_its_task_page() {
    let mut world = configured(8524, vec![], procedure_configuration);
    let owner = world.client_with(
        1,
        jig_test_client_domain::Address::Jig(jig_client::address::Address::Project {
            project: 1,
            section: jig_client::address::ProjectSection::Goals,
        }),
        receiving_limits(),
        client,
    );
    let within = Duration::from_secs(1);
    world.person(
        owner,
        vec![
            Step::See { find: Find::exactly_named(Role::Button, b"Sign in with Test"), within },
            Step::Press { find: Find::exactly_named(Role::Button, b"Sign in with Test") },
            Step::See { find: Find::exactly_named(Role::TextArea, b"Goal specification"), within },
            Step::Type {
                find: Find::exactly_named(Role::TextArea, b"Goal specification"),
                words: b"a live procedure".as_slice().into(),
            },
            Step::Press { find: Find::exactly_named(Role::Button, b"Connector 1, procedure 1") },
            Step::Type { find: Find::exactly_named(Role::TextField, b"Goal budget"), words: b"50".as_slice().into() },
            Step::Press { find: Find::exactly_named(Role::Button, b"Set goal") },
            Step::Press { find: Find::exactly_named(Role::Button, b"Confirm") },
            Step::See { find: Find::text(b"Task 1: a live procedure"), within },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed, "{outcome:?}");
    assert!(outcome.judged.0 > 0);
    let tasks: Vec<_> = world
        .front()
        .session
        .driver
        .world
        .store
        .rows
        .values()
        .filter_map(|row| match row {
            jig_test_domain::Record::Core(jig_core::Record::People(people::Stored::Answer {
                outcome: people::Outcome::GoalStarted { task },
                ..
            })) => Some(*task),
            _ => None,
        })
        .collect();
    assert_eq!(tasks.len(), 1);
    let task = tasks[0];
    assert!(
        world
            .front()
            .session
            .driver
            .world
            .procedure_inputs
            .iter()
            .any(|(_, connector, code, context)| *connector == 1 && *code == 1 && context.task == task)
    );
    eprintln!(
        "actual task connector rows: {:?}",
        world
            .front()
            .session
            .driver
            .world
            .store
            .rows
            .values()
            .filter(|row| matches!(row, jig_test_domain::Record::Connector { number: 1, .. }))
            .collect::<Vec<_>>()
    );
    world.person(
        owner,
        vec![
            Step::Go {
                address: jig_test_client_domain::Address::Jig(jig_client::address::Address::Task {
                    task,
                    section: jig_client::address::TaskSection::Activity,
                }),
            },
            Step::See { find: Find::text(b"Procedure panel"), within },
            Step::See { find: Find::text(b"Procedure program: 1"), within },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed, "{outcome:?}");
    assert!(outcome.judged.0 > 0);
    assert!(world.front().session.driver.world.peers.as_ref().expect("actual worker").workers[0].seen.is_empty());
}
