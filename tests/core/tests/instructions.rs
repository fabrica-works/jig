use jig_core_people as people;
use jig_core_tasks as tasks;
use jig_core_world::{
    effects::fixture,
    stories::{instructions as story, reads::request},
};

#[test]
fn an_instruction_amendment_reaches_the_live_run_after_its_commit_and_future_runs_after_restart() {
    let mut w = story::world(180);
    let task = w.assigned.as_ref().unwrap().task;
    let attempt = w.assigned.as_ref().unwrap().attempt;
    let original = w.assigned.as_ref().unwrap().run.policy.instructions.clone();
    let before = story::task_document(&mut w, task, 70_001);
    assert_eq!(before.instructions.unwrap().bytes, original);
    let messages = w.inbound.len();
    let answers = w.people_answers.len();
    w.store.fault(jig_fake_store::Fault::Hold { commits: 1 });
    request(
        &mut w,
        180,
        people::Ask::Amend {
            project: 1,
            task,
            revision: 0,
            release: None,
            amendment: story::change(b"Use the new evidence"),
        },
    );
    assert_eq!(w.people_answers.len(), answers);
    assert_eq!(w.inbound.len(), messages, "the live run hears only committed instructions");
    w.release_commits();
    assert_eq!(
        w.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Amended { task, released: false }))
    );
    assert!(w.inbound.iter().any(|word| matches!(word.kind, tasks::MessageKind::Amendment { .. })
        && word.words.as_ref() == b"Use the new evidence"));
    assert_eq!(w.assigned.as_ref().unwrap().attempt, attempt, "instruction change does not restart a live run");
    assert_eq!(
        w.assigned.as_ref().unwrap().run.policy.instructions,
        original,
        "the active charter remains its original snapshot"
    );
    let after = story::task_document(&mut w, task, 70_002);
    assert_eq!(after.instructions.unwrap().bytes.as_ref(), b"Use the new evidence");
    assert_eq!(after.spec_revision, 1);
    assert_eq!(story::row(&w, task).revisions.hold, 0);
    w.end(tasks::End::Parked, 0);
    w.restart_with(fixture(180, false).0);
    story::sign_in(&mut w);
    w.say(181);
    let next = w.assigned.as_ref().unwrap();
    assert!(next.attempt > attempt);
    assert_eq!(next.run.policy.instructions.as_ref(), b"Use the new evidence");
    assert_eq!(story::row(&w, task).revisions.spec, 1);
}

#[test]
fn an_instruction_amendment_releases_only_the_displayed_hold_and_keeps_the_new_instructions() {
    let mut w = story::world(181);
    let task = w.assigned.as_ref().unwrap().task;
    request(&mut w, 182, people::Ask::Stop { project: 1, task });
    w.end(tasks::End::Parked, 0);
    let hold = story::row(&w, task).revisions.hold;
    request(
        &mut w,
        183,
        people::Ask::Amend { project: 1, task, revision: 0, release: Some(hold), amendment: story::change(b"") },
    );
    assert_eq!(
        w.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Amended { task, released: true }))
    );
    assert_eq!(story::row(&w, task).instructions.as_deref(), Some(b"".as_slice()));
    assert_eq!(story::row(&w, task).revisions.spec, 1);
    assert_eq!(w.assigned.as_ref().unwrap().run.policy.instructions.as_ref(), b"");
    request(
        &mut w,
        184,
        people::Ask::Amend {
            project: 1,
            task,
            revision: 0,
            release: Some(hold),
            amendment: story::change(b"obsolete"),
        },
    );
    assert!(matches!(
        w.people_answers.last(),
        Some(people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Moved {
            now: people::Moved::Spec { revision: 1 }
        })))
    ));
    assert_eq!(story::row(&w, task).instructions.as_deref(), Some(b"".as_slice()));
}
