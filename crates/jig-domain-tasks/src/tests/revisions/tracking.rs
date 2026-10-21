use super::{Harness, amendment, task, to};
use crate::{Event, Key, Party, Problem, Refusal, Request, Stored, Tracking};

const TASK: u64 = 1;
const SECOND: u64 = 2;
const THIRD: u64 = 3;
const REQUESTER: u64 = 9;
const MESSAGE: u64 = 1;

#[test]
fn an_amendment_tracks_a_task_at_its_priority() {
    let mut test = Harness::new();
    test.make(vec![task(TASK, &[])]);
    let mut change = amendment();
    change.tracked = Some(Tracking::Tracked { priority: 7 });
    test.amend(TASK, change);
    let row = test.domain.task(TASK).unwrap();
    assert_eq!(row.tracked, Some(7));
    assert_eq!(row.revisions.spec, 0);
    assert_eq!(row.revision, 1);
}

#[test]
fn an_amendment_untracks_a_goal_and_moves_no_spec_revision() {
    let mut test = Harness::new();
    let mut goal = task(TASK, &[]);
    goal.tracked = Some(7);
    test.make(vec![goal]);
    let mut change = amendment();
    change.tracked = Some(Tracking::Untracked);
    test.amend(TASK, change);
    let row = test.domain.task(TASK).unwrap();
    assert_eq!(row.tracked, None);
    assert_eq!(row.revisions.spec, 0);
    assert_eq!(row.revision, 1);
}

#[test]
fn a_priority_change_alone_moves_no_spec_revision() {
    let mut test = Harness::new();
    let mut goal = task(TASK, &[]);
    goal.tracked = Some(7);
    test.make(vec![goal]);
    let mut change = amendment();
    change.tracked = Some(Tracking::Tracked { priority: 8 });
    test.amend(TASK, change);
    let row = test.domain.task(TASK).unwrap();
    assert_eq!(row.tracked, Some(8));
    assert_eq!(row.revisions.spec, 0);
    assert_eq!(row.revision, 1);
}

#[test]
fn tracking_past_goals_per_project_is_refused_and_changes_nothing() {
    let mut test = Harness::new();
    let mut first = task(TASK, &[]);
    first.tracked = Some(1);
    let mut second = task(SECOND, &[]);
    second.tracked = Some(2);
    test.make(vec![first, second, task(THIRD, &[])]);
    let before = test.domain.task(THIRD).unwrap().clone();
    let stored = test.stored.clone();
    let mut change = amendment();
    change.tracked = Some(Tracking::Tracked { priority: 3 });
    let out = test.send(Event::Amend {
        places: Box::new([]),
        funder: None,
        reply_to: to(),
        by: Party::Person(REQUESTER),
        task: THIRD,
        message: MESSAGE,
        stop_run: false,
        amendment: change,
    });
    assert_eq!(out, vec![Request::Refused { reply_to: to(), problem: Problem::new(Some(THIRD), Refusal::Goals) }]);
    assert_eq!(test.domain.task(THIRD), Some(&before));
    assert_eq!(test.stored, stored);
}

#[test]
fn a_batch_past_goals_per_project_is_refused_naming_the_limit() {
    let mut test = Harness::new();
    let mut batch = vec![task(TASK, &[]), task(SECOND, &[]), task(THIRD, &[])];
    for row in &mut batch {
        row.tracked = Some(1);
    }
    let out =
        test.send(Event::Make { reply_to: to(), creator: Party::Person(REQUESTER), batch: batch.into_boxed_slice() });
    assert_eq!(out, vec![Request::Refused { reply_to: to(), problem: Problem::new(Some(TASK), Refusal::Goals) }]);
    assert_eq!(test.domain.task(TASK), None);
}

#[test]
fn the_history_keeps_tracked_before_and_after() {
    let mut test = Harness::new();
    test.make(vec![task(TASK, &[])]);
    let mut change = amendment();
    change.tracked = Some(Tracking::Tracked { priority: 7 });
    test.amend(TASK, change);
    let Stored::History(history) = &test.stored[&Key::History { task: TASK, revision: 1 }] else { panic!() };
    let settings = history.amendment.as_ref().unwrap();
    assert_eq!(settings.before.tracked, None);
    assert_eq!(settings.after.tracked, Some(7));
}
