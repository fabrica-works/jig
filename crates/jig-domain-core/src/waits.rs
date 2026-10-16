//! What a task waits on, derived from its live subtree (engine.md, 11.3).
use crate::{Core, Limits};
use jig_domain_fleet as fleet;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{List, Token};

/// What a task waits on, computed for each document and never stored.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum WaitsOn {
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
            let Some(candidate) = self.tasks.task(number) else { continue };
            for entry in self.waiting_entries(limits, candidate).as_ref() {
                match entry.kind {
                    people::EntryKind::Question { .. }
                    | people::EntryKind::Proposal { .. }
                    | people::EntryKind::Escalation { .. }
                    | people::EntryKind::PersonTask => {}
                    people::EntryKind::ChatReply { .. } | people::EntryKind::Reply { .. } => continue,
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
                match self.fleet.observation(Token::new(task), Token::new(attempt)) {
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
}
