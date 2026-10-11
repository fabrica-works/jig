//! Genuine engine acts feed the root; the person's corrections use the view
//! (domain/engine.md, 15; client/domain/testing.md, 5).
use super::*;

fn action(
    world: &mut World<Test, Joint>,
    assignment: &jig_fake_workers::Assignment,
    completion: u32,
    action: jig_core::NamedAction,
) {
    let now = world.now();
    world.front().session.driver.send(
        now,
        clock(now),
        jig_test_domain::Event::Core(jig_core::Event::NamedAction {
            to: skein_lib::ReplyTo::new(Token::new(9_900 + u64::from(completion))),
            key: jig_core::CallKey { task: assignment.task, attempt: assignment.attempt, completion, position: 0 },
            action,
        }),
    );
}

#[test]
fn a_note_written_corrected_by_a_person_and_recalled() {
    let mut world = configured(8517, vec![Box::new([Script::Wait])], |configuration, owner| {
        let mut rules = configuration.core.authority.rules().clone();
        rules.ceiling.notes = jig_core_authority::Scopes::PROJECT;
        let mut policy = configuration.core.authority.policy(1).expect("project policy").clone();
        policy.ceiling.notes = jig_core_authority::Scopes::PROJECT;
        policy.roles[2].authority.notes = jig_core_authority::Scopes::PROJECT;
        let mut checked = jig_core_authority::Domain::new(rules, owner.core.authority).expect("bounded note policy");
        let mut out = skein_lib::Queue::with_capacity(jig_core_authority::POLICY_MAX_OUT);
        jig_core_authority::step(&mut checked, jig_core_authority::Event::Policy { project: 1, policy }, &mut out);
        assert_eq!(out.pop(), Some(jig_core_authority::PolicyFact::Added { project: 1 }));
        configuration.core.authority = checked;
        configuration.core.settings.chat_authority.notes = jig_core_authority::Scopes::PROJECT;
    });
    let member = world.client_with(3, crate::stories::conversation(), receiving_limits(), client);
    world.person(member, start(b"remember and correct"));
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    let assignment =
        world.front().session.driver.world.peers.as_ref().expect("actual worker").workers[0].seen[0].clone();
    action(
        &mut world,
        &assignment,
        1,
        jig_core::NamedAction::Note {
            entry: jig_core_notes::New {
                name: 0,
                scope: jig_core_notes::Scope::Project { project: 1 },
                description: b"original".as_slice().into(),
                body: b"first body".as_slice().into(),
                references: skein_lib::List::with_capacity(1),
                author: jig_core_notes::Author::Task { task: assignment.task, attempt: assignment.attempt },
            },
            recalled: None,
        },
    );
    let name = match world.front().session.driver.world.answers.last().expect("committed note answer").2 {
        jig_core::CallPart::NoteWritten { name, revision: 1 } => name,
        ref other => panic!("note write must be admitted: {other:?}"),
    };
    let within = Duration::from_secs(1);
    world.person(
        member,
        vec![
            Step::Go {
                address: jig_test_client_domain::Address::Jig(jig_client::address::Address::Project {
                    project: 1,
                    section: jig_client::address::ProjectSection::Notes,
                }),
            },
            Step::See { find: Find::text(b"original"), within },
            Step::See { find: Find::exactly_named(Role::Button, b"Edit note"), within },
            Step::Press { find: Find::exactly_named(Role::Button, b"Edit note") },
            Step::Type {
                find: Find::named(Role::TextArea, b"Note description"),
                words: b"corrected".as_slice().into(),
            },
            Step::Type { find: Find::named(Role::TextArea, b"Note body"), words: b"second body".as_slice().into() },
            Step::Press { find: Find::exactly_named(Role::Button, b"Confirm") },
            Step::See { find: Find::text(b"Note edited"), within },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    let actual = world.front().session.driver.world.peers.as_ref().expect("actual member").parties[2]
        .person
        .expect("member sign-in");
    assert!(world.front().session.driver.world.store.rows.values().any(|row| matches!(row,
        jig_test_domain::Record::Core(jig_core::Record::Notes(jig_core_notes::Record::Entry(entry)))
        if entry.name == name && entry.revision == 2 && entry.body.as_ref() == b"second body"
        && entry.author == jig_core_notes::Author::Party { party: actual })));
    action(
        &mut world,
        &assignment,
        2,
        jig_core::NamedAction::Recall { by: jig_core_notes::Recall::Name { name }, page: 0 },
    );
    assert!(matches!(world.front().session.driver.world.answers.last(),
        Some((_, _, jig_core::CallPart::NoteRecalled { entries, more: false }))
        if entries.len() == 1 && entries[0].revision == 2 && entries[0].body.as_ref() == b"second body"));
}

#[test]
fn a_delegate_held_past_its_tries_is_escalated_to_a_person_and_released() {
    use jig_core_tasks as tasks;
    use jig_test_domain as root;
    let mut world = configured_with_slots(
        8518,
        vec![
            Box::new([
                Script::Silence { for_: Duration::from_secs(3) },
                Script::Finish { report: b"parent done".as_slice().into() },
            ]),
            Box::new([Script::Fail(jig_fake_workers::Failure::Run)]),
            Box::new([Script::Fail(jig_fake_workers::Failure::Run)]),
            Box::new([Script::Finish { report: b"delegate recovered".as_slice().into() }]),
        ],
        2,
        |_, _| {},
    );
    let member = world.client_with(3, crate::stories::conversation(), receiving_limits(), client);
    world.person(member, start(b"release the exhausted delegate"));
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    let assignment =
        world.front().session.driver.world.peers.as_ref().expect("actual worker").workers[0].seen[0].clone();
    let owner = &world.front().session.driver.world;
    let mut authority =
        match owner.store.rows.get(&root::Key::Core(jig_core::Key::Tasks(tasks::Key::Live(assignment.task)))) {
            Some(root::Record::Core(jig_core::Record::Tasks(tasks::Stored::Live(row)))) => row.authority.clone(),
            other => panic!("actual parent authority: {other:?}"),
        };
    authority.budget.spend = 20;
    authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    let now = world.now();
    world.front().session.driver.send(
        now,
        clock(now),
        root::Event::Core(jig_core::Event::DelegateValidated {
            to: skein_lib::ReplyTo::new(Token::new(9801)),
            key: jig_core::CallKey { task: assignment.task, attempt: assignment.attempt, completion: 1, position: 0 },
            batch: Box::new([jig_core::Delegate {
                executor: tasks::Executor::Agent { charter: 1 },
                spec: tasks::Spec {
                    words: b"retry this delegate".as_slice().into(),
                    parameters: Box::new([]),
                    inputs: Box::new([]),
                },
                contract: tasks::Contract::Report { words: 128 },
                authority,
                symbolic_grants: Box::new([]),
                dependencies: Box::new([]),
                wake: tasks::WakePolicy::DEFAULT,
            }]),
            stubs: Box::new([]),
        }),
    );
    let child = match &world.front().session.driver.world.answers.last().expect("delegate answer").2 {
        jig_core::CallPart::Delegated(numbers) => numbers[0],
        other => panic!("delegation must be admitted: {other:?}"),
    };
    world.person(member, vec![Step::Pause { span: Duration::from_millis(2100) }]);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    assert!(outcome.judged.0 > 0);
    let revision = match world
        .front()
        .session
        .driver
        .world
        .store
        .rows
        .get(&root::Key::Core(jig_core::Key::Tasks(tasks::Key::Live(child))))
    {
        Some(root::Record::Core(jig_core::Record::Tasks(tasks::Stored::Live(row)))) => {
            assert!(matches!(row.phase, tasks::Phase::Held { why: tasks::Hold::Failures(tasks::Class::Run), .. }));
            match row.escalation {
                tasks::Escalation::Waiting { revision, holder: tasks::EscalationHolder::Task(parent), .. }
                    if parent == assignment.task =>
                {
                    revision
                }
                ref other => panic!("the parent receives the actual exhausted hold first: {other:?}"),
            }
        }
        other => panic!("held delegate: {other:?}"),
    };
    action(
        &mut world,
        &assignment,
        2,
        jig_core::NamedAction::DecideEscalation { task: child, revision, choice: jig_core::EscalationChoice::Pass },
    );
    let within = Duration::from_secs(1);
    world.person(
        member,
        vec![
            Step::Go { address: crate::stories::inbox() },
            Step::See { find: Find::text(b"retry this delegate"), within },
            Step::See { find: Find::text(b"Held: out of tries"), within },
            Step::See { find: Find::exactly_named(Role::Button, b"Release"), within },
            Step::Press { find: Find::exactly_named(Role::Button, b"Release") },
            Step::Press { find: Find::exactly_named(Role::Button, b"Confirm") },
            Step::See { find: Find::text(b"Released"), within },
            Step::Pause { span: Duration::from_secs(1) },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed, "{outcome:?}");
    assert!(outcome.judged.0 > 0);
    let owner = &world.front().session.driver.world;
    for (task, words) in [(child, b"delegate recovered".as_slice()), (assignment.task, b"parent done".as_slice())] {
        assert!(matches!(owner.store.rows.get(&root::Key::Core(jig_core::Key::Tasks(tasks::Key::Ended(task)))),
            Some(root::Record::Core(jig_core::Record::Tasks(tasks::Stored::Ended(row))))
            if matches!(&row.phase, tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report { words: actual })) if actual.as_ref() == words)));
    }
    assert!(owner.store.rows.values().any(|row| matches!(row,
        root::Record::Core(jig_core::Record::People(people::Stored::Answer { outcome, .. }))
        if matches!(outcome, people::Outcome::EscalationDecided { .. }))));
    let attempts: std::collections::BTreeSet<_> = owner.peers.as_ref().expect("actual worker").workers[0]
        .seen
        .iter()
        .filter(|row| row.task == child)
        .map(|row| row.attempt)
        .collect();
    assert_eq!(attempts.len(), 3, "two failures then the person's released activation");
}
