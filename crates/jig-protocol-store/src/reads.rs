//! Parties' bounded durable reads and joins (protocol/store.md, 7.3).
mod indexed;
mod joins;
mod paged;
mod point;
use crate::{Event, Phase, Store, loads};
use alloc::boxed::Box;
use jig_domain_core::reads::{Loaded, Range, Scope};
use skein_lib::{Queue, Token};

pub(crate) struct Spent {
    pub rows: u32,
    pub bytes: u64,
    pub joins: u32,
}

pub(crate) enum Failure {
    Cut,
    Corrupt,
}

pub(crate) fn empty() -> Loaded {
    Loaded::Page { rows: Box::new([]), next: None }
}

pub(crate) fn read(store: &mut Store, owner: Token, range: Range, above: &mut Queue<Event>) {
    if store.phase == Phase::Closed {
        loads::corrupt(store, owner, above);
        return;
    }
    let mut spent = Spent { rows: range.rows, bytes: range.bytes, joins: store.limits.joins };
    let result = match range.scope {
        Scope::Task { task } | Scope::PersonTask { task } | Scope::Result { task } => {
            point::task(store, &range, task, &mut spent)
        }
        Scope::Proposal { proposer, proposal } => point::proposal(store, &range, proposer, proposal, &mut spent),
        Scope::Escalation { task } => point::escalation(store, &range, task, &mut spent),
        Scope::Question { task, message } => point::question(store, &range, task, message, &mut spent),
        Scope::Reply { task, activity } => point::reply(store, &range, task, activity, &mut spent),
        Scope::Turn { .. } => Ok(point::turn()),
        Scope::Activity { task } => paged::activity(store, &range, task, &mut spent),
        Scope::History { task, subtree: false } => paged::history(store, &range, task, &mut spent),
        Scope::Spend { .. } => paged::spend(store, &range, &mut spent),
        Scope::Notes { .. } => paged::notes(store, &range, &mut spent),
        Scope::ScopedNotes { .. } => paged::scoped_notes(store, &range, &mut spent),
        Scope::EndedDelegates { .. } => indexed::ended_delegates(store, &range, &mut spent),
        Scope::History { subtree: true, .. } => indexed::subtree_history(store, &range, &mut spent),
        Scope::Inbox { .. } => indexed::inbox(store, &range, &mut spent),
        Scope::Conversations => indexed::conversations(store, &range, &mut spent),
        Scope::Goals { .. } => indexed::goals(store, &range, &mut spent),
    };
    match result {
        Ok(result) => above.push(Event::Read { owner, result }),
        Err(Failure::Cut) => above.push(Event::Read { owner, result: empty() }),
        Err(Failure::Corrupt) => loads::corrupt(store, owner, above),
    }
}
