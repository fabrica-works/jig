//! What a task waits on, derived from its live subtree (engine.md, 11.3).
use crate::{Core, Limits};
use jig_domain_fleet as fleet;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{List, Token};

/// What a task waits on, computed for each document and never stored.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum WaitsOn {
    /// Pending prerequisites, oldest first, bounded by document lines (engine.md, 11.3).
    Dependencies { pending: List<u64>, count: u32 },
    /// The oldest waiting entry and the number of waiting entries in the subtree.
    Party { whom: people::Whom, entries: u32, task: u64, kind: people::EntryKind },
    /// Live delegates, with no party waiting in the subtree.
    Delegates,
    /// A message, a run's resumption or a procedure's facts.
    News,
    /// A working run, a phase that explains its own wait, or an ended task.
    Nothing,
}

impl Core {
    /// Walk the task and its live delegates, within the tasks-per-tree bound.
    #[must_use]
    pub fn waits_on(&self, limits: &Limits, task: u64) -> WaitsOn {
        let Some(row) = self.tasks.task(task) else { return WaitsOn::Nothing };
        from_row(self, limits, row)
    }
}

pub(crate) fn from_row(core: &Core, limits: &Limits, row: &tasks::TaskRecord) -> WaitsOn {
    let task = row.number;
    let mut pending = List::with_capacity(limits.tasks.tree_tasks);
    pending.push(task).expect("validated tree bound");
    let mut oldest: Option<people::Entry> = None;
    let mut entries = 0_u32;
    for _ in 0..limits.tasks.tree_tasks {
        if pending.is_empty() {
            break;
        }
        let index = pending.len().checked_sub(1).expect("nonempty pending subtree");
        let number = *pending.get(index).expect("pending subtree task");
        pending.truncate(index);
        let candidate = if number == row.number {
            row
        } else {
            let Some(candidate) = core.tasks.task(number) else { continue };
            candidate
        };
        for entry in core.waiting_entries(limits, candidate).as_ref() {
            match entry.kind {
                people::EntryKind::Question { .. }
                | people::EntryKind::Proposal { .. }
                | people::EntryKind::Escalation { .. }
                | people::EntryKind::PersonTask => {}
                people::EntryKind::ChatReply { .. } | people::EntryKind::Reply { .. } => continue,
            }
            if entry.kind == people::EntryKind::PersonTask && !candidate.waiting_on.is_empty() {
                continue;
            }
            entries = entries.saturating_add(1);
            let replace = match oldest {
                Some(previous) => (entry.at, entry.task) < (previous.at, previous.task),
                None => true,
            };
            if replace {
                oldest = Some(*entry);
            }
        }
        for delegate in candidate.delegates.iter().rev() {
            pending.push(*delegate).expect("live subtree within tree_tasks");
        }
    }
    if let Some(entry) = oldest {
        return WaitsOn::Party { whom: entry.whom, entries, task: entry.task, kind: entry.kind };
    }
    let working = match row.phase {
        tasks::Phase::Active(tasks::Active::Claimed { attempt } | tasks::Active::Running { attempt }) => {
            match core.fleet.observation(Token::new(task), Token::new(attempt)) {
                Some(value) => match value.activity {
                    fleet::Activity::Waiting => false,
                    fleet::Activity::Preparing
                    | fleet::Activity::Starting
                    | fleet::Activity::Active
                    | fleet::Activity::Writing
                    | fleet::Activity::Long { .. }
                    | fleet::Activity::Ending
                    | fleet::Activity::Tool { .. } => true,
                },
                None => true,
            }
        }
        tasks::Phase::Waiting
        | tasks::Phase::Active(_)
        | tasks::Phase::Held { .. }
        | tasks::Phase::Closing(_)
        | tasks::Phase::Ended(_) => false,
    };
    if working {
        WaitsOn::Nothing
    } else if match row.phase {
        tasks::Phase::Active(_) => true,
        tasks::Phase::Waiting | tasks::Phase::Held { .. } | tasks::Phase::Closing(_) | tasks::Phase::Ended(_) => false,
    } && !row.waiting_on.is_empty()
    {
        dependencies(core, limits, row)
    } else if !row.delegates.is_empty() {
        WaitsOn::Delegates
    } else {
        match row.phase {
            tasks::Phase::Active(
                tasks::Active::Idle | tasks::Active::Claimed { .. } | tasks::Active::Running { .. },
            ) => WaitsOn::News,
            tasks::Phase::Waiting
            | tasks::Phase::Active(_)
            | tasks::Phase::Held { .. }
            | tasks::Phase::Closing(_)
            | tasks::Phase::Ended(_) => WaitsOn::Nothing,
        }
    }
}

fn dependencies(core: &Core, limits: &Limits, row: &tasks::TaskRecord) -> WaitsOn {
    let mut pending = List::with_capacity(limits.documents.lines);
    for _ in 0..limits.documents.lines {
        let mut oldest = None;
        for number in &row.waiting_on {
            if pending.as_slice().contains(number) {
                continue;
            }
            let at = match core.tasks.task(*number) {
                Some(task) => task.created_at,
                None => skein_lib::Wall::from_nanos(0),
            };
            if let Some(previous) = oldest
                && previous <= (at, *number)
            {
                continue;
            }
            oldest = Some((at, *number));
        }
        let Some((_, number)) = oldest else { break };
        pending.push(number).expect("bounded pending dependencies");
    }
    WaitsOn::Dependencies { pending, count: u32::try_from(row.waiting_on.len()).expect("bounded dependency count") }
}
