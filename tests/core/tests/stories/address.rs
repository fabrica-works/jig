use super::*;
use jig_core_world::{reads::Parties, stories::instructions::task_document};

#[test]
fn a_person_task_is_readdressed_after_durability_and_the_previous_taker_cannot_answer_after_restart() {
    let (configuration, limits) = person_world_fixture(185);
    let mut w = World::configured(185, false, configuration, limits);
    w.reads = Some(Parties::new(w.store.applied));
    let parent = w.assignments[0];
    let mut child = member(&w, tasks::Executor::Person(tasks::PersonAddress::Role(0)), b"check the report");
    child.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    let task = delegate(&mut w, parent, 1, child);
    let first_sign_in = w.sign_in.expect("person signed in");
    let tasks::Party::Person(first) = live_row(&w, parent.0).requester else { panic!("requester") };
    ask(&mut w, 186, people::Ask::TakePerson { project: 1, task });
    let before = task_document(&mut w, task, 80_001);
    assert_eq!(before.person_revision, Some(0));
    assert_eq!(before.taken_by, Some(first));
    w.send(root::Event::Core(core::Event::SignIn {
        reply_to: ReplyTo::new(Token::new(80_002)),
        identity: people::Identity {
            key: people::IdentityKey { provider: 0, subject: 8_u64.to_be_bytes().into() },
            login: b"second".as_slice().into(),
            name: b"new addressee".as_slice().into(),
        },
    }));
    let second_sign_in = w.sign_in.expect("person signed in");
    let Some(people::Reply::SignedIn { person: second, .. }) = w.people_answers.last().copied() else {
        panic!("sign-in")
    };
    w.sign_in = Some(first_sign_in);
    let mut edit = amendment(b"check the report");
    edit.spec = None;
    edit.addressed = Some(people::AddressChange { revision: 0, address: people::PersonAddress::Person(second) });
    let request = people::Ask::Amend { project: 1, task, revision: 0, release: None, amendment: edit };
    let answers = w.people_answers.len();
    w.store.fault(jig_core_world::Fault::Hold { commits: 1 });
    ask(&mut w, 187, request.clone());
    assert_eq!(w.people_answers.len(), answers, "address outcome waits for its decision");
    w.release_commits();
    assert_eq!(
        w.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Amended { task, released: false }))
    );
    let after = task_document(&mut w, task, 80_003);
    assert_eq!(after.executor, tasks::Executor::Person(tasks::PersonAddress::Person(second)));
    assert_eq!(after.person_revision, Some(1));
    assert_eq!(after.spec_revision, before.spec_revision);
    assert_eq!(after.taken_by, None);
    let answer = people::Ask::AnswerPerson {
        project: 1,
        task,
        revision: 0,
        result: people::PersonResult::Report { words: b"answer".as_slice().into() },
    };
    ask(&mut w, 188, answer.clone());
    assert_eq!(
        w.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Unknown)))
    );
    ask_as(&mut w, second_sign_in, 189, answer);
    assert_eq!(
        w.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Moved {
            now: people::Moved::Person { revision: 1 },
        })))
    );
    w.restart_with(person_world_fixture(185).0);
    assert_eq!(live_row(&w, task).executor, tasks::Executor::Person(tasks::PersonAddress::Person(second)));
    assert_eq!(live_row(&w, task).taken_by, None);
    ask_as(&mut w, first_sign_in, 187, request);
    assert_eq!(
        w.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Amended { task, released: false }))
    );
    assert_eq!(live_row(&w, task).revisions.person, 1, "replayed request changes nothing");
    ask_as(
        &mut w,
        second_sign_in,
        190,
        people::Ask::AnswerPerson {
            project: 1,
            task,
            revision: 1,
            result: people::PersonResult::Report { words: b"answer".as_slice().into() },
        },
    );
    assert_eq!(w.people_answers.last(), Some(&people::Reply::Outcome(people::Outcome::PersonAnswered { task })));
}

#[test]
fn address_and_spec_and_exact_hold_release_are_one_decision_and_stale_address_refuses_every_change() {
    let (configuration, limits) = person_world_fixture(186);
    let mut w = World::configured(186, false, configuration, limits);
    let parent = w.assignments[0];
    let mut child = member(&w, tasks::Executor::Person(tasks::PersonAddress::Role(0)), b"first question");
    child.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    let task = delegate(&mut w, parent, 1, child);
    ask(&mut w, 191, people::Ask::TakePerson { project: 1, task });
    w.send(root::Event::Core(core::Event::Tasks(tasks::Event::Hold { task, why: tasks::Hold::Stalled })));
    let hold = live_row(&w, task).revisions.hold;
    let mut edit = amendment(b"second question");
    edit.addressed = Some(people::AddressChange { revision: 0, address: people::PersonAddress::Role(1) });
    ask(&mut w, 192, people::Ask::Amend { project: 1, task, revision: 0, release: Some(hold), amendment: edit });
    assert_eq!(
        w.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Amended { task, released: true }))
    );
    let row = live_row(&w, task);
    assert_eq!(row.executor, tasks::Executor::Person(tasks::PersonAddress::Role(1)));
    assert_eq!(row.taken_by, None);
    assert_eq!(row.revisions.spec, 1);
    assert_eq!(row.revisions.person, 1);
    assert!(!matches!(row.phase, tasks::Phase::Held { .. }));
    let before = row.clone();
    let mut edit = amendment(b"must not land");
    edit.addressed = Some(people::AddressChange { revision: 0, address: people::PersonAddress::Role(0) });
    ask(&mut w, 193, people::Ask::Amend { project: 1, task, revision: 1, release: None, amendment: edit });
    assert_eq!(
        w.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Moved {
            now: people::Moved::Person { revision: 1 },
        })))
    );
    assert_eq!(live_row(&w, task), &before, "every owner field remains unchanged");
    w.send(root::Event::Core(core::Event::Tasks(tasks::Event::Hold { task, why: tasks::Hold::Budget })));
    let now = live_row(&w, task).revisions.hold;
    let mut edit = amendment(b"must not release");
    edit.addressed = Some(people::AddressChange { revision: 1, address: people::PersonAddress::Role(0) });
    ask(&mut w, 194, people::Ask::Amend { project: 1, task, revision: 1, release: Some(hold), amendment: edit });
    assert_eq!(
        w.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Moved {
            now: people::Moved::Hold { revision: now, reason: people::HoldReason::Budget },
        })))
    );
    assert_eq!(live_row(&w, task).executor, tasks::Executor::Person(tasks::PersonAddress::Role(1)));
    assert_eq!(live_row(&w, task).spec.words.as_ref(), b"second question");
}
