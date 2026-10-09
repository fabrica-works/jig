//! Authenticated unkeyed reads (`client/domain/contract.md`, section 6).
//! Live documents are copied at admission; past rows load in bounded owner
//! ranges. The core keeps only each load's reply, sign-in and query. A terminal
//! consumes it, checks current visibility and moves the answer to the journal.
//! Neither reading nor loading advances a party's read position.
//!
//! | State | Event | Next | Emits |
//! | --- | --- | --- | --- |
//! | absent | visible live read | absent | held document or live page |
//! | absent | stored read, room | loading | held owner-range load |
//! | absent | full load slab | absent | held busy |
//! | loading | bounded visible page | retired | held answer |
//! | loading | signed out or invisible | retired | held unknown |
//! | loading | missing transport range | retired | held gone |
//! | retired | late duplicate terminal | retired | nothing |

mod vocabulary;
pub use vocabulary::{
    Answer, Cursor, Filter, Group, Historical, Loaded, Page, PageRow, Past, Position, Range, Read, Row, Scope, Why,
};
mod rows;
pub use rows::row_key;

use crate::documents::{self, DocumentClock, Missing, Subject, Window};
use crate::{Core, Held, Limits, Request, Requests};
use alloc::boxed::Box;
use core::mem::size_of;
use skein_lib::{Env, Id, List, Queue, ReplyTo, Slab, Token};

#[derive(Debug)]
pub(crate) struct Pending {
    reply: Option<ReplyTo>,
    sign_in: u64,
    party: u64,
    range: Range,
}

fn terminal(reply: ReplyTo, answer: Answer) -> Requests {
    let mut out = Queue::with_capacity(2);
    out.push(Request::Held(Box::new(Held::Read { reply, answer })));
    out.push(Request::Decided);
    Requests::Out(out)
}

pub(crate) fn scope(read: &Read) -> Option<Scope> {
    match read {
        Read::Task { task } => Some(Scope::Task { task: *task }),
        Read::Proposal { proposer, proposal } => Some(Scope::Proposal { proposer: *proposer, proposal: *proposal }),
        Read::Escalation { task } => Some(Scope::Escalation { task: *task }),
        Read::Question { task, message } => Some(Scope::Question { task: *task, message: *message }),
        Read::PersonTask { task } => Some(Scope::PersonTask { task: *task }),
        Read::Result { task } => Some(Scope::Result { task: *task }),
        Read::Reply { task, activity } => Some(Scope::Reply { task: *task, activity: *activity }),
        Read::Turn { task, attempt, turn } => Some(Scope::Turn { task: *task, attempt: *attempt, turn: *turn }),
        Read::EndedDelegates { task, .. } => Some(Scope::EndedDelegates { task: *task }),
        Read::History { task, subtree, .. } => Some(Scope::History { task: *task, subtree: *subtree }),
        Read::Activity { task, .. } => Some(Scope::Activity { task: *task }),
        Read::Inbox { group, filter, .. } => Some(Scope::Inbox { group: *group, filter: *filter }),
        Read::Conversations { .. } => Some(Scope::Conversations),
        Read::Goals { project, .. } => Some(Scope::Goals { project: *project }),
        Read::Notes { project, .. } => Some(Scope::Notes { project: *project }),
        Read::Spend { project, .. } => Some(Scope::Spend { project: *project }),
        Read::Frame | Read::Project { .. } | Read::Deployment => None,
    }
}

fn cursor(read: &Read) -> Option<&Cursor> {
    match read {
        Read::EndedDelegates { from, .. }
        | Read::History { from, .. }
        | Read::Inbox { from, .. }
        | Read::Conversations { from }
        | Read::Goals { from, .. }
        | Read::Notes { from, .. }
        | Read::Spend { from, .. } => from.as_ref(),
        Read::Activity { before, .. } => before.as_ref(),
        Read::Frame
        | Read::Task { .. }
        | Read::Proposal { .. }
        | Read::Escalation { .. }
        | Read::Question { .. }
        | Read::PersonTask { .. }
        | Read::Result { .. }
        | Read::Reply { .. }
        | Read::Turn { .. }
        | Read::Project { .. }
        | Read::Deployment => None,
    }
}

fn subject(read: &Read, party: u64) -> Option<Subject> {
    match read {
        Read::Frame => Some(Subject::Frame { party }),
        Read::Task { task } => Some(Subject::Task { task: *task }),
        Read::Proposal { proposer, proposal } => Some(Subject::Proposal { proposer: *proposer, number: *proposal }),
        Read::Escalation { task } => Some(Subject::Escalation { task: *task }),
        Read::Question { task, message } => Some(Subject::Question { task: *task, message: *message }),
        Read::PersonTask { task } => Some(Subject::PersonTask { task: *task }),
        Read::Result { task } => Some(Subject::Result { task: *task }),
        Read::Project { project } => Some(Subject::Project { project: *project }),
        Read::Deployment => Some(Subject::Deployment),
        Read::Inbox { .. } => Some(Subject::Inbox { party }),
        Read::Conversations { .. } => Some(Subject::Conversations { party }),
        Read::Goals { project, .. } => Some(Subject::Goals { project: *project }),
        Read::EndedDelegates { .. }
        | Read::History { .. }
        | Read::Activity { .. }
        | Read::Notes { .. }
        | Read::Spend { .. }
        | Read::Reply { .. }
        | Read::Turn { .. } => None,
    }
}

fn backwards(scope: &Scope) -> bool {
    match scope {
        Scope::Activity { .. }
        | Scope::Conversations
        | Scope::Escalation { .. }
        | Scope::Proposal { .. }
        | Scope::Inbox { group: Group::Updates, .. } => true,
        Scope::Task { .. }
        | Scope::Question { .. }
        | Scope::PersonTask { .. }
        | Scope::Result { .. }
        | Scope::Reply { .. }
        | Scope::Turn { .. }
        | Scope::EndedDelegates { .. }
        | Scope::History { .. }
        | Scope::Inbox { group: Group::Needs, .. }
        | Scope::Goals { .. }
        | Scope::Notes { .. }
        | Scope::Spend { .. } => false,
    }
}

pub(crate) fn start(core: &mut Core, env: &Env<Limits>, reply: ReplyTo, sign_in: u64, read: Read) -> Requests {
    let Some(party) = core.people.person(sign_in, env.now, env.wall) else {
        return terminal(reply, Answer::None(Why::Unknown));
    };
    let target = scope(&read);
    let mut after_live = None;
    let mut after_store = None;
    let mut stored = false;
    match cursor(&read) {
        Some(Cursor::Live { scope, after }) => {
            if Some(scope) != target.as_ref() {
                return terminal(reply, Answer::None(Why::Unknown));
            }
            if subject(&read, party).is_none() {
                return terminal(reply, Answer::None(Why::Unknown));
            }
            after_live = Some(*after);
        }
        Some(Cursor::Store { scope, after }) => {
            if Some(scope) != target.as_ref() {
                return terminal(reply, Answer::None(Why::Unknown));
            }
            stored = true;
            after_store.clone_from(after);
        }
        None => {}
    }
    if !stored && let Some(subject) = subject(&read, party) {
        let window = Window { rows: env.limits.documents.rows, after: after_live };
        match documents::make_with_clock(
            core,
            &env.limits,
            party,
            &subject,
            &window,
            DocumentClock { now: env.now, wall: env.wall },
        ) {
            Ok(document) => {
                let answer = rows::live_page(core, &env.limits, party, &read, document);
                return terminal(reply, answer);
            }
            Err(Missing::NotAllowed) => return terminal(reply, Answer::None(Why::Unknown)),
            Err(Missing::TooLarge) => return terminal(reply, Answer::None(Why::Busy)),
            Err(Missing::Unknown | Missing::Ended) => {}
        }
    }
    let Some(scope) = target else {
        return terminal(reply, Answer::None(Why::Unknown));
    };
    if let Some(position) = &after_store
        && !rows::position_allowed(&scope, position)
    {
        return terminal(reply, Answer::None(Why::Unknown));
    }
    if !rows::scope_allowed(core, party, &scope) {
        return terminal(reply, Answer::None(Why::Unknown));
    }
    if core.reads.len().checked_add(core.reading_results.len()).expect("bounded read slots") >= env.limits.load_slots {
        return terminal(reply, Answer::None(Why::Busy));
    }
    let range = Range {
        party,
        backwards: backwards(&scope),
        scope,
        after: after_store,
        rows: env.limits.documents.rows,
        bytes: env.limits.load_bytes,
    };
    let emitted = range.clone();
    let pending = Pending { reply: Some(reply), sign_in, party, range };
    let load = match core.reads.insert(pending) {
        Ok(load) => load.token(),
        Err(pending) => return terminal(pending.reply.expect("new read owns its reply"), Answer::None(Why::Busy)),
    };
    let mut out = Queue::with_capacity(2);
    out.push(Request::Held(Box::new(Held::ReadLoad { load, range: Box::new(emitted) })));
    out.push(Request::Decided);
    Requests::Out(out)
}

pub(crate) fn loaded(core: &mut Core, env: &Env<Limits>, load: Token, result: Loaded) -> Requests {
    let name = Id::<Pending>::from_token(load);
    let Some(pending) = core.reads.get(name) else {
        return Requests::Out(Queue::with_capacity(0));
    };
    if pending.reply.is_none() {
        return Requests::Out(Queue::with_capacity(0));
    }
    let visible = core.people.person(pending.sign_in, env.now, env.wall) == Some(pending.party)
        && rows::scope_allowed(core, pending.party, &pending.range.scope);
    let answer = if visible {
        match result {
            Loaded::Busy => Answer::None(Why::Busy),
            Loaded::Gone => Answer::None(Why::Gone),
            Loaded::Page { rows, next } => rows::from_store(core, &env.limits, &pending.range, rows, next),
        }
    } else {
        Answer::None(Why::Unknown)
    };
    let reply = core.reads.get_mut(name).expect("live read").reply.take().expect("terminal consumes its reply");
    core.reads.retire(name);
    terminal(reply, answer)
}

/// Maximum answer payload retained in a root's held-output slot.
#[must_use]
pub fn answer_worst_case(limits: &Limits) -> Option<u64> {
    let document = documents::document_worst_case(&limits.documents)?;
    u64::try_from(size_of::<Answer>()).ok()?.checked_add(
        document
            .checked_mul(u64::from(limits.documents.rows).checked_add(3)?)?
            .checked_add(limits.load_bytes.checked_mul(2)?)?
            .checked_add(List::<PageRow>::worst_case(limits.documents.rows)?)?,
    )
}

/// Pending load slots, one incoming bounded page and its constructed answer.
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    if limits.load_bytes == 0 || limits.load_slots == 0 {
        return None;
    }
    Slab::<Pending>::worst_case(limits.load_slots)?
        .checked_add(u64::from(limits.load_slots).checked_mul(u64::try_from(size_of::<Position>()).ok()?)?)?
        .checked_add(limits.load_bytes)?
        .checked_add(answer_worst_case(limits)?)
}
