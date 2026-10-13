//! Parties' reads through the memory component (protocol/store.md, 7.3).
use super::fixtures::{self, ALICE, CHILD, GOAL, GRANDCHILD};
use crate::{Event, Request, Store};
use alloc::boxed::Box;
use jig_domain_core::{self as core, reads};
use jig_domain_tasks as tasks;
use skein_lib::Token;

fn child_store() -> Store {
    let mut store = fixtures::opened();
    fixtures::committed(
        &mut store,
        Box::new([
            fixtures::header(1),
            fixtures::save(fixtures::ended_of(GOAL, tasks::Party::Person(ALICE))),
            fixtures::save(fixtures::ended_of(CHILD, tasks::Party::Task(GOAL))),
            fixtures::save(fixtures::terminal(CHILD, 1)),
        ]),
    );
    store
}

fn load(store: &mut Store, range: reads::Range) -> reads::Loaded {
    let events = fixtures::run(store, Request::Read { owner: Token::new(7), range });
    let mut events = events.into_iter();
    let Some(Event::Read { owner, result }) = events.next() else { panic!("one read event") };
    assert_eq!(owner, Token::new(7));
    assert!(events.next().is_none());
    result
}

fn one_row(result: &reads::Loaded) -> &reads::Row {
    let reads::Loaded::Page { rows, next: None } = result else { panic!("one page: {result:?}") };
    let [row] = rows.as_ref() else { panic!("one row: {rows:?}") };
    row
}

fn assert_empty(result: &reads::Loaded) {
    let reads::Loaded::Page { rows, next: None } = result else { panic!("one page: {result:?}") };
    assert!(rows.is_empty());
}

fn linked_numbers(row: &reads::Row) -> Box<[u64]> {
    let mut numbers = skein_lib::List::with_capacity(8);
    for task in &row.linked {
        numbers.push(task.number).expect("small fixture tree");
    }
    numbers.into_boxed()
}

#[test]
fn an_ended_task_read_joins_its_terminal_and_ancestry() {
    let mut store = child_store();
    let result = load(&mut store, fixtures::read(ALICE, reads::Scope::Task { task: CHILD }));
    let row = one_row(&result);
    assert_eq!(row.ancestors.as_ref(), [GOAL]);
    assert_eq!(row.related, Some(fixtures::terminal(CHILD, 1)));
}

#[test]
fn a_live_requester_joins_before_an_ended_one() {
    let mut store = child_store();
    let core::Record::Tasks(tasks::Stored::Ended(mut live)) = fixtures::ended_of(GOAL, tasks::Party::Person(ALICE))
    else {
        panic!("task fixture");
    };
    live.attempt = 2;
    fixtures::committed(
        &mut store,
        Box::new([fixtures::header(2), fixtures::save(core::Record::Tasks(tasks::Stored::Live(live)))]),
    );
    let result = load(&mut store, fixtures::read(ALICE, reads::Scope::Task { task: CHILD }));
    let [parent] = &*one_row(&result).linked else { panic!("one parent") };
    assert_eq!(parent.attempt, 2);
}

#[test]
fn ancestry_stops_at_the_domains_depth_limit() {
    let mut store = child_store();
    store.limits.core.tasks.depth = 1;
    fixtures::committed(
        &mut store,
        Box::new([fixtures::header(2), fixtures::save(fixtures::ended_of(GRANDCHILD, tasks::Party::Task(CHILD)))]),
    );
    let result = load(&mut store, fixtures::read(ALICE, reads::Scope::Task { task: GRANDCHILD }));
    assert_eq!(one_row(&result).ancestors.as_ref(), [CHILD]);
}

#[test]
fn links_join_dependencies_and_delegates_once_each() {
    let mut store = child_store();
    fixtures::committed(
        &mut store,
        Box::new([
            fixtures::header(2),
            fixtures::save(fixtures::with_links(
                fixtures::ended_of(CHILD, tasks::Party::Task(GOAL)),
                &[GOAL, GRANDCHILD],
            )),
            fixtures::save(fixtures::ended_of(GRANDCHILD, tasks::Party::Person(ALICE))),
        ]),
    );
    let result = load(&mut store, fixtures::read(ALICE, reads::Scope::Task { task: CHILD }));
    assert_eq!(linked_numbers(one_row(&result)).as_ref(), [GOAL, GRANDCHILD]);
}

#[test]
fn a_missing_task_reads_as_an_empty_page() {
    let mut store = child_store();
    let result = load(&mut store, fixtures::read(ALICE, reads::Scope::Task { task: GRANDCHILD }));
    assert_empty(&result);
}

#[test]
fn a_page_stops_before_the_row_past_its_bytes() {
    let mut store = child_store();
    let mut range = fixtures::read(ALICE, reads::Scope::Task { task: CHILD });
    range.bytes = 1;
    let result = load(&mut store, range);
    assert_empty(&result);
}

#[test]
fn a_page_stops_before_the_row_past_its_joins() {
    let mut store = child_store();
    store.limits.joins = 2;
    let result = load(&mut store, fixtures::read(ALICE, reads::Scope::Task { task: CHILD }));
    assert_empty(&result);
}

#[test]
fn task_scopes_share_their_joined_owner() {
    let mut store = child_store();
    for scope in [
        reads::Scope::Task { task: CHILD },
        reads::Scope::PersonTask { task: CHILD },
        reads::Scope::Result { task: CHILD },
    ] {
        let result = load(&mut store, fixtures::read(ALICE, scope));
        assert_eq!(one_row(&result).value, fixtures::ended_of(CHILD, tasks::Party::Task(GOAL)));
    }
}

#[test]
fn a_point_cursor_excludes_the_owner_already_returned() {
    let mut store = child_store();
    let mut range = fixtures::read(ALICE, reads::Scope::Task { task: CHILD });
    range.after = Some(reads::Position::Owner(core::Key::Tasks(tasks::Key::Ended(CHILD))));
    let result = load(&mut store, range);
    assert_empty(&result);
}

#[test]
fn a_zero_row_bound_keeps_no_point_row() {
    let mut store = child_store();
    let mut range = fixtures::read(ALICE, reads::Scope::Task { task: CHILD });
    range.rows = 0;
    let result = load(&mut store, range);
    assert_empty(&result);
}
