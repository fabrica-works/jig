//! A genuine member's bounded goal is disclosed as a proposal before sending.
use super::*;

#[test]
fn an_outside_allotment_goal_is_shown_as_proposed_before_send_and_named_in_the_answer() {
    let mut world = world(8519, vec![]);
    let member = world.client_with(
        3,
        jig_test_client_domain::Address::Jig(jig_client::address::Address::Project {
            project: 1,
            section: jig_client::address::ProjectSection::Goals,
        }),
        receiving_limits(),
        client,
    );
    let within = Duration::from_secs(1);
    world.person(
        member,
        vec![
            Step::See { find: Find::exactly_named(Role::Button, b"Sign in with Test"), within },
            Step::Press { find: Find::exactly_named(Role::Button, b"Sign in with Test") },
            Step::See { find: Find::exactly_named(Role::TextArea, b"Goal specification"), within },
            Step::Type {
                find: Find::exactly_named(Role::TextArea, b"Goal specification"),
                words: b"a proposed goal".as_slice().into(),
            },
            Step::Press { find: Find::exactly_named(Role::Button, b"Charter 1") },
            Step::Type { find: Find::exactly_named(Role::TextField, b"Goal budget"), words: b"150".as_slice().into() },
            Step::See { find: Find::text(b"This will be proposed to its holder"), within },
            Step::Press { find: Find::exactly_named(Role::Button, b"Set goal") },
            Step::See { find: Find::text(b"This will be proposed"), within },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed, "{outcome:?}");
    assert!(outcome.judged.0 > 0);
    assert!(!world.front().session.driver.world.store.rows.values().any(|row| matches!(
        row,
        jig_test_domain::Record::Core(jig_domain_core::Record::People(people::Stored::Answer { .. }))
    )));
    world.person(
        member,
        vec![
            Step::Press { find: Find::exactly_named(Role::Button, b"Confirm") },
            Step::See { find: Find::text(b"Goal proposed"), within },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed, "{outcome:?}");
    assert!(outcome.judged.0 > 0);
    let answers: Vec<_> = world
        .front()
        .session
        .driver
        .world
        .store
        .rows
        .values()
        .filter_map(|row| match row {
            jig_test_domain::Record::Core(jig_domain_core::Record::People(people::Stored::Answer {
                outcome: people::Outcome::GoalProposed { proposal },
                ..
            })) => Some(*proposal),
            jig_test_domain::Record::Core(_) | jig_test_domain::Record::Connector { .. } => None,
        })
        .collect();
    assert_eq!(answers.len(), 1);
    let label = format!("Proposal number: {}", answers[0]).into_bytes();
    world.person(member, vec![Step::See { find: Find::text(&label), within }]);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    assert!(world.front().session.driver.world.peers.as_ref().expect("actual worker").workers[0].seen.is_empty());
}
