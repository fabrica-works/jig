//! Financial payer queries (accounts.md, 4.1; authority.md, section 7).
use crate::{Domain, Funder};

/// The end of a task's financial chain, independent of its requester.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Payer {
    Party(u64),
    Project(u32),
}

/// A prospective task's financial link. Staged links replace live links, so the same query
/// serves a batch, acceptance, refunding or a compound's ordered preview.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PayerLink {
    pub task: u64,
    pub funder: Funder,
}

impl Domain {
    /// Current payer of a live task. No ledger, readiness state or output is changed.
    #[must_use]
    pub fn payer(&self, task: u64) -> Option<Payer> {
        self.prospective_payer(Funder::Task(task), &[])
    }

    /// Resolve a prospective funder using staged links before live rows. The caller supplies
    /// at most the admitted task bound, one link per task. Unknown task names, duplicate links,
    /// cycles and chains longer than that bound return `None` without mutation. An external
    /// funder names its payer even before its period/pool is opened; financial admission remains
    /// the ledger's responsibility. The bounded walk allocates nothing.
    #[must_use]
    pub fn prospective_payer(&self, mut funder: Funder, staged: &[PayerLink]) -> Option<Payer> {
        let bound = self.names.capacity();
        if staged.len() > usize::try_from(bound).expect("task bound fits") {
            return None;
        }
        for (at, link) in staged.iter().enumerate() {
            for previous in staged.iter().take(at) {
                if previous.task == link.task {
                    return None;
                }
            }
        }
        for _ in 0..=bound {
            match funder {
                Funder::Pool { person, .. } => return Some(Payer::Party(person)),
                Funder::Period { project, .. } | Funder::Recurring { project, .. } => {
                    return Some(Payer::Project(project));
                }
                Funder::Task(number) => {
                    let mut next = None;
                    for link in staged {
                        if link.task == number {
                            next = Some(link.funder);
                            break;
                        }
                    }
                    funder = match next {
                        Some(next) => next,
                        None => crate::domain::record(self, number)?.funder,
                    };
                }
            }
        }
        None
    }
}
