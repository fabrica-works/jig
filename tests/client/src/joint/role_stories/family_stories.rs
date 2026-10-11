//! Actual connector documents fill the procedure, effect and Hold slots.
use super::*;

fn family_limits() -> jig_test_client_domain::Limits {
    let mut limits = receiving_limits();
    limits.first.connector.paths = 2;
    limits.first.connector.segments = 2;
    limits.first.connector.segment_bytes = 16;
    limits
}
fn family_client(seed: u64) -> (jig_test_client_domain::Domain, jig_test_client_view::View) {
    crate::test::new(&family_limits(), seed)
}

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
    let mut world = configured_with_bounds(8524, vec![], 1, procedure_configuration, family_limits());
    let owner = world.client_with(
        1,
        jig_test_client_domain::Address::Jig(jig_client::address::Address::Project {
            project: 1,
            section: jig_client::address::ProjectSection::Goals,
        }),
        family_limits(),
        family_client,
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
        .filter_map(|row| {
            if let jig_test_domain::Record::Core(jig_core::Record::People(people::Stored::Answer {
                outcome: people::Outcome::GoalStarted { task },
                ..
            })) = row
            {
                Some(*task)
            } else {
                None
            }
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
    world.person(
        owner,
        vec![
            Step::Go {
                address: jig_test_client_domain::Address::Jig(jig_client::address::Address::Task {
                    task,
                    section: jig_client::address::TaskSection::Activity,
                }),
            },
            Step::See { find: Find::exactly_named(Role::Region, b"Procedure panel"), within },
            Step::See { find: Find::text(b"Procedure program: 1"), within },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed, "{outcome:?}");
    assert!(outcome.judged.0 > 0);
    assert!(world.front().session.driver.world.peers.as_ref().expect("actual worker").workers[0].seen.is_empty());
}

fn effect_configuration(configuration: &mut jig_test_domain::Config, owner: &jig_test_domain::Limits) {
    let grant = jig_core_authority::Grant {
        connector: 1,
        kind: 5,
        pattern: jig_core_authority::Pattern {
            segments: Box::new([]),
            last: jig_core_authority::Last::Open(Box::new([])),
        },
    };
    configuration.core.settings.chat_authority.grants = Box::new([grant.clone()]);
    let mut rules = configuration.core.authority.rules().clone();
    rules.ceiling.grants = Box::new([grant.clone()]);
    let mut policy = configuration.core.authority.policy(1).expect("actual effect policy").clone();
    policy.ceiling.grants = Box::new([grant.clone()]);
    policy.roles[0].authority.grants = Box::new([grant]);
    let mut checked = jig_core_authority::Domain::new(rules, owner.core.authority).expect("declared grant bounds");
    let mut out = skein_lib::Queue::with_capacity(jig_core_authority::POLICY_MAX_OUT);
    jig_core_authority::step(&mut checked, jig_core_authority::Event::Policy { project: 1, policy }, &mut out);
    assert_eq!(out.pop(), Some(jig_core_authority::PolicyFact::Added { project: 1 }));
    configuration.core.authority = checked;
}

#[test]
fn a_proposed_effect_shows_its_real_connector_description_in_the_proposal_card() {
    let mut world =
        configured_with_bounds(8526, vec![Box::new([Script::Wait])], 1, effect_configuration, family_limits());
    let owner = world.client_with(1, crate::stories::conversation(), family_limits(), family_client);
    world.person(owner, start(b"describe its effect"));
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    let assignment =
        world.front().session.driver.world.peers.as_ref().expect("actual worker").workers[0].seen[0].clone();
    let now = world.now();
    world.front().session.driver.send(
        now,
        clock(now),
        jig_test_domain::Event::EffectCall {
            to: skein_lib::ReplyTo::new(Token::new(9860)),
            key: jig_core::CallKey { task: assignment.task, attempt: assignment.attempt, completion: 1, position: 0 },
            number: 1,
            effect: jig_core_world::effects::World::effect(),
            deadline: skein_lib::Wall::from_nanos(clock(now).as_nanos() + 1_000_000_000),
            proposal: Some(b"ask the holder".as_slice().into()),
        },
    );
    let within = Duration::from_secs(1);
    world.person(
        owner,
        vec![
            Step::Go { address: crate::stories::inbox() },
            Step::See { find: Find::text(b"ask the holder"), within },
            Step::See { find: Find::text(b"Proposed effect"), within },
            Step::See { find: Find::exactly_named(Role::Region, b"Effect description"), within },
            Step::See { find: Find::text(b"Effect kind: 5"), within },
            Step::See { find: Find::text(b"Effect purpose: 19"), within },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed, "{outcome:?}");
    assert!(outcome.judged.0 > 0);
    let engine = &world.front().session.driver.world;
    assert_eq!(engine.answers.len(), 1);
    assert!(matches!(engine.answers[0].2, jig_core::CallPart::Proposed { .. }));
    assert!(
        !engine.store.rows.values().any(|row| matches!(
            row,
            jig_test_domain::Record::Connector { record: jig_test_connector::Record::Outbox(_), .. }
        )),
        "a description does not make the proposed effect"
    );
    assert!(engine.systems[0].observed().is_empty());
}
