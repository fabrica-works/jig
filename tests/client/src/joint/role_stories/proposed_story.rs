//! A genuine member's bounded goal is disclosed as a proposal before sending.
use super::*;

#[test]
fn an_outside_allotment_goal_is_shown_as_proposed_before_send_and_named_in_the_answer() {
    let mut world = world(8519, vec![]);
    let member = world.client_with(
        3,
        jig_client_test_domain::Address::Jig(jig_client_view::address::Address::Project {
            project: 1,
            section: jig_client_view::address::ProjectSection::Goals,
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
            Step::See { find: Find::text(b"Your pool has"), within },
        ],
    );
    // Until jig-16.3 sends shapes::set_goal's request, the goal act breaks its proposal offer.
    let stopped = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| world.run(Until::Done)));
    assert!(stopped.is_err());
    assert_eq!(
        world.referee().verdict(),
        skein_world::domain::Verdict::Failed(skein_world::domain::Failure {
            at: Time::from_nanos(8_416_106),
            why: "offers kept: draft answer differs from its offer".to_owned(),
            pending: vec![],
        })
    );
    assert!(world.referee().judged().0 > 0);
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
                outcome, ..
            })) => Some(*outcome),
            jig_test_domain::Record::Core(_) | jig_test_domain::Record::Connector { .. } => None,
        })
        .collect();
    assert_eq!(
        answers,
        vec![people::Outcome::Refused(people::Refusal::Authority {
            lacking: people::Lack::Budget { needs: 150, left: 50 },
            proposable: Some(people::Accepter::Role(0)),
        })]
    );
    assert!(world.front().session.driver.world.peers.as_ref().expect("actual worker").workers[0].seen.is_empty());
}
