const NO_TOOLS: u64 = 0;
const OWNER_ROLE: u32 = 0;
use jig_core_world::{
    effects::World,
    fixtures::{ask_as, live_row, refusal},
    stories::offers::{no_holder_fixture, world_with},
};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
const PARTY: u64 = 1;
const FIRST: u64 = 2;
const LAST: u64 = 3;
const PROJECT: u32 = 1;
fn batch_world(seed: u64) -> World {
    world_with(seed, Some(people::Role::Member), |limits| {
        limits.core.tasks.batch = 2;
        limits.core.authority.batch = 2;
        limits.core.people.batch = 2;
        limits.core.tasks.tasks = 8;
        limits.core.tasks.project_tasks = 8;
    })
}
fn member(words: &[u8], spend: u64) -> people::Member {
    people::Member {
        executor: people::TaskExecutor::Agent { charter: 1 },
        words: words.into(),
        parameters: Box::new([]),
        inputs: Box::new([]),
        budget: people::Budget::Named,
        named: Box::new([]),
        contract: people::Contract::Report { words: 16 },
        authority: people::Authority {
            tools: NO_TOOLS,
            grants: Box::new([]),
            delegation: people::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 },
            spend,
            deadline: None,
            notes: 0,
            note_resources: Box::new([]),
        },
        wake: people::WakePolicy {
            words: people::WakeRule::Immediate,
            notices: people::WakeRule::Never,
            news: people::WakeRule::Never,
            results: people::ResultsWake::Never,
            questions: false,
            answers: false,
            timers: false,
        },
        dependencies: Box::new([]),
        tracked: None,
        label: 0,
        flags: people::Flags::PLAIN,
    }
}
fn create(world: &mut World, batch: Box<[people::Member]>) {
    ask_as(world, world.sign_in.expect("member signed in"), 31, people::Ask::Create { project: PROJECT, batch });
}
#[test]
fn a_partys_batch_of_two_agents_tasks_is_made_as_their_roots() {
    let mut world = batch_world(301);
    let mut first = member(b"gather the facts", 20);
    first.tracked = Some(3);
    first.label = 41;
    let mut second = member(b"write the summary", 30);
    second.label = 42;
    second.dependencies = Box::new([people::Dependency::Batch(0)]);
    create(&mut world, Box::new([first, second]));
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Created { first: FIRST, last: LAST, given: None }))
    );
    let first = live_row(&world, FIRST);
    assert_eq!(first.requester, tasks::Party::Person(PARTY));
    assert_eq!((first.tracked, first.label, first.numbers.budget), (Some(3), 41, 20));
    assert_eq!(first.flags, tasks::Flags::PLAIN);
    let second = live_row(&world, LAST);
    assert_eq!((second.label, second.dependencies.as_ref()), (42, [FIRST].as_slice()));
    assert_eq!(second.funder, tasks::Funder::Pool { project: PROJECT, person: PARTY, period: 1 });
}
#[test]
fn a_partys_batch_with_a_dependency_cycle_is_refused_naming_its_member() {
    let mut world = batch_world(302);
    let mut first = member(b"first", 20);
    first.dependencies = Box::new([people::Dependency::Batch(1)]);
    let mut second = member(b"second", 20);
    second.dependencies = Box::new([people::Dependency::Batch(0)]);
    create(&mut world, Box::new([first, second]));
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Refused(people::Refusal::NotWellFormed {
            at: 0,
            why: people::Malformed::Cycle
        })))
    );
    assert_eq!(
        world
            .store
            .rows
            .keys()
            .filter(|key| matches!(key, jig_test_domain::Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(_)))))
            .count(),
        1
    );
}
#[test]
fn a_partys_batch_is_refused_while_nobody_holds_its_escalations() {
    let (configuration, limits) = no_holder_fixture(303);
    let mut world = World::configured(303, false, configuration, limits);
    world.send(jig_test_domain::Event::Core(jig_domain_core::Event::People(people::Event::Roles {
        project: PROJECT,
        holdings: Box::new([people::Holding { person: PARTY, role: people::Role::Member }]),
    })));
    create(&mut world, Box::new([member(b"first", 20)]));
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Refused(people::Refusal::NoEscalationHolder)))
    );
}
#[test]
fn a_partys_batch_beyond_their_pool_is_refused_naming_the_budget() {
    let mut world = batch_world(304);
    create(&mut world, Box::new([member(b"costly", 101)]));
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Authority {
            lacking: people::Lack::Budget { needs: 101, left: 100 },
            proposable: Some(people::Accepter::Role(OWNER_ROLE))
        })))
    );
}
#[test]
fn a_partys_batch_naming_an_unknown_charter_is_not_offered() {
    let mut world = batch_world(305);
    let mut task = member(b"first", 20);
    task.executor = people::TaskExecutor::Agent { charter: 99 };
    create(&mut world, Box::new([task]));
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Refused(people::Refusal::NotOffered)))
    );
}

const MEMBER_ROLE: u32 = 2;
const CONNECTOR: u16 = 1;
const PROCEDURE: u32 = 1;
const CHARTER: u32 = 1;
const OTHER_PARTY: u64 = 2;
const CHAT: u64 = 1;
const SECOND_MEMBER: u32 = 1;
const FIRST_MEMBER: u32 = 0;
const SIGN_IN_REQUEST: u64 = 8300;
const INBOX_REQUEST: u64 = 8301;
const PARAMETER: u32 = 13;
const WRITE_KIND: u16 = 5;

fn finish(world: &mut World) {
    jig_core_world::fixtures::finish_chat(world, 0, b"");
}

fn person_world(seed: u64) -> World {
    jig_core_world::stories::offers::world_with_policy(seed, Some(people::Role::Member), |limits, rules, policy| {
        limits.core.people.batch = 2;
        limits.core.tasks.batch = 2;
        limits.core.authority.batch = 2;
        limits.core.tasks.tasks = 8;
        limits.core.tasks.project_tasks = 8;
        limits.core.tasks.executor_kinds = 3;
        limits.core.authority.executors = 3;
        let kinds: Box<[jig_domain_authority::Executor]> = Box::new([
            jig_domain_authority::Executor::Charter(CHARTER),
            jig_domain_authority::Executor::Procedure(PROCEDURE),
            jig_domain_authority::Executor::Role(MEMBER_ROLE),
        ]);
        rules.ceiling.delegation.kinds.clone_from(&kinds);
        policy.ceiling.delegation.kinds.clone_from(&kinds);
        for role in &mut policy.roles {
            role.authority.delegation.kinds.clone_from(&kinds);
        }
    })
}
fn other_party(world: &mut World, role: people::Role) {
    world.send(jig_test_domain::Event::Core(jig_domain_core::Event::SignIn {
        reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(SIGN_IN_REQUEST)),
        identity: people::Identity {
            key: people::IdentityKey { provider: 0, subject: b"second-party".as_slice().into() },
            login: b"second".as_slice().into(),
            name: b"Second".as_slice().into(),
        },
    }));
    world.send(jig_test_domain::Event::Core(jig_domain_core::Event::People(people::Event::Roles {
        project: PROJECT,
        holdings: Box::new([
            people::Holding { person: PARTY, role: people::Role::Member },
            people::Holding { person: OTHER_PARTY, role },
        ]),
    })));
}
fn person_inbox(world: &mut World) -> Vec<u64> {
    use jig_domain_core::{documents, reads};
    jig_core_world::reads::ask(
        world,
        INBOX_REQUEST,
        reads::Read::Inbox {
            group: reads::Group::PersonTasks,
            filter: reads::Filter { project: Some(PROJECT), goal: None },
            from: None,
        },
    );
    let reads::Answer::Page(page) = jig_core_world::reads::latest(world) else { panic!("person-task page") };
    page.rows
        .iter()
        .map(|row| {
            let reads::PageRow::Inbox(entry) = row else { panic!("inbox row") };
            let documents::TaskLink::Live(chip) = &entry.task else { panic!("live person task") };
            chip.number
        })
        .collect()
}
#[test]
fn a_partys_batch_of_an_agents_task_and_a_person_task_is_made() {
    let mut world = person_world(306);
    let agent = member(b"collect", 20);
    let mut person = member(b"check", 20);
    person.executor = people::TaskExecutor::Person(people::PersonAddress::Role(MEMBER_ROLE));
    person.dependencies = Box::new([people::Dependency::Batch(FIRST_MEMBER)]);
    create(&mut world, Box::new([agent, person]));
    assert_eq!(live_row(&world, LAST).dependencies.as_ref(), &[FIRST]);
    assert_eq!(live_row(&world, LAST).person_wait, None);
    assert_eq!(person_inbox(&mut world), Vec::<u64>::new());
    finish(&mut world);
    assert_eq!(world.assigned.as_ref().expect("batch agent").task, FIRST);
    finish(&mut world);
    assert_eq!(live_row(&world, LAST).executor, tasks::Executor::Person(tasks::PersonAddress::Role(MEMBER_ROLE)));
    assert_eq!(person_inbox(&mut world), vec![LAST]);
}
#[test]
fn a_budget_as_allowed_beside_named_amounts_is_named_in_the_answer() {
    let mut world = batch_world(307);
    let mut allowed = member(b"remaining", 999);
    allowed.budget = people::Budget::AsAllowed { cap: None };
    create(&mut world, Box::new([member(b"named", 30), allowed]));
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Created {
            first: FIRST,
            last: LAST,
            given: Some(people::Given { task: LAST, amount: 70 })
        }))
    );
    assert_eq!(live_row(&world, LAST).numbers.budget, 70);
}
#[test]
fn a_budget_as_allowed_that_comes_to_nothing_is_refused() {
    let mut world = batch_world(308);
    let mut allowed = member(b"remaining", 0);
    allowed.budget = people::Budget::AsAllowed { cap: None };
    create(&mut world, Box::new([member(b"all", 100), allowed]));
    refusal(
        &world,
        people::Refusal::Authority { lacking: people::Lack::Budget { needs: 101, left: 100 }, proposable: None },
    );
}
#[test]
fn a_batch_where_two_tasks_give_as_allowed_is_not_well_formed() {
    let mut world = batch_world(309);
    let mut allowed = member(b"remaining", 0);
    allowed.budget = people::Budget::AsAllowed { cap: None };
    create(&mut world, Box::new([allowed.clone(), allowed]));
    refusal(&world, people::Refusal::NotWellFormed { at: SECOND_MEMBER, why: people::Malformed::AsAllowedTwice });
}
#[test]
fn a_partys_batch_with_a_conversation_on_a_procedure_is_not_well_formed() {
    let mut world = batch_world(310);
    let mut procedure = member(b"procedure", 10);
    procedure.executor = people::TaskExecutor::Procedure { connector: CONNECTOR, code: PROCEDURE };
    procedure.flags.conversation = true;
    create(&mut world, Box::new([procedure]));
    refusal(&world, people::Refusal::NotWellFormed { at: FIRST_MEMBER, why: people::Malformed::Conversation });
}
#[test]
fn a_partys_procedure_task_is_offered_and_its_parameters_reach_its_connector() {
    let mut world = batch_world(311);
    world.hold_procedures = true;
    let mut procedure = member(b"procedure", 10);
    procedure.executor = people::TaskExecutor::Procedure { connector: CONNECTOR, code: PROCEDURE };
    let parameters = Box::new([people::Parameter::Number { name: PARAMETER, value: 19 }]);
    procedure.parameters = parameters;
    create(&mut world, Box::new([procedure]));
    let (_, connector, code, context) = world.procedure_inputs.last().expect("procedure delivered");
    assert_eq!(
        (*connector, *code, context.task),
        (CONNECTOR, u16::try_from(PROCEDURE).expect("procedure fits u16"), FIRST)
    );
    assert_eq!(context.spec.parameters.as_ref(), &[tasks::Parameter::Number { name: PARAMETER, value: 19 }]);
}
#[test]
fn a_member_whose_charter_the_role_lacks_is_refused_naming_it() {
    let mut world =
        jig_core_world::stories::offers::world_with_policy(312, Some(people::Role::Member), |_, _, policy| {
            policy
                .roles
                .iter_mut()
                .find(|role| role.number == MEMBER_ROLE)
                .expect("member")
                .authority
                .delegation
                .kinds = Box::new([jig_domain_authority::Executor::Procedure(PROCEDURE)]);
        });
    create(&mut world, Box::new([member(b"agent", 10)]));
    refusal(
        &world,
        people::Refusal::Authority {
            lacking: people::Lack::TaskKind { executor: Some(people::TaskExecutor::Agent { charter: CHARTER }) },
            proposable: Some(people::Accepter::Role(OWNER_ROLE)),
        },
    );
}
#[test]
fn a_members_dependency_outside_their_tree_is_unknown() {
    let mut world = batch_world(313);
    other_party(&mut world, people::Role::Member);
    let mut task = member(b"dependent", 10);
    task.dependencies = Box::new([people::Dependency::Existing(CHAT)]);
    create(&mut world, Box::new([task]));
    refusal(&world, people::Refusal::Unknown);
}
#[test]
fn a_members_input_outside_their_tree_is_unknown() {
    let mut world = batch_world(314);
    finish(&mut world);
    other_party(&mut world, people::Role::Member);
    let mut task = member(b"input", 10);
    task.inputs = Box::new([CHAT]);
    create(&mut world, Box::new([task]));
    refusal(&world, people::Refusal::Unknown);
}
#[test]
fn a_maintainers_dependency_on_any_project_task_is_made() {
    let mut world = world_with(315, Some(people::Role::Maintainer), |limits| {
        limits.core.tasks.tasks = 8;
        limits.core.tasks.project_tasks = 8;
    });
    other_party(&mut world, people::Role::Maintainer);
    let mut task = member(b"dependent", 0);
    task.dependencies = Box::new([people::Dependency::Existing(CHAT)]);
    create(&mut world, Box::new([task]));
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Created { first: FIRST, last: FIRST, given: None }))
    );
    assert_eq!(live_row(&world, FIRST).dependencies.as_ref(), &[CHAT]);
    assert_eq!(live_row(&world, FIRST).requester, tasks::Party::Person(OTHER_PARTY));
}
#[test]
fn a_named_resource_is_narrowed_to_the_new_tasks_number() {
    let mut world = batch_world(316);
    let mut task = member(b"named", 10);
    task.named = Box::new([people::Grant {
        connector: CONNECTOR,
        kind: WRITE_KIND,
        pattern: people::Pattern { segments: Box::new([]), last: people::Last::Open(b"new-".as_slice().into()) },
    }]);
    create(&mut world, Box::new([task]));
    assert_eq!(
        live_row(&world, FIRST).authority.grants.as_ref(),
        &[tasks::Grant {
            connector: CONNECTOR,
            kind: WRITE_KIND,
            pattern: tasks::Pattern { segments: Box::new([]), last: tasks::Last::Exact(b"new-2".as_slice().into()) }
        }]
    );
}
#[test]
fn a_partys_ended_input_is_loaded_and_kept_as_a_stub() {
    let mut world = batch_world(317);
    finish(&mut world);
    let mut task = member(b"input", 10);
    task.inputs = Box::new([CHAT]);
    create(&mut world, Box::new([task]));
    assert_eq!(live_row(&world, FIRST).spec.inputs.as_ref(), &[CHAT]);
    let key = jig_test_domain::Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Stub(CHAT)));
    assert_eq!(
        world.store.rows.get(&key),
        Some(&jig_test_domain::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Stub(tasks::Stub {
            task: CHAT,
            phase: tasks::Status::Done,
            result: skein_lib::Token::new(CHAT)
        }))))
    );
}

#[test]
fn a_partys_batch_beyond_their_role_is_refused_naming_who_may_accept() {
    let mut world = batch_world(320);
    create(&mut world, Box::new([member(b"beyond role", 150)]));
    refusal(
        &world,
        people::Refusal::Authority {
            lacking: people::Lack::Budget { needs: 150, left: 100 },
            proposable: Some(people::Accepter::Role(OWNER_ROLE)),
        },
    );
    assert!(
        !world
            .store
            .rows
            .contains_key(&jig_test_domain::Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(FIRST))))
    );
}
#[test]
fn a_partys_batch_beyond_the_project_ceiling_says_nobody_could_accept() {
    let mut world =
        jig_core_world::stories::offers::world_with_policy(321, Some(people::Role::Member), |_, _, policy| {
            policy.ceiling.budget.spend = 120;
            for role in &mut policy.roles {
                role.authority.budget.spend = role.authority.budget.spend.min(120);
            }
        });
    create(&mut world, Box::new([member(b"beyond project", 150)]));
    refusal(
        &world,
        people::Refusal::Authority {
            lacking: people::Lack::Budget { needs: 150, left: 100 },
            proposable: Some(people::Accepter::Nobody),
        },
    );
}
#[test]
fn a_partys_batch_no_role_may_accept_says_nobody() {
    let (configuration, limits) = jig_core_world::stories::offers::no_acceptor_fixture(322);
    let mut world = World::configured(322, false, configuration, limits);
    world.send(jig_test_domain::Event::Core(jig_domain_core::Event::People(people::Event::Roles {
        project: PROJECT,
        holdings: Box::new([people::Holding { person: PARTY, role: people::Role::Member }]),
    })));
    create(&mut world, Box::new([member(b"nobody accepts", 150)]));
    refusal(
        &world,
        people::Refusal::Authority {
            lacking: people::Lack::Budget { needs: 150, left: 100 },
            proposable: Some(people::Accepter::Nobody),
        },
    );
}
