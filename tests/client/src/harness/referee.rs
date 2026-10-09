//! Boundary observations checked in the loop (client/domain/testing.md, 6).
//! The truth ledger is independent of roots; later card and word observations
//! share the same referee and never inspect a client's state.
use super::Observed;
use skein_world::domain::{Expectations, Judge};
use std::collections::{BTreeMap, BTreeSet};

/// Initial durable truth rules, shared by every application/front pair.
#[derive(Debug, Default)]
pub struct Rules {
    durable: BTreeSet<u64>,
    made: BTreeMap<(u64, [u8; 16]), u64>,
}
impl Expectations for Rules {
    type Seen = Observed;
    type Name = u64;
    type Stimulus = ();
    fn observe(&mut self, seen: Observed, judge: &mut Judge<u64, ()>) {
        match seen {
            Observed::Durable { commit, at } => {
                judge.check(at <= judge.now(), "durable commit cannot be observed before its write");
                judge.check(self.durable.insert(commit), "a durable commit is reported once");
            }
            Observed::Made { party, key, commit } => {
                judge.check(self.durable.contains(&commit), "made answer belongs to a durable commit");
                let prior = self.made.insert((party, key), commit);
                judge.check(prior.is_none(), "one party's keyed request is made once");
            }
            Observed::Kept { task: _, message: _ } => {}
        }
    }
}
