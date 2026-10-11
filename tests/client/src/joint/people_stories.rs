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

#[test]
fn a_request_sent_twice_is_made_once_across_an_engine_restart() {
    use jig_fake_person::{Face, Found, TreeFace};
    fn configuration() -> jig_test_domain::Config { jig_core_world::world::config(8510) }
    let mut world = super::configured_world(super::turn(), |front| front.restart_configuration(configuration));
    let client = world.client(1, crate::stories::conversation());
    let within = Duration::from_secs(1);
    world.person(client, vec![
        Step::See { find: Find::named(Role::Button, b"Sign in with Test"), within },
        Step::Press { find: Find::named(Role::Button, b"Sign in with Test") },
        Step::See { find: Find::named(Role::TextArea, b"First words"), within },
        Step::Type { find: Find::named(Role::TextArea, b"First words"), words: b"Hello".as_slice().into() },
        Step::Press { find: Find::named(Role::Button, b"Start a conversation") },
        Step::See { find: Find::text(b"Hello"), within },
        Step::Pause { span: Duration::from_millis(450) },
        Step::See { find: Find::exactly_named(Role::Button, b"Write"), within },
        Step::Press { find: Find::exactly_named(Role::Button, b"Write") },
        Step::Type { find: Find::named(Role::TextArea, b"Your answer"), words: b"Once after restart".as_slice().into() },
    ]);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    let confirm = match TreeFace::new(world.tree(client)).find(&Find::exactly_named(Role::Button, b"Confirm")) {
        Found::Node(id) => id,
        Found::Absent | Found::Pending => panic!("person's ready confirmation"),
    };
    world.fault(crate::Fault::DoublePress { client, node: confirm });
    world.fault(crate::FrontFault::NoAnswer);
    world.fault(crate::FrontFault::Restart { offline: Duration::from_millis(50) });
    world.person(client, vec![Step::See { find: Find::text(b"Once after restart"), within }]);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    let owner = &world.front().session.driver.world;
    assert_eq!(owner.store.rows.values().filter(|row| matches!(row,
        jig_test_domain::Record::Core(jig_core::Record::People(people::Stored::Answer { .. })))).count(), 2);
    let peers = owner.peers.as_ref().expect("actual worker across restart");
    assert_eq!(peers.workers[0].messages.iter().filter(|(_, _, _, words)| words.as_ref() == b"Once after restart").count(), 1);
    let words = format!("words: {:?}", b"Once after restart");
    let asks: Vec<_> = owner.trace.iter().filter(|event| event.starts_with("input Core(People(Ask")
        && event.contains("ask: Say") && event.contains(&words)).collect();
    assert!(asks.len() >= 2, "the actual Say must reach the engine again after restart");
    let keys: std::collections::BTreeSet<_> = asks.iter().map(|event| {
        let start = event.find("key: [").expect("actual request key");
        let end = event[start..].find("], ask:").expect("whole key followed by ask") + start + 1;
        &event[start..end]
    }).collect();
    assert_eq!(keys.len(), 1, "every actual retry preserves the original request key");

}
