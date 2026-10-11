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
