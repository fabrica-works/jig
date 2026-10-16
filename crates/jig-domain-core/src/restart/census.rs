//! Index ranges for the inbox's restart census (protocol/store.md 7.2).
use crate::Key;
use alloc::boxed::Box;
use jig_domain_tasks as tasks;
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ReadResults {
    pub party: u64,
    pub after: u64,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Census {
    pub results: Box<[ReadResults]>,
}
impl Census {
    #[must_use]
    pub fn admits(&self, key: &Key) -> bool {
        match key {
            Key::Tasks(tasks::Key::ConversationOldest { .. }) => true,
            Key::Tasks(tasks::Key::Result { party, position }) => {
                for read in &self.results {
                    if read.party == *party && *position > read.after {
                        return true;
                    }
                }
                false
            }
            Key::Tasks(
                tasks::Key::Live(_)
                | tasks::Key::Ended(_)
                | tasks::Key::Stub(_)
                | tasks::Key::Ledger(_)
                | tasks::Key::ClosedLedger(_)
                | tasks::Key::DecidedProposal(_)
                | tasks::Key::Writer(_)
                | tasks::Key::Pool(_)
                | tasks::Key::PersonProposal(_)
                | tasks::Key::History { .. }
                | tasks::Key::Milestone { .. }
                | tasks::Key::Activity { .. }
                | tasks::Key::Update { .. }
                | tasks::Key::ConversationNewest { .. }
                | tasks::Key::Goal { .. }
                | tasks::Key::EndedByParty { .. }
                | tasks::Key::EndedDelegate { .. }
                | tasks::Key::Delegate { .. },
            )
            | Key::Core(_)
            | Key::People(_)
            | Key::Notes(_) => false,
        }
    }
}
