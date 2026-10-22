//! Owner joins count decoded records (protocol/store.md, 7.3).
use jig_domain_core::{self as core, reads};
use jig_domain_tasks as tasks;
use jig_protocol_store::{Event, Request, Write};
use jig_store_world::{World, root};
use skein_lib::{Token, Wall};

const GOAL: u64 = 1;
const CHILD: u64 = 2;
const GRANDCHILD: u64 = 3;
const ALICE: u64 = 1;
// The Small profile's store join bound (jig-protocol-config/src/profile/small.rs).
const SMALL_JOINS: u32 = 4;
// A point read has one row; this fixture's bounded task records fit 1 MiB.
const POINT_ROWS: u32 = 1;
const POINT_BYTES: u64 = 1_048_576;

fn chain(ended: &[u64]) {
    let mut world = World::with_joins(18, SMALL_JOINS);
    let mut records = Vec::new();
    for (number, requester, depth) in [
        (GOAL, tasks::Party::Person(ALICE), 0),
        (CHILD, tasks::Party::Task(GOAL), 1),
        (GRANDCHILD, tasks::Party::Task(CHILD), 2),
    ] {
        let core::Record::Tasks(tasks::Stored::Live(mut task)) = root::live_task(number) else {
            panic!("live task fixture")
        };
        task.requester = requester;
        task.root = GOAL;
        task.depth = depth;
        let stored = if ended.contains(&number) {
            task.phase = tasks::Phase::Ended(tasks::Ending::Failed { reason: Box::new([]) });
            task.ended_at = Some(Wall::EPOCH);
            tasks::Stored::Ended(task)
        } else {
            tasks::Stored::Live(task)
        };
        records.push(core::Record::Tasks(stored));
    }
    let mut writes = vec![Write::Save(root::header(1))];
    writes.extend(records.iter().cloned().map(Write::Save));
    world.commit(writes.into_boxed_slice()).expect("chain commits");
    let events = world.request(Request::Read {
        owner: Token::new(1),
        range: reads::Range {
            party: ALICE,
            scope: reads::Scope::Task { task: GRANDCHILD },
            after: None,
            backwards: false,
            rows: POINT_ROWS,
            bytes: POINT_BYTES,
        },
    });
    let [Event::Read { result: reads::Loaded::Page { rows, next: None }, .. }] = events.as_slice() else {
        panic!("one uncut point-read terminal: {events:?}")
    };
    let [row] = rows.as_ref() else { panic!("one task row: {rows:?}") };
    let [goal, child, grandchild] = records.as_slice() else { panic!("three task fixtures") };
    assert_eq!(&row.value, grandchild);
    assert_eq!(row.ancestors.as_ref(), [CHILD, GOAL]);
    let [parent, root] = row.linked.as_ref() else { panic!("both ancestors: {:?}", row.linked) };
    for (task, expected) in [(parent, child), (root, goal), (row.owner.as_ref().expect("task owner"), grandchild)] {
        let core::Record::Tasks(tasks::Stored::Live(expected) | tasks::Stored::Ended(expected)) = expected else {
            panic!("task fixture")
        };
        assert_eq!(task, expected);
    }
}

#[test]
fn an_ended_ancestry_reads_within_smalls_joins() {
    for ended in [&[GOAL, CHILD][..], &[GOAL, CHILD, GRANDCHILD][..]] {
        chain(ended);
    }
}

#[test]
fn a_live_ancestry_still_reads_within_smalls_joins() {
    chain(&[]);
}
