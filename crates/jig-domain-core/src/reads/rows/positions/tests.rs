use super::super::{Answer, Cursor, Filter, Group, Key, Position, Range, Record, Row, Scope, Why, tasks};
use crate::reads;
use crate::tests::{core, make_task, stored_row};
use alloc::boxed::Box;
use skein_lib::Wall;

fn fixture() -> (crate::Core, crate::Limits) {
    let (mut core, limits) = core();
    make_task(&mut core, &limits, 1);
    (core, limits)
}

fn range(scope: Scope) -> Range {
    Range { party: 1, scope, after: None, backwards: false, rows: 8, bytes: 65_536 }
}

fn update_position(at: u64, task: u64) -> Position {
    Position::Owner(Key::Tasks(
        tasks::Update { party: 1, at: Wall::from_nanos(at), task, object: tasks::UpdateObject::Result }.key(),
    ))
}

fn result_row(owner: &tasks::TaskRecord) -> Row {
    let mut owner = owner.clone();
    owner.number = 99;
    owner.result_position = 1;
    owner.ended_at = Some(Wall::from_nanos(20));
    owner.phase = tasks::Phase::Ended(tasks::Ending::Failed { reason: Box::new([]) });
    let value = Record::Tasks(tasks::Stored::Update(tasks::Update {
        party: 1,
        at: Wall::from_nanos(20),
        task: 99,
        object: tasks::UpdateObject::Result,
    }));
    let mut row = stored_row(value, Some(owner.clone()));
    row.inbox = Some(crate::documents::Cursor::Inbox {
        task: 99,
        object: crate::documents::InboxObject::Result { position: owner.result_position },
        at: owner.ended_at,
    });
    row
}

fn subtree_rows(owner: &tasks::TaskRecord) -> Box<[Row]> {
    let root = stored_row(
        Record::Tasks(tasks::Stored::Milestone(tasks::Milestone {
            task: 1,
            position: 2,
            phase: tasks::Phase::Waiting,
            at: Wall::EPOCH,
        })),
        Some(owner.clone()),
    );
    let mut child = owner.clone();
    child.number = 99;
    child.requester = tasks::Party::Task(1);
    let mut row = stored_row(
        Record::Tasks(tasks::Stored::History(tasks::History {
            task: 99,
            revision: 1,
            at: Wall::EPOCH,
            by: tasks::Party::Person(1),
            change: tasks::Change::Amended,
            reason: Box::new([]),
            proposal: None,
            amendment: None,
        })),
        Some(child),
    );
    row.ancestors = Box::new([1]);
    Box::new([root, row])
}

#[test]
fn an_inbox_page_that_skipped_rows_goes_on_from_the_last_examined() {
    let (core, limits) = fixture();
    let scope = Scope::Inbox { group: Group::Results, filter: Filter { project: None, goal: None } };
    let row = result_row(core.tasks.task(1).unwrap());
    let next = update_position(10, 98);
    let answer = super::super::from_store(&core, &limits, &range(scope.clone()), Box::new([row]), Some(next.clone()));
    let Answer::Page(page) = answer else { panic!("indexed cursor accepted") };
    assert_eq!(page.rows.len(), 1);
    assert_eq!(page.next, Some(Cursor::Store { scope, after: Some(next) }));
}

#[test]
fn a_page_that_returned_none_within_its_bound_is_cut_not_ended() {
    let (core, limits) = fixture();
    let scope = Scope::Inbox { group: Group::Results, filter: Filter { project: None, goal: None } };
    let next = update_position(10, 98);
    let answer = super::super::from_store(&core, &limits, &range(scope.clone()), Box::new([]), Some(next.clone()));
    let Answer::Page(page) = answer else { panic!("empty filtered page is retained") };
    assert!(page.rows.is_empty());
    assert_eq!(page.next, Some(Cursor::Store { scope, after: Some(next) }));
}

#[test]
fn a_subtree_history_page_moves_from_one_task_to_the_next() {
    let (core, limits) = fixture();
    let scope = Scope::History { task: 1, subtree: true };
    let rows = subtree_rows(core.tasks.task(1).unwrap());
    let next = reads::row_key(rows.last().unwrap()).unwrap();
    let answer = super::super::from_store(&core, &limits, &range(scope.clone()), rows, Some(next.clone()));
    let Answer::Page(page) = answer else { panic!("task-by-task history accepted") };
    assert_eq!(page.rows.len(), 2);
    assert_eq!(page.next, Some(Cursor::Store { scope, after: Some(next) }));
}

#[test]
fn subtree_history_does_not_return_to_a_task_already_examined() {
    let (core, limits) = fixture();
    let scope = Scope::History { task: 1, subtree: true };
    let rows = subtree_rows(core.tasks.task(1).unwrap());
    let owner = core.tasks.task(1).unwrap();
    let mut repeated = skein_lib::List::with_capacity(3);
    for row in rows {
        repeated.push(row).unwrap();
    }
    let mut extra = subtree_rows(owner).into_iter();
    repeated.push(extra.next().unwrap()).unwrap();
    assert_eq!(
        super::super::from_store(&core, &limits, &range(scope), repeated.into_boxed(), None),
        Answer::None(Why::Unknown)
    );
}
