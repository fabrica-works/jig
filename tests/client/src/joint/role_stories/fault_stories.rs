//! Faults across authenticated tabs use the actual proposal and retained decision.
use super::*;
use jig_fake_person::{Face, Found, TreeFace};

fn confirm(world: &World<Test, Joint>, tab: crate::ClientId) -> skein_ui::NodeId {
    match TreeFace::new(world.tree(tab)).find(&Find::exactly_named(Role::Button, b"Confirm")) {
        Found::Node(node) => node,
        Found::Absent | Found::Pending => panic!("both real confirmations must be ready"),
    }
}

#[test]
fn two_tabs_decide_one_proposal_and_both_receive_the_original_once_only_decision() {
    let mut world = world(
        8523,
        vec![
            Box::new([
                Script::Call(Call::ProposeGoal {
                    words: b"decide this goal".as_slice().into(),
                    budget: 150,
                    priority: 7,
                }),
                Script::Park,
            ]),
            Box::new([Script::Finish { report: b"one accepted goal".as_slice().into() }]),
        ],
    );
    let member = world.client_with(3, crate::stories::conversation(), receiving_limits(), client);
    world.person(member, start(b"propose once"));
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    let owner = world.client_with(1, crate::stories::inbox(), receiving_limits(), client);
    let within = Duration::from_secs(1);
    world.person(
        owner,
        vec![
            Step::See { find: Find::exactly_named(Role::Button, b"Sign in with Test"), within },
            Step::Press { find: Find::exactly_named(Role::Button, b"Sign in with Test") },
            Step::See { find: Find::text(b"decide this goal"), within },
            Step::See { find: Find::exactly_named(Role::Button, b"Accept"), within },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    let other = world.tab(owner);
    world.person(
        other,
        vec![
            Step::See { find: Find::exactly_named(Role::Button, b"Sign in with Test"), within },
            Step::Press { find: Find::exactly_named(Role::Button, b"Sign in with Test") },
            Step::See { find: Find::text(b"decide this goal"), within },
            Step::See { find: Find::exactly_named(Role::Button, b"Accept"), within },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    // Both independently restored tabs open ready confirmations before either sends.
    for party in [owner, other] {
        world.person(party, vec![Step::Press { find: Find::exactly_named(Role::Button, b"Accept") }]);
    }
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    let first_person = world.front().session.driver.world.peers.as_ref().expect("actual parties").parties[0]
        .person
        .expect("actual owner");
    let by = format!("Decided by person: {first_person}").into_bytes();
    let first_node = confirm(&world, owner);
    let second_node = confirm(&world, other);
    world.fault(crate::Fault::TwoTabs { first: owner, first_node, second: other, second_node });
    for tab in [owner, other] {
        world.person(
            tab,
            vec![Step::See { find: Find::text(b"Accepted"), within }, Step::See { find: Find::text(&by), within }],
        );
    }
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    let engine = &world.front().session.driver.world;
    assert_eq!(
        engine
            .store
            .rows
            .values()
            .filter(|row| matches!(row,
        jig_test_domain::Record::Core(jig_domain_core::Record::People(people::Stored::Answer { outcome, .. }))
        if matches!(outcome, people::Outcome::ProposalDecided { by, choice: people::ProposalChoice::Accepted, .. }
            if *by == first_person)))
            .count(),
        2
    );
    assert_eq!(
        engine
            .store
            .rows
            .values()
            .filter(|row| matches!(row,
        jig_test_domain::Record::Core(jig_domain_core::Record::Tasks(jig_domain_tasks::Stored::Ended(task)))
        if task.tracked == Some(7) && task.numbers.budget == 150))
            .count(),
        1
    );
    let assignments = &engine.peers.as_ref().expect("actual worker").workers[0].seen;
    assert_eq!(assignments.len(), 3, "the proposing chat resumes after its one accepted goal");
    let tasks: std::collections::BTreeSet<_> = assignments.iter().map(|row| row.task).collect();
    assert_eq!(tasks.len(), 2, "only the proposing chat and its accepted goal");
    let parent = assignments[0].task;
    let parent_attempts: std::collections::BTreeSet<_> =
        assignments.iter().filter(|row| row.task == parent).map(|row| row.attempt).collect();
    assert_eq!(parent_attempts.len(), 2, "the original chat parks and then resumes");
    assert_eq!(assignments.iter().filter(|row| row.task != parent).count(), 1, "one goal assignment");
}

#[test]
fn an_inbox_acceptance_updates_the_same_proposal_still_shown_in_the_chat() {
    let mut world = world(
        8525,
        vec![
            Box::new([
                Script::Call(Call::ProposeGoal {
                    words: b"decide this goal".as_slice().into(),
                    budget: 150,
                    priority: 7,
                }),
                Script::Park,
            ]),
            Box::new([Script::Finish { report: b"one accepted goal".as_slice().into() }]),
        ],
    );
    let member = world.client_with(3, crate::stories::conversation(), receiving_limits(), client);
    world.person(member, start(b"propose once"));
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    let within = Duration::from_secs(1);
    let owner = world.client_with(1, crate::stories::inbox(), receiving_limits(), client);
    world.person(
        owner,
        vec![
            Step::See { find: Find::exactly_named(Role::Button, b"Sign in with Test"), within },
            Step::Press { find: Find::exactly_named(Role::Button, b"Sign in with Test") },
            Step::See { find: Find::text(b"decide this goal"), within },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    world.person(
        member,
        vec![
            Step::Press { find: Find::exactly_named(Role::Button, b"Read earlier") },
            Step::See { find: Find::text(b"decide this goal"), within },
            Step::See { find: Find::text(b"Proposal"), within },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    world.person(
        owner,
        vec![
            Step::Press { find: Find::exactly_named(Role::Button, b"Accept") },
            Step::Press { find: Find::exactly_named(Role::Button, b"Confirm") },
            Step::See { find: Find::text(b"Accepted"), within },
        ],
    );
    world.person(
        member,
        vec![
            Step::See { find: Find::text(b"decide this goal"), within },
            Step::See { find: Find::text(b"Accepted"), within },
            Step::See { find: Find::exactly_named(Role::Button, b"Write"), within },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    let engine = &world.front().session.driver.world;
    let person = engine.peers.as_ref().expect("actual parties").parties[0].person.expect("actual owner");
    assert_eq!(engine.store.rows.values().filter(|row| matches!(row,
        jig_test_domain::Record::Core(jig_domain_core::Record::People(people::Stored::Answer { outcome, .. }))
        if matches!(outcome, people::Outcome::ProposalDecided { by, choice: people::ProposalChoice::Accepted, .. } if *by == person))).count(), 1);
    assert_eq!(
        engine
            .store
            .rows
            .values()
            .filter(|row| matches!(row,
        jig_test_domain::Record::Core(jig_domain_core::Record::Tasks(jig_domain_tasks::Stored::Ended(task)))
        if task.tracked == Some(7) && task.numbers.budget == 150))
            .count(),
        1
    );
    let assignments = &engine.peers.as_ref().expect("actual worker").workers[0].seen;
    assert_eq!(assignments.len(), 3);
    let tasks: std::collections::BTreeSet<_> = assignments.iter().map(|row| row.task).collect();
    assert_eq!(tasks.len(), 2);
    assert_eq!(assignments.iter().filter(|row| row.task != assignments[0].task).count(), 1);
}
