//! Compound decisions are judged against the first independently observed ask.
use super::{Promise, Snapshot, Task, Violation, require};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug)]
pub(super) struct Compound {
    baseline: BTreeMap<u64, Option<Task>>,
    changed: Option<(u64, BTreeSet<u64>)>,
}
impl Compound {
    pub(super) fn asked(tasks: &[u64], snapshot: &Snapshot) -> Self {
        Self { baseline: tasks.iter().map(|&task| (task, snapshot.tasks.get(&task).cloned())).collect(), changed: None }
    }
    pub(super) fn durable(&mut self, number: u64, snapshot: &Snapshot) {
        if self.changed.is_none() {
            let changed: BTreeSet<_> = self
                .baseline
                .iter()
                .filter(|(task, row)| snapshot.tasks.get(task) != row.as_ref())
                .map(|(&task, _)| task)
                .collect();
            if !changed.is_empty() {
                self.changed = Some((number, changed));
            }
        }
    }
    pub(super) fn answered(&self, made: bool) -> Result<(), Violation> {
        require(
            if made {
                self.changed.as_ref().is_some_and(|(_, changed)| changed.len() == self.baseline.len())
            } else {
                self.changed.is_none()
            },
            Promise::Once,
            "compound changed only some tasks in one commit, or changed a task despite refusal",
        )
    }
}

#[cfg(test)]
mod tests;
