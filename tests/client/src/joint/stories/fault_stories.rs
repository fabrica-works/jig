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
