const NO_TOOLS: u64 = 0;
use jig_core_world::{
    effects::World,
    fixtures::{ask_as, live_row},
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
    second.dependencies = Box::new([0]);
    create(&mut world, Box::new([first, second]));
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Created { first: FIRST, last: LAST }))
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
    first.dependencies = Box::new([1]);
    let mut second = member(b"second", 20);
    second.dependencies = Box::new([0]);
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
            proposable: None
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
