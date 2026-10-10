//! Filtered live project tasks in last-change order (client/domain/pages.md, 6.3).
//! A project's role admits the list. Each shown row keeps its requester and goal context;
//! the window bounds rows and `more` says whether additional matching live rows remain.
use super::{Chip, Cursor, DocumentLimits, Missing, Window};
use crate::{Core, Limits};
use core::cmp::Reverse;
use jig_core_tasks as tasks;
use skein_lib::{List, Map, Wall};

/// The task owner's lifecycle state selected by a project list.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TaskPhase {
    /// Dependencies or holdings are not yet ready.
    Waiting,
    /// Ready without a due run.
    Idle,
    /// Ready and due for a run.
    Due,
    /// Run preparation is in progress.
    Preparing,
    /// A placement is claimed.
    Claimed,
    /// A run remains active in the owner.
    Running,
    /// A retry deadline is pending.
    BackingOff,
    /// Any active state, including idle and backoff.
    Active,
    /// Ending waits for delegates or connectors.
    Closing,
    /// A hold prevents progress.
    Held,
    /// Ended without selecting a terminal outcome; absent from the live list.
    Ended,
    /// Successfully ended; absent from the live list.
    Done,
    /// Failed; absent from the live list.
    Failed,
    /// Cancelled; absent from the live list.
    Cancelled,
}

/// Owner facts selected without reading any connector-specific value.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct TaskFilter {
    pub phase: Option<TaskPhase>,
    pub executor: Option<tasks::Executor>,
    pub goal: Option<u64>,
    pub asked_by_me: bool,
}

/// One live task's context beside its chip.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TaskLine {
    pub chip: Chip,
    pub goal: Option<u64>,
    pub requester: tasks::Party,
    pub asked_by_me: bool,
    pub changed: Wall,
}

/// A bounded matching list, newest change first with descending task identity ties.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Tasks {
    pub project: u32,
    pub filter: TaskFilter,
    pub rows: List<TaskLine>,
    pub more: bool,
}

fn phase_matches(phase: &tasks::Phase, selected: TaskPhase) -> bool {
    match phase {
        tasks::Phase::Waiting => selected == TaskPhase::Waiting,
        tasks::Phase::Active(active) => {
            selected == TaskPhase::Active
                || match active {
                    tasks::Active::Idle => selected == TaskPhase::Idle,
                    tasks::Active::Due => selected == TaskPhase::Due,
                    tasks::Active::Preparing => selected == TaskPhase::Preparing,
                    tasks::Active::Claimed { .. } => selected == TaskPhase::Claimed,
                    tasks::Active::Running { .. } => selected == TaskPhase::Running,
                    tasks::Active::BackingOff { .. } => selected == TaskPhase::BackingOff,
                }
        }
        tasks::Phase::Closing(_) => selected == TaskPhase::Closing,
        tasks::Phase::Held { .. } => selected == TaskPhase::Held,
        tasks::Phase::Ended(_) => false,
    }
}
fn goal(core: &Core, row: &tasks::TaskRecord) -> Option<u64> {
    if row.tracked.is_some() {
        return Some(row.number);
    }
    match core.tasks.task(row.root) {
        Some(root) if root.tracked.is_some() => Some(root.number),
        Some(_) | None => None,
    }
}
fn matches(core: &Core, row: &tasks::TaskRecord, party: u64, filter: &TaskFilter) -> bool {
    if let Some(phase) = filter.phase
        && !phase_matches(&row.phase, phase)
    {
        return false;
    }
    if let Some(executor) = filter.executor
        && executor != row.executor
    {
        return false;
    }
    if let Some(selected) = filter.goal
        && goal(core, row) != Some(selected)
    {
        return false;
    }
    !filter.asked_by_me || row.requester == tasks::Party::Person(party)
}

pub(super) fn make(
    core: &Core,
    limits: &Limits,
    party: u64,
    project: u32,
    filter: &TaskFilter,
    window: &Window,
) -> Result<Tasks, Missing> {
    if core.people.role(party, project).is_none() {
        return Err(Missing::NotAllowed);
    }
    let mut order = Map::with_capacity(limits.tasks.tasks);
    for candidate in core.tasks.view_tasks().as_ref() {
        let row = core.tasks.task(candidate.number).expect("live task view");
        if row.project == project && matches(core, row, party, filter) {
            order.insert(Reverse((row.changed, row.number)), row.number).expect("bounded live order");
        }
    }
    let mut rows = List::with_capacity(window.rows);
    let mut taking = window.after.is_none();
    let mut more = false;
    for (_, number) in &order {
        if !taking {
            if window.after == Some(Cursor::Number { number: *number }) {
                taking = true;
            }
            continue;
        }
        if rows.room() == 0 {
            more = true;
            break;
        }
        let row = core.tasks.task(*number).expect("ordered live task");
        rows.push(TaskLine {
            chip: super::small::chip_with_core(core, limits, row)?,
            goal: goal(core, row),
            requester: row.requester,
            asked_by_me: row.requester == tasks::Party::Person(party),
            changed: row.changed,
        })
        .expect("bounded task list");
    }
    Ok(Tasks { project, filter: *filter, rows, more })
}

pub(super) fn worst_case(limits: &DocumentLimits) -> Option<u64> {
    List::<TaskLine>::worst_case(limits.rows)?
        .checked_add(u64::from(limits.rows).checked_mul(super::chip_heap_worst_case(limits)?)?)
}
pub(super) fn scratch_worst_case(limits: &Limits) -> Option<u64> {
    Map::<Reverse<(Wall, u64)>, u64>::worst_case(limits.tasks.tasks)
}
