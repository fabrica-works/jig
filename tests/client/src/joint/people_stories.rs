//! People act on the view; the engine and workers decide the outcomes
//! (domain/people.md, 11; domain/engine.md, 15).
use super::*;

#[test]
fn a_person_s_words_reach_a_running_run() {
    let mut world = super::world(super::turn());
    let client = world.client(1, crate::stories::conversation());
    let within = Duration::from_secs(1);
    world.person(
        client,
        vec![
            Step::See { find: Find::named(Role::Button, b"Sign in with Test"), within },
            Step::Press { find: Find::named(Role::Button, b"Sign in with Test") },
            Step::See { find: Find::named(Role::TextArea, b"First words"), within },
            Step::Type { find: Find::named(Role::TextArea, b"First words"), words: b"Hello".as_slice().into() },
            Step::Press { find: Find::named(Role::Button, b"Start a conversation") },
            Step::See { find: Find::text(b"Hello"), within },
            Step::Pause { span: Duration::from_millis(450) },
            Step::See { find: Find::named(Role::Button, b"Write"), within },
            Step::Press { find: Find::named(Role::Button, b"Write") },
            Step::Type {
                find: Find::named(Role::TextArea, b"Your answer"),
                words: b"Please continue".as_slice().into(),
            },
            Step::Press { find: Find::named(Role::Button, b"Confirm") },
            Step::See { find: Find::text(b"Please continue"), within },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    super::offered_acts::task(&mut world, client, 1);
    let peers = world.front().session.driver.world.peers.as_ref().expect("worker peer");
    let assigned = peers.workers[0].seen.first().expect("assigned chat");
    assert!(peers.workers[0].messages.iter().any(|(_, task, attempt, words)| *task == assigned.task
        && *attempt == assigned.attempt
        && words.as_ref() == b"Please continue"));
    assert_eq!(
        world
            .front()
            .session
            .driver
            .world
            .store
            .rows
            .values()
            .filter(|row| matches!(
                row,
                jig_test_domain::Record::Core(jig_core::Record::People(people::Stored::Answer { .. }))
            ))
            .count(),
        2
    );
}

#[test]
fn a_person_stops_a_run_and_releases_it() {
    let mut world = super::world(super::turn());
    let client = world.client(1, crate::stories::conversation());
    let within = Duration::from_secs(1);
    world.person(
        client,
        vec![
            Step::See { find: Find::named(Role::Button, b"Sign in with Test"), within },
            Step::Press { find: Find::named(Role::Button, b"Sign in with Test") },
            Step::See { find: Find::named(Role::TextArea, b"First words"), within },
            Step::Type { find: Find::named(Role::TextArea, b"First words"), words: b"Hello".as_slice().into() },
            Step::Press { find: Find::named(Role::Button, b"Start a conversation") },
            Step::See { find: Find::text(b"Hello"), within },
            Step::Pause { span: Duration::from_millis(450) },
            Step::See { find: Find::named(Role::Button, b"Stop"), within },
            Step::Press { find: Find::named(Role::Button, b"Stop") },
            Step::See { find: Find::text(b"Stop the live run and hold its task"), within },
            Step::Press { find: Find::named(Role::Button, b"Confirm") },
            Step::See { find: Find::text(b"Held stopped"), within },
            Step::See { find: Find::exactly_named(Role::Button, b"Release"), within },
            Step::Press { find: Find::exactly_named(Role::Button, b"Release") },
            Step::Press { find: Find::named(Role::Button, b"Confirm") },
            Step::See { find: Find::text(b"Released"), within },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    super::offered_acts::task(&mut world, client, 1);
    let peers = world.front().session.driver.world.peers.as_ref().expect("worker peer");
    assert_eq!(peers.workers[0].seen.len(), 2);
    let before = &peers.workers[0].seen[0];
    let after = &peers.workers[0].seen[1];
    assert_eq!(before.task, after.task);
    assert_ne!(before.attempt, after.attempt);
    assert_eq!(
        world
            .front()
            .session
            .driver
            .world
            .store
            .rows
            .values()
            .filter(|row| matches!(
                row,
                jig_test_domain::Record::Core(jig_core::Record::People(people::Stored::Answer { .. }))
            ))
            .count(),
        3
    );
}
