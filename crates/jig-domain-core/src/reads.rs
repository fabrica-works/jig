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
    Answer, Cursor, Filter, Group, Historical, Loaded, Page, PageRow, PageTurn, Past, Position, Range, Read, Row,
    Scope, UnjoinedTurn, Why,
};
pub(crate) mod activity;
mod answers;
pub mod indexed;
mod rows;
pub use rows::row_key;
mod cursors;
pub use cursors::conversations_cursor;

use crate::documents::{self, DocumentClock, Missing, Subject, Window};
use crate::{Core, Held, Limits, Request, Requests};
use alloc::boxed::Box;
use core::mem::size_of;
use skein_lib::{Env, Id, List, Queue, ReplyTo, Slab, Token};

/// Which physical rows a read scope loads (protocol/store.md, 4.3 and 7.3).
/// Cursor admission is separate: notes lines are joined to entry rows before paging.
#[must_use]
pub fn admits(party: u64, scope: &Scope, key: &crate::Key) -> bool {
    use crate::{Key, notes};
    use jig_domain_people::Proposer;
    use jig_domain_tasks::{Funder, Key as TaskKey};
    match scope {
        Scope::Task { task } | Scope::PersonTask { task } | Scope::Result { task } => {
            key == &Key::Tasks(TaskKey::Live(*task)) || key == &Key::Tasks(TaskKey::Ended(*task))
        }
        Scope::Reply { task, activity } => key == &Key::Tasks(TaskKey::Activity { task: *task, position: *activity }),
        Scope::History { task, subtree: false } => match key {
            Key::Tasks(TaskKey::History { task: owner, .. } | TaskKey::Milestone { task: owner, .. }) => owner == task,
            Key::Tasks(_) | Key::Core(_) | Key::People(_) | Key::Notes(_) => false,
        },
        Scope::Proposal { proposer: Proposer::Party(owner), proposal } => {
            *owner == party
                && (key == &Key::Tasks(TaskKey::PersonProposal(*proposal))
                    || key == &Key::Tasks(TaskKey::DecidedProposal(*proposal)))
        }
        Scope::EndedDelegates { task } => match key {
            Key::Tasks(TaskKey::EndedDelegate { requester, .. }) => requester == task,
            Key::Tasks(_) | Key::Core(_) | Key::People(_) | Key::Notes(_) => false,
        },
        Scope::Goals { project } => match key {
            Key::Tasks(TaskKey::Goal { project: owner, .. }) => owner == project,
            Key::Tasks(_) | Key::Core(_) | Key::People(_) | Key::Notes(_) => false,
        },
        Scope::Spend { project } => match key {
            Key::Tasks(
                TaskKey::Ledger(Funder::Period { project: owner, .. })
                | TaskKey::ClosedLedger(Funder::Period { project: owner, .. }),
            ) => owner == project,
            Key::Tasks(_) | Key::Core(_) | Key::People(_) | Key::Notes(_) => false,
        },
        Scope::Notes { project } => match key {
            Key::Notes(notes::Key::Line { scope, .. }) => match scope {
                notes::Scope::Deployment => *project == 0,
                notes::Scope::Project { project: owner }
                | notes::Scope::Goal { project: owner, .. }
                | notes::Scope::Resources { project: owner, .. } => owner == project,
            },
            Key::Notes(_) | Key::Tasks(_) | Key::Core(_) | Key::People(_) => false,
        },
        Scope::ScopedNotes { scope: wanted, .. } => match key {
            Key::Notes(notes::Key::Line { scope, .. }) => scope == wanted,
            Key::Notes(_) | Key::Tasks(_) | Key::Core(_) | Key::People(_) => false,
        },
        Scope::Inbox { group: Group::Updates | Group::Results | Group::Replies, .. } => match key {
            Key::Tasks(TaskKey::Update { party: owner, .. }) => *owner == party,
            Key::Tasks(_) | Key::Core(_) | Key::People(_) | Key::Notes(_) => false,
        },
        Scope::Conversations => match key {
            Key::Tasks(
                TaskKey::ConversationNewest { party: owner, .. } | TaskKey::ConversationOldest { party: owner, .. },
            ) => *owner == party,
            Key::Tasks(_) | Key::Core(_) | Key::People(_) | Key::Notes(_) => false,
        },
        Scope::Tasks { .. }
        | Scope::Proposal { proposer: Proposer::Task(_), .. }
        | Scope::Escalation { .. }
        | Scope::Question { .. }
        | Scope::Turn { .. }
        | Scope::History { subtree: true, .. }
        | Scope::Activity { .. }
        | Scope::Inbox {
            group: Group::Needs | Group::Proposals | Group::Escalations | Group::Questions | Group::PersonTasks,
            ..
        } => false,
    }
}

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
        Read::Tasks { query, .. } => Some(Scope::Tasks { query: *query }),
        Read::Goals { project, .. } => Some(Scope::Goals { project: *project }),
        Read::Notes { project, .. } => Some(Scope::Notes { project: *project }),
        Read::ScopedNotes { scope, description, .. } => {
            Some(Scope::ScopedNotes { scope: scope.clone(), description: description.clone() })
        }
        Read::Spend { project, .. } => Some(Scope::Spend { project: *project }),
        Read::Frame | Read::Project { .. } | Read::Deployment | Read::Application { .. } => None,
    }
}

fn cursor(read: &Read) -> Option<&Cursor> {
    match read {
        Read::EndedDelegates { from, .. }
        | Read::History { from, .. }
        | Read::Inbox { from, .. }
        | Read::Conversations { from }
        | Read::Tasks { from, .. }
        | Read::Goals { from, .. }
        | Read::Notes { from, .. }
        | Read::ScopedNotes { from, .. }
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
        | Read::Deployment
        | Read::Application { .. } => None,
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
        Read::Tasks { query, .. } => Some(Subject::Query { query: *query }),
        Read::Goals { project, .. } => Some(Subject::Goals { project: *project }),
        Read::Application { .. }
        | Read::EndedDelegates { .. }
        | Read::History { .. }
        | Read::Activity { .. }
        | Read::Notes { .. }
        | Read::ScopedNotes { .. }
        | Read::Spend { .. }
        | Read::Reply { .. }
        | Read::Turn { .. } => None,
    }
}

fn backwards(scope: &Scope) -> bool {
    match scope {
        Scope::Activity { .. } | Scope::Conversations | Scope::Escalation { .. } | Scope::Proposal { .. } => true,
        Scope::Task { .. }
        | Scope::Question { .. }
        | Scope::PersonTask { .. }
        | Scope::Result { .. }
        | Scope::Reply { .. }
        | Scope::Turn { .. }
        | Scope::EndedDelegates { .. }
        | Scope::History { .. }
        | Scope::Inbox {
            group: Group::Needs | Group::Proposals | Group::Escalations | Group::Questions | Group::PersonTasks,
            ..
        }
        | Scope::Inbox { group: Group::Updates | Group::Results | Group::Replies, .. }
        | Scope::Tasks { .. }
        | Scope::Goals { .. }
        | Scope::Notes { .. }
        | Scope::ScopedNotes { .. }
        | Scope::Spend { .. } => false,
    }
}

fn refused_application(reply: ReplyTo, read: &Read, why: Why) -> Requests {
    match read {
        Read::Application { connector, read, .. } => {
            let mut out = Queue::with_capacity(3);
            out.push(Request::Ask { connector: *connector, ask: crate::Ask::DropDocument { subject: *read } });
            out.push(Request::Held(Box::new(Held::Read { reply, answer: Answer::None(why) })));
            out.push(Request::Decided);
            Requests::Out(out)
        }
        Read::Frame
        | Read::Task { .. }
        | Read::Proposal { .. }
        | Read::Escalation { .. }
        | Read::Question { .. }
        | Read::PersonTask { .. }
        | Read::Result { .. }
        | Read::Reply { .. }
        | Read::Turn { .. }
        | Read::EndedDelegates { .. }
        | Read::History { .. }
        | Read::Activity { .. }
        | Read::Inbox { .. }
        | Read::Conversations { .. }
        | Read::Tasks { .. }
        | Read::Goals { .. }
        | Read::Notes { .. }
        | Read::ScopedNotes { .. }
        | Read::Spend { .. }
        | Read::Project { .. }
        | Read::Deployment => terminal(reply, Answer::None(why)),
    }
}

pub(crate) fn start(core: &mut Core, env: &Env<Limits>, reply: ReplyTo, sign_in: u64, read: Read) -> Requests {
    let Some(party) = core.people.person(sign_in, env.now, env.wall) else {
        let why = if core.people.ready() { Why::SignedOut } else { Why::Unknown };
        return refused_application(reply, &read, why);
    };
    match &read {
        Read::Application { scope, connector, read: name, window } => {
            if (*scope != documents::ApplicationScope::Own && core.people.role(party, scope.project()).is_none())
                || !core.connectors.as_ref().contains(connector)
            {
                return refused_application(reply, &read, Why::Unknown);
            }
            let mut out = Queue::with_capacity(2);
            out.push(Request::Ask {
                connector: *connector,
                ask: crate::Ask::Read { reply, scope: *scope, read: *name, party, window: *window },
            });
            out.push(Request::Decided);
            return Requests::Out(out);
        }
        Read::Frame
        | Read::Task { .. }
        | Read::Proposal { .. }
        | Read::Escalation { .. }
        | Read::Question { .. }
        | Read::PersonTask { .. }
        | Read::Result { .. }
        | Read::Reply { .. }
        | Read::Turn { .. }
        | Read::EndedDelegates { .. }
        | Read::History { .. }
        | Read::Activity { .. }
        | Read::Inbox { .. }
        | Read::Conversations { .. }
        | Read::Tasks { .. }
        | Read::Goals { .. }
        | Read::Notes { .. }
        | Read::ScopedNotes { .. }
        | Read::Spend { .. }
        | Read::Project { .. }
        | Read::Deployment => {}
    }
    start_jig(core, env, reply, sign_in, party, read)
}

fn bounded_query(read: &Read, limits: &Limits) -> bool {
    match read {
        Read::ScopedNotes { scope, description, .. } => {
            jig_domain_notes::valid_scope(scope, &limits.notes)
                && match description {
                    Some(query) => {
                        query.len() <= usize::try_from(limits.notes.description_bytes).expect("description bound fits")
                    }
                    None => true,
                }
        }
        Read::Frame
        | Read::Task { .. }
        | Read::Proposal { .. }
        | Read::Escalation { .. }
        | Read::Question { .. }
        | Read::PersonTask { .. }
        | Read::Result { .. }
        | Read::Reply { .. }
        | Read::Turn { .. }
        | Read::EndedDelegates { .. }
        | Read::History { .. }
        | Read::Activity { .. }
        | Read::Inbox { .. }
        | Read::Conversations { .. }
        | Read::Tasks { .. }
        | Read::Goals { .. }
        | Read::Project { .. }
        | Read::Notes { .. }
        | Read::Spend { .. }
        | Read::Deployment
        | Read::Application { .. } => true,
    }
}

struct ReadStart {
    after_live: Option<documents::Cursor>,
    after_store: Option<Position>,
    stored: bool,
    live_before: Option<documents::Cursor>,
    reverse_live: bool,
}

fn read_start(read: &Read, party: u64) -> Option<ReadStart> {
    let target = scope(read);
    let mut live_before = None;
    let mut reverse_live = false;
    let mut after_live = None;
    let mut after_store = None;
    let mut stored = false;
    match cursor(read) {
        Some(Cursor::Live { scope, after, store_after }) => {
            if Some(scope) != target.as_ref() {
                return None;
            }
            subject(read, party)?;
            after_live = Some(*after);
            after_store.clone_from(store_after);
        }
        Some(Cursor::ConversationsBefore { before }) => {
            if target != Some(Scope::Conversations) {
                return None;
            }
            stored = true;
            after_store = Some(before.clone());
        }
        Some(Cursor::ConversationsLiveBefore { before }) => {
            if target != Some(Scope::Conversations) {
                return None;
            }
            live_before = *before;
            reverse_live = true;
        }
        Some(Cursor::ConversationsStart) if target != Some(Scope::Conversations) => return None,
        Some(Cursor::Store { scope: Scope::Tasks { .. }, .. }) => return None,
        Some(Cursor::Store { scope, after }) => {
            if Some(scope) != target.as_ref() {
                return None;
            }
            stored = true;
            after_store.clone_from(after);
        }
        Some(Cursor::ConversationsStart) | None => {}
    }
    Some(ReadStart { after_live, after_store, stored, live_before, reverse_live })
}

fn start_jig(core: &mut Core, env: &Env<Limits>, reply: ReplyTo, sign_in: u64, party: u64, read: Read) -> Requests {
    if !bounded_query(&read, &env.limits) {
        return terminal(reply, Answer::None(Why::Busy));
    }
    let target = scope(&read);
    let Some(start) = read_start(&read, party) else {
        return terminal(reply, Answer::None(Why::Unknown));
    };
    if start.reverse_live {
        return terminal(reply, cursors::live_before(core, env, party, start.live_before.as_ref()));
    }
    let ReadStart { after_live, after_store, stored, live_before: _, reverse_live: _ } = start;
    if let Some(position) = &after_store
        && match &target {
            Some(scope) => !rows::position_allowed(scope, position),
            None => true,
        }
    {
        return terminal(reply, Answer::None(Why::Unknown));
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
                if !rows::needs_store(&read, &document) {
                    let answer = rows::live_page(core, &env.limits, party, &read, document);
                    return terminal(reply, answer);
                }
            }
            Err(Missing::NotAllowed) => return terminal(reply, Answer::None(Why::Unknown)),
            Err(Missing::TooLarge) => return terminal(reply, Answer::None(Why::Busy)),
            Err(Missing::Unknown | Missing::Ended) => {}
        }
    }
    let Some(scope) = target else {
        return terminal(reply, Answer::None(Why::Unknown));
    };
    if !rows::scope_allowed(core, party, &scope) {
        return terminal(reply, Answer::None(Why::Unknown));
    }
    if core.reads.len() >= env.limits.load_slots {
        return terminal(reply, Answer::None(Why::Busy));
    }
    let range = Range {
        party,
        backwards: match cursor(&read) {
            Some(Cursor::ConversationsBefore { .. }) => false,
            Some(
                Cursor::ConversationsLiveBefore { .. }
                | Cursor::ConversationsStart
                | Cursor::Live { .. }
                | Cursor::Store { .. },
            )
            | None => backwards(&scope),
        },
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
    let answer = if core.people.person(pending.sign_in, env.now, env.wall) != Some(pending.party) {
        Answer::None(Why::SignedOut)
    } else if !rows::scope_allowed(core, pending.party, &pending.range.scope) {
        Answer::None(Why::Unknown)
    } else {
        match result {
            Loaded::Busy => Answer::None(Why::Busy),
            Loaded::Gone => Answer::None(Why::Gone),
            Loaded::Page { rows, next } => rows::from_store(core, &env.limits, &pending.range, rows, next),
        }
    };
    let reply = core.reads.get_mut(name).expect("live read").reply.take().expect("terminal consumes its reply");
    core.reads.retire(name);
    terminal(reply, answer)
}

/// Maximum answer payload retained in a root's held-output slot.
#[must_use]
pub fn answer_worst_case(limits: &Limits) -> Option<u64> {
    let document = documents::document_with_context_worst_case(limits)?;
    u64::try_from(size_of::<Answer>()).ok()?.checked_add(
        document
            .checked_mul(u64::from(limits.documents.rows).checked_add(3)?)?
            .checked_add(limits.load_bytes.checked_mul(2)?)?
            .checked_add(List::<PageRow>::worst_case(limits.documents.rows)?)?
            .checked_add(u64::try_from(size_of::<Page>()).ok()?)?
            .checked_add(query_worst_case(limits)?)?,
    )
}

// Resource pattern segments and literal bytes, plus one description query.
fn query_worst_case(limits: &Limits) -> Option<u64> {
    u64::from(limits.notes.pattern_bytes)
        .checked_mul(u64::try_from(size_of::<Box<[u8]>>()).ok()?.checked_add(1)?)?
        .checked_add(u64::from(limits.notes.description_bytes))
}

/// Pending load slots, one incoming bounded page and its constructed answer.
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    if limits.load_bytes == 0 || limits.load_slots == 0 {
        return None;
    }
    Slab::<Pending>::worst_case(limits.load_slots)?
        // Each pending query, plus the incoming read, its cursor and the target
        // clone during admission. The emitted range and page cursor are priced
        // in the held output's answer bound.
        .checked_add(u64::from(limits.load_slots).checked_add(3)?.checked_mul(query_worst_case(limits)?)?)?
        .checked_add(u64::from(limits.load_slots).checked_mul(u64::try_from(size_of::<Position>()).ok()?)?)?
        .checked_add(limits.load_bytes)?
        // Reverse live reads retain one bounded tail while making producer pages.
        .checked_add(Queue::<PageRow>::worst_case(limits.documents.rows)?)?
        .checked_add(answer_worst_case(limits)?)
}
