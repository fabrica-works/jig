//! Small requester trees for party reads (protocol/store.md, 7.3).
use super::{core_rows, ended_task};
use alloc::boxed::Box;
use jig_domain_core::{self as core, Record};
use jig_domain_tasks as tasks;

pub const GOAL: u64 = 31;
pub const CHILD: u64 = 32;
pub const GRANDCHILD: u64 = 33;

pub fn read(party: u64, scope: core::reads::Scope) -> core::reads::Range {
    core::reads::Range { party, scope, after: None, backwards: false, rows: 8, bytes: 65_536 }
}

pub fn ended_of(task: u64, requester: tasks::Party) -> Record {
    let Record::Tasks(tasks::Stored::Ended(mut row)) = ended_task(task) else { panic!("ended fixture") };
    row.requester = requester;
    row.attempt = 1;
    row.dependencies = Box::new([]);
    row.spec.inputs = Box::new([]);
    row.references = Box::new([]);
    row.delegates = Box::new([]);
    row.result_followups = Box::new([]);
    Record::Tasks(tasks::Stored::Ended(row))
}

pub fn terminal(task: u64, attempt: u64) -> Record {
    let mut row = core_rows::terminal();
    row.task = task;
    row.attempt = attempt;
    Record::Core(core::CoreRecord::Terminal(row))
}

pub fn with_links(record: Record, dependencies: &[u64]) -> Record {
    let Record::Tasks(tasks::Stored::Ended(mut row)) = record else { panic!("ended fixture") };
    row.dependencies = Box::from(dependencies);
    row.spec.inputs = Box::from(dependencies);
    row.references = Box::from(dependencies);
    row.delegates = Box::from(dependencies);
    row.result_followups = Box::from(dependencies);
    Record::Tasks(tasks::Stored::Ended(row))
}
