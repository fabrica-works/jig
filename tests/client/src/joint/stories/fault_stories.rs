//! Client faults cross the real protocol loop (testing.md, sections 5–7).
use super::*;

fn start() -> Vec<Step<jig_test_client_domain::Address>> {
    let within = Duration::from_secs(1);
    vec![
        Step::See { find: Find::exactly_named(Role::Button, b"Sign in with Test"), within },
        Step::Press { find: Find::exactly_named(Role::Button, b"Sign in with Test") },
        Step::See { find: Find::named(Role::TextArea, b"First words"), within },
        Step::Type { find: Find::named(Role::TextArea, b"First words"), words: Box::from(*b"Hello") },
        Step::Press { find: Find::exactly_named(Role::Button, b"Start a conversation") },
        Step::See { find: Find::text(b"Hello"), within },
        Step::Pause { span: Duration::from_millis(450) },
    ]
}

#[test]
fn a_draft_kept_across_a_reload_stays_unsent_on_the_actual_engine() {
    let mut world = super::world(super::turn());
    let client = world.client(1, crate::stories::conversation());
    let within = Duration::from_secs(1);
    let mut script = start();
    script.extend([
        Step::Press { find: Find::exactly_named(Role::Button, b"Write") },
        Step::Type { find: Find::named(Role::TextArea, b"Your answer"), words: Box::from(*b"Unsent draft") },
        Step::Pause { span: Duration::from_millis(30) },
        Step::Reload,
        Step::See { find: Find::exactly_named(Role::Button, b"Write"), within },
        Step::Press { find: Find::exactly_named(Role::Button, b"Write") },
    ]);
    world.person(client, script);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed, "{outcome:?}");
    assert!(outcome.judged.0 > 0);
    assert!(world.kept(client).iter().any(|record| matches!(record,
        jig_test_client_domain::store::Record::Jig(jig_client::store::Record::Draft { words, .. })
        if words.bytes() == b"Unsent draft")));
    assert!(
        world
            .tree(client)
            .nodes()
            .iter()
            .any(|node| node.value.as_ref().is_some_and(|value| value.text.as_ref() == b"Unsent draft"))
    );
    let owner = &world.front().session.driver.world;
    assert_eq!(
        owner
            .store
            .rows
            .values()
            .filter(|record| matches!(
                record,
                jig_test_domain::Record::Core(jig_core::Record::People(people::Stored::Answer { .. }))
            ))
            .count(),
        1
    );
    assert!(owner.peers.as_ref().expect("actual worker").workers[0].messages.is_empty());
}

fn stop_ready() -> (World<Test, Joint>, crate::ClientId) {
    let mut world = super::world(super::turn());
    let client = world.client(1, crate::stories::conversation());
    let within = Duration::from_secs(1);
    let mut script = start();
    script.extend([
        Step::Press { find: Find::exactly_named(Role::Button, b"Stop") },
        Step::See { find: Find::text(b"Stop the live run and hold its task"), within },
    ]);
    world.person(client, script);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    (world, client)
}
fn answers(world: &mut World<Test, Joint>) -> usize {
    world
        .front()
        .session
        .driver
        .world
        .store
        .rows
        .values()
        .filter(|record| {
            matches!(record, jig_test_domain::Record::Core(jig_core::Record::People(people::Stored::Answer { .. })))
        })
        .count()
}
fn shown(world: &World<Test, Joint>, client: crate::ClientId, words: &[u8]) -> bool {
    use jig_fake_person::{Face, Found, TreeFace};
    matches!(TreeFace::new(world.tree(client)).find(&Find::text(words)), Found::Node(_))
}

#[test]
fn a_held_actual_commit_keeps_the_stop_answer_pending_until_its_completion() {
    let (mut world, client) = stop_ready();
    let submitted = world.front().session.driver.world.store.applied.checked_add(1).expect("next transaction");
    world.front().session.driver.world.cut = Some(jig_core_world::effects::Cut::Submitted(submitted));
    world.person(
        client,
        vec![
            Step::Press { find: Find::exactly_named(Role::Button, b"Confirm") },
            Step::Pause { span: Duration::from_millis(30) },
        ],
    );
    let cut_boundary = world.now().saturating_add(Duration::from_millis(50));
    let outcome = world.run(Until::Time(cut_boundary));
    assert!(
        matches!(outcome.verdict, skein_world::domain::Verdict::Open { .. }),
        "the held request remains a real liveness obligation"
    );
    assert!(outcome.judged.0 > 0);
    assert_eq!(
        world.front().session.driver.world.reached_cut,
        Some(jig_core_world::effects::Cut::Submitted(submitted))
    );
    assert_eq!(answers(&mut world), 1, "only starting the chat is answered durably");
    assert!(!shown(&world, client, b"Held stopped"));
    assert!(world.front().session.driver.withhold_completion().expect("actual store applied").is_none());
    assert_eq!(answers(&mut world), 2, "the stop is durable while its completion is withheld");
    assert!(!shown(&world, client, b"Held stopped"), "durability alone cannot fabricate a callback");
    world.front().session.driver.release_commits();
    world.person(client, vec![Step::See { find: Find::text(b"Held stopped"), within: Duration::from_secs(1) }]);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed, "{outcome:?}");
    assert!(outcome.judged.0 > 0);
    assert_eq!(answers(&mut world), 2);
    assert!(world.front().session.driver.world.cancellations.iter().any(|(task, _)| *task == 1));
}

#[test]
fn a_failed_commit_stops_the_engine_and_the_person_sees_offline_before_keyed_recovery() {
    fn configuration() -> jig_test_domain::Config {
        jig_core_world::world::config(8510)
    }
    let (mut world, client) = stop_ready();
    world.front().restart_configuration(configuration);
    let (applied, rows) = {
        let owner = &world.front().session.driver.world;
        (owner.store.applied, owner.store.rows.clone())
    };
    world.front().session.driver.world.store.fail_at = Some(applied.checked_add(1).expect("next actual transaction"));
    world.person(
        client,
        vec![
            Step::Press { find: Find::exactly_named(Role::Button, b"Confirm") },
            Step::See { find: Find::text(b"Offline"), within: Duration::from_secs(1) },
        ],
    );
    {
        for cut in 0..5 {
            let observed = world.run(Until::Steps(2));
            eprintln!("FAILED_COMMIT_EARLY_{cut} {observed:?}");
            eprintln!("FAILED_COMMIT_TREE_{cut} {:?}", world.tree(client).nodes());
        }
    }
    let boundary = world.now().saturating_add(Duration::from_millis(250));
    let outcome = world.run(Until::Time(boundary));
    assert!(matches!(outcome.verdict, skein_world::domain::Verdict::Open { .. }));
    assert!(outcome.judged.0 > 0);
    assert!(shown(&world, client, b"Offline"));
    assert!(!shown(&world, client, b"Held stopped"));
    let owner = &world.front().session.driver.world;
    assert!(owner.stopped);
    assert_eq!(owner.store.applied, applied);
    assert_eq!(owner.store.rows, rows);
    assert_eq!(answers(&mut world), 1);
    world.fault(crate::FrontFault::Restart { offline: Duration::from_millis(150) });
    world.person(client, vec![Step::See { find: Find::text(b"Held stopped"), within: Duration::from_secs(1) }]);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed, "{outcome:?}");
    assert!(outcome.judged.0 > 0);
    assert!(!world.front().session.driver.world.stopped);
    assert_eq!(answers(&mut world), 2, "the original keyed stop is made once after real recovery");
}
