//! Explicit proposal asks checked independently of the engine (domain/testing.md, 6).
use super::{Promise, Snapshot, Source, Violation};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Default)]
pub(super) struct Proposals {
    credits: BTreeMap<Source, u32>,
    seen: BTreeSet<u64>,
}

impl Proposals {
    pub(super) fn asked(&mut self, proposer: Source) {
        *self.credits.entry(proposer).or_default() += 1;
    }

    pub(super) fn refused(&mut self, proposer: Source) {
        let credit = self.credits.entry(proposer).or_default();
        *credit = credit.saturating_sub(1);
    }

    pub(super) fn durable(&mut self, snapshot: &Snapshot) -> Result<(), Violation> {
        for task in snapshot.tasks.values() {
            if let Some((number, _)) = task.proposal {
                self.proposal(number, Source::Task(task.number), task.executor.0 == 1)?;
            }
        }
        for (&number, &person) in &snapshot.party_proposals {
            self.proposal(number, Source::Person(person), false)?;
        }
        Ok(())
    }

    fn proposal(&mut self, number: u64, proposer: Source, procedure: bool) -> Result<(), Violation> {
        if !self.seen.insert(number) || procedure {
            return Ok(());
        }
        let credit = self.credits.entry(proposer).or_default();
        if *credit == 0 {
            return Err(Violation {
                promise: Promise::Authority,
                why: format!("proposal {number} of {proposer:?} waits unasked"),
            });
        }
        *credit -= 1;
        Ok(())
    }
}
