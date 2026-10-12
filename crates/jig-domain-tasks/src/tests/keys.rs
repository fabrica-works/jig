//! Key order against protocol/store.md, 4.3: live families come first.
use crate::{Funder, Key};

#[test]
fn every_live_tasks_key_sorts_before_every_history_key() {
    let live = [
        Key::Live(u64::MAX),
        Key::Stub(u64::MAX),
        Key::Ledger(Funder::Period { project: u32::MAX, period: u64::MAX }),
        Key::Writer(u64::MAX),
        Key::Pool(u64::MAX),
        Key::PersonProposal(u64::MAX),
    ];
    let history = [
        Key::Ended(0),
        Key::ClosedLedger(Funder::Task(0)),
        Key::DecidedProposal(0),
        Key::History { task: 0, revision: 0 },
        Key::Milestone { task: 0, position: 0 },
        Key::Activity { task: 0, position: 0 },
    ];
    for early in live {
        assert!(early.live(), "{early:?} is a live family");
        for late in history {
            assert!(!late.live(), "{late:?} is history");
            assert!(early < late, "{early:?} sorts before {late:?}");
        }
    }
}
