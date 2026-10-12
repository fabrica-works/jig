use jig_core_world::{effects::World, reads, stories::reads::request};
use jig_domain_core::{self as core, documents};
use jig_domain_fleet as fleet;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_domain_views as views;
use jig_test_domain as root;
use skein_lib::{Duration, Token};

#[test]
fn a_partys_request_after_the_stop_is_answered_not_ready() {
    let mut world = World::new(160, false);
    world.stop();
    let commits = world.store.applied;
    request(&mut world, 55, people::Ask::StartChat { project: 1, words: b"later".as_slice().into() });
    assert_eq!(world.people_answers.last(), Some(&people::Reply::Again(people::Again::NotReady)));
    assert_eq!(world.store.applied, commits);
}

#[test]
fn a_turn_after_the_stop_is_answered_busy_and_not_committed() {
    let mut world = World::new(161, false);
    world.stop();
    let commits = world.store.applied;
    world.turn(1, 1, None, b"a turn the engine will not take");
    assert_eq!(world.store.applied, commits, "nothing is committed after the stop");
    assert_eq!(world.busy_turns, [(world.task(), world.attempt(), 1)]);
}

#[test]
fn an_answer_after_the_stop_is_left_unacknowledged() {
    let mut world = World::new(162, false);
    world.stop();
    let commits = world.store.applied;
    world.end(tasks::End::Parked, 0);
    assert!(!world.stop_deliveries.iter().any(|held| matches!(held, core::Held::Acknowledge { .. })));
    assert_eq!(world.store.applied, commits);
}

#[test]
fn a_call_after_the_stop_is_answered_busy() {
    let mut world = World::new(163, false);
    world.stop();
    let commits = world.store.applied;
    world.send(root::Event::Core(core::Event::Fleet(fleet::Event::Relay {
        channel: Token::new(7),
        run: Token::new(world.task()),
        attempt: Token::new(world.attempt()),
        call: fleet::Call {
            name: b"call".as_slice().into(),
            tool: b"tool".as_slice().into(),
            writes: true,
            input: Box::new([]),
            deadline: Duration::from_secs(2),
        },
    })));
    assert_eq!(world.busy_calls, [Box::<[u8]>::from(b"call".as_slice())]);
    assert_eq!(world.store.applied, commits);
}

#[test]
fn a_new_watch_after_the_stop_is_ended_as_stopping() {
    let mut world = World::new(164, false);
    world.stop();
    world.send(root::Event::Core(core::Event::Watch {
        watcher: Token::new(801),
        sign_in: world.sign_in.expect("signed in"),
        subject: documents::Subject::Task { task: world.task() },
        window: documents::Window { rows: 1, after: None },
    }));
    assert!(world.stop_deliveries.iter().any(
        |held| matches!(held, core::Held::WatchEnded { watcher, why: views::End::Stopping } if watcher.raw() == 801)
    ));
}

#[test]
fn no_run_is_placed_after_the_stop() {
    let mut world = World::new(165, false);
    world.stop();
    let assignments = world.assignments.len();
    let commits = world.store.applied;
    world.send(root::Event::Timer(core::Timer::Tasks));
    world.send(root::Event::Timer(core::Timer::Fleet));
    request(&mut world, 56, people::Ask::StartChat { project: 1, words: b"later".as_slice().into() });
    assert_eq!(world.assignments.len(), assignments);
    assert_eq!(world.store.applied, commits);
}

#[test]
fn a_read_after_the_stop_is_answered_as_before() {
    let mut world = jig_core_world::stories::reads::world(166);
    world.stop();
    let task = world.task();
    reads::ask(&mut world, 1, core::reads::Read::Task { task });
    assert!(matches!(reads::latest(&world), core::reads::Answer::Document(_)));
}

#[test]
fn a_connector_event_after_the_stop_is_handed_back_busy() {
    let mut world = World::new(167, false);
    world.stop();
    let commits = world.store.applied;
    world.send(root::Event::Connector {
        number: 1,
        event: jig_test_connector::Event::ApplicationDrop { owner: Token::new(9) },
    });
    assert!(matches!(world.busy_events.as_slice(), [root::Event::Connector { number: 1, .. }]));
    assert_eq!(world.store.applied, commits);
}
