//! Named decisions checked against durable revisions (domain/testing.md, 6).
use super::{Phase, Promise, Snapshot, Violation};
use std::collections::BTreeSet;

/// Which revision a decision names (domain/people.md, 5.1).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Revision {
    /// The specification an amendment changes.
    Spec(u64),
    /// The hold a release or escalation decision changes.
    Hold(u64),
}

#[derive(Clone, Debug, Default)]
pub(super) struct Revisions(BTreeSet<(u64, Revision)>);

impl Revisions {
    pub(super) fn named(&mut self, task: u64, revision: Revision) {
        self.0.insert((task, revision));
    }

    pub(super) fn durable(&mut self, last: &Snapshot, now: &Snapshot) -> Result<(), Violation> {
        for (&number, task) in &now.tasks {
            let Some(old) = last.tasks.get(&number) else { continue };
            for revision in old.spec_revision..task.spec_revision {
                self.consume(number, Revision::Spec(revision))?;
            }
            if old.phase == Phase::Held && matches!(task.phase, Phase::Waiting | Phase::Active) {
                self.consume(number, Revision::Hold(old.hold_revision))?;
            }
        }
        Ok(())
    }

    fn consume(&mut self, task: u64, revision: Revision) -> Result<(), Violation> {
        if self.0.remove(&(task, revision)) {
            return Ok(());
        }
        let (kind, number) = match revision {
            Revision::Spec(r) => ("spec", r),
            Revision::Hold(r) => ("hold", r),
        };
        Err(Violation {
            promise: Promise::Ownership,
            why: format!("task {task}'s {kind} moved past revision {number}, which nothing named"),
        })
    }
}
