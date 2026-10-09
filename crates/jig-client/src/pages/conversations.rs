//! The party's chat list and first-words form, with page-owned keyed intents.
//! List rows retain committed runtime, spend and unread facts; card objects are
//! shared by key. No charter or budget is chosen here: the frame offers both.
//! A pending start survives page navigation and opens its task only after a durable
//! answer. Historical reads are bounded and use opaque cursors. See pages.md, 4.
//!
//! | Form | Event | Next | Emits |
//! |---|---|---|---|
//! | ready/refused | submit | pending | one start-chat intent |
//! | pending | submit | pending | nothing |
//! | pending | refused | refused | nothing |
//! | pending | made chat | ready | navigate to chat activity |
use crate::{
    Domain, Request,
    action::End,
    address::{Address, Place, TaskSection},
    contract::{
        self, Answer, Ask, ConversationLine, InboxGroup, Limit, Made, ObjectKey, Outcome, Read, Refusal, Seen, Text,
    },
    fields::Purpose,
    intents::Origin,
    objects::{Body, Source},
    pages::{IntentPurpose, Page, inbox::Filter},
    windows::Edge,
};
use alloc::boxed::Box;
use skein_lib::{Env, List, Queue, Token};
/// First-words submission and its refusal, held apart from the list window.
#[derive(Debug)]
pub struct State {
    pub starting: Option<Token>,
    pub refusal: Option<Refusal>,
    pub project: Option<u32>,
    pub scope: Option<u64>,
    pub words: Option<Text>,
    pub more: bool,
}
impl State {
    pub(crate) const fn new() -> Self {
        Self { starting: None, refusal: None, project: None, scope: None, words: None, more: false }
    }
}
/// Starts a chat with only a project and the first words; a second press joins.
pub fn start(domain: &mut Domain, _env: &Env<crate::Limits>, project: u32, out: &mut Queue<Request>) {
    if domain.blocked || domain.chats.starting.is_some() {
        return;
    }
    let Some(scope) = domain.confirmed else {
        return;
    };
    let mut offered = false;
    if let Some(frame) = &domain.frame.document {
        for role in &frame.projects {
            if role.project == project && !role.chats.is_empty() {
                offered = true;
            }
        }
    }
    if !offered {
        domain.chats.refusal = Some(Refusal::NotOffered);
        domain.changed();
        return;
    }
    let Some(field) = domain.fields.get(Purpose::NewChat) else {
        return;
    };
    if field.text.bytes().is_empty() {
        return;
    }
    let words = field.text.clone();
    let ask = Box::new(Ask::StartChat { project, words: words.clone() });
    match domain.intents.ask(
        scope,
        Origin::Page { generation: domain.generation, purpose: IntentPurpose::StartChat },
        ask,
        out,
    ) {
        Some(id) => {
            domain.chats.starting = Some(id.token());
            domain.chats.refusal = None;
            domain.chats.project = Some(project);
            domain.chats.scope = Some(scope);
            domain.chats.words = Some(words);
        }
        None => domain.chats.refusal = Some(Refusal::Limit { which: Limit::Requests }),
    }
    domain.changed();
}
/// Starts from a card, naming the thing by its committed title in the first words.
pub fn about(domain: &mut Domain, env: &Env<crate::Limits>, key: ObjectKey, out: &mut Queue<Request>) {
    let Some(id) = domain.objects.find(key) else {
        return;
    };
    let Some(object) = domain.objects.get(id) else {
        return;
    };
    let (project, title) = match &object.body {
        Some(Body::Task(task)) => (Some(task.project), Some(task.chip.title.clone())),
        Some(Body::Chip(chip)) => (None, Some(chip.title.clone())),
        Some(Body::Escalation(value)) => (None, Some(value.task.title.clone())),
        Some(Body::Question(value)) => (None, Some(value.task.title.clone())),
        Some(Body::PersonTask(value)) => (None, Some(value.task.title.clone())),
        Some(Body::Result(value)) => (None, Some(value.task.title.clone())),
        Some(Body::Proposal(value)) => (Some(value.project), Some(value.why.clone())),
        None => (None, None),
    };
    let mut project = project;
    if project.is_none() {
        for entry in &domain.inbox.rows {
            if entry.object == key {
                project = Some(entry.project);
                break;
            }
        }
    }
    let Some(project) = project else {
        return;
    };
    let Some(title) = title else {
        return;
    };
    match domain.write_field(Purpose::NewChat, title) {
        Ok(()) => start(domain, env, project, out),
        Err(which) => crate::pages::routing::refuse(domain, which),
    }
}
pub(crate) fn answered(
    domain: &mut Domain,
    env: &Env<crate::Limits>,
    request: Token,
    generation: u32,
    purpose: IntentPurpose,
    answer: Answer,
    out: &mut Queue<Request>,
) {
    match purpose {
        IntentPurpose::Seen(_) => {
            match answer {
                Answer::Done(_) => {}
                Answer::Refused(refusal) => {
                    crate::notices::show(domain, crate::notices::Kind::PageAnswer { answer: Answer::Refused(refusal) });
                }
            }
            domain.changed();
        }
        IntentPurpose::StartChat => {
            if domain.chats.starting != Some(request) {
                return;
            }
            domain.chats.starting = None;
            match answer {
                Answer::Refused(refusal) => domain.chats.refusal = Some(refusal),
                Answer::Done(outcome) => match outcome {
                    Outcome::Made { what: Made::Chat { task } } => {
                        let same = match &domain.chats.words {
                            Some(words) => match domain.fields.get(Purpose::NewChat) {
                                Some(field) => field.text == *words,
                                None => false,
                            },
                            None => false,
                        };
                        if same && domain.chats.scope == domain.confirmed {
                            let empty = Text::new(Box::from([]), domain.text_limit).expect("empty words fit");
                            match domain.write_field(Purpose::NewChat, empty) {
                                Ok(()) => {}
                                Err(which) => crate::pages::routing::refuse(domain, which),
                            }
                        }
                        let address = Address::Task { task, section: TaskSection::Activity };
                        if generation == domain.generation && domain.chats.scope == domain.confirmed {
                            crate::pages::routing::navigate(domain, env, Place::Jig(address), false, out);
                        } else {
                            crate::notices::show(
                                domain,
                                crate::notices::Kind::Answer {
                                    object: ObjectKey::Task(task),
                                    answer: Answer::Done(Outcome::Made { what: Made::Chat { task } }),
                                },
                            );
                        }
                    }
                    Outcome::Made {
                        what:
                            Made::Words { .. }
                            | Made::Goal { .. }
                            | Made::Tasks { .. }
                            | Made::Service { .. }
                            | Made::PersonResult { .. }
                            | Made::Note { .. },
                    }
                    | Outcome::Proposed { .. }
                    | Outcome::Decided { .. }
                    | Outcome::DecidedFirst { .. }
                    | Outcome::Changed => {}
                },
            }
            domain.chats.words = None;
            domain.changed();
        }
    }
}
fn seen(domain: &mut Domain, value: Seen, out: &mut Queue<Request>) -> bool {
    let Some(scope) = domain.confirmed else {
        return false;
    };
    domain
        .intents
        .ask(
            scope,
            Origin::Page { generation: domain.generation, purpose: IntentPurpose::Seen(value) },
            Box::new(Ask::MarkSeen { seen: value }),
            out,
        )
        .is_some()
}
pub(crate) fn open_updates(domain: &mut Domain, out: &mut Queue<Request>) {
    match domain.address {
        Address::Inbox { .. } => {}
        Address::Conversations { .. } | Address::Task { .. } | Address::Project { .. } | Address::Deployment => return,
    }
    domain.inbox.opening_updates = true;
    mark_results(domain, out);
}
pub(crate) fn mark_results(domain: &mut Domain, out: &mut Queue<Request>) {
    if !domain.inbox.opening_updates {
        return;
    }
    let mut through = domain.inbox.marked_results;
    for entry in &domain.inbox.rows {
        match entry.group {
            InboxGroup::Results => through = through.max(entry.position),
            InboxGroup::Questions
            | InboxGroup::Proposals
            | InboxGroup::Escalations
            | InboxGroup::PersonTasks
            | InboxGroup::Replies => {}
        }
    }
    if through > domain.inbox.marked_results && seen(domain, Seen::Results { through }, out) {
        domain.inbox.marked_results = through;
        domain.inbox.opening_updates = false;
    }
}
pub(crate) fn open_chat(domain: &mut Domain, env: &Env<crate::Limits>, task: u64, out: &mut Queue<Request>) {
    let mut through = None;
    for row in &domain.conversation_rows.rows {
        if row.chip.task == task && row.last_reply > row.seen {
            through = Some(row.last_reply);
        }
    }
    if let Some(through) = through {
        let sent = seen(domain, Seen::Replies { chat: task, through }, out);
        if !sent {
            crate::pages::routing::refuse(domain, Limit::Requests);
        }
    }
    crate::pages::routing::navigate(
        domain,
        env,
        Place::Jig(Address::Task { task, section: TaskSection::Activity }),
        false,
        out,
    );
}
pub(crate) fn filter_inbox(domain: &mut Domain, filter: Filter, out: &mut Queue<Request>) {
    match domain.page {
        Page::Work { address: Address::Inbox { .. }, .. } => {}
        Page::Work { .. }
        | Page::Starting
        | Page::Elsewhere
        | Page::SigningIn
        | Page::Missing
        | Page::OutOfDate
        | Page::SomeoneElse => return,
    }
    if domain.inbox.filter == filter && domain.inbox.reading.is_some() {
        return;
    }
    domain.inbox.filter = filter;
    domain.inbox.reading = None;
    domain.inbox.rows.clear();
    domain.inbox.before = None;
    domain.inbox.after = None;
    let what = Read::Inbox { project: filter.project, kind: filter.kind, goal: filter.goal, after: None };
    ask_read(domain, 10, what, out);
    domain.changed();
}
fn ask_read(domain: &mut Domain, number: u16, what: Read, out: &mut Queue<Request>) {
    for (id, ()) in &domain.reads.ids {
        let reading = domain.reads.slab.get(*id).expect("registered read exists");
        if reading.what == what && reading.generation == domain.generation && !reading.abandoned {
            if number == 10 {
                domain.inbox.reading = Some(id.token());
            }
            return;
        }
    }
    match domain.reads.ask(domain.generation, Source::Window { generation: domain.generation, number }, what, out) {
        Some(id) => {
            if number == 10 {
                domain.inbox.reading = Some(id.token());
            }
        }
        None => crate::pages::routing::refuse(domain, Limit::Reads),
    }
}
/// Stable window numbers decoded from jig's own view bindings.
pub const INBOX_WINDOW: u64 = 10;
/// The conversations window's binding number.
pub const CONVERSATIONS_WINDOW: u64 = 11;
pub(crate) fn more(domain: &mut Domain, window: Token, end: End, out: &mut Queue<Request>) {
    if window == Token::new(INBOX_WINDOW) {
        let cursor = match end {
            End::Before => domain.inbox.before.clone(),
            End::After => domain.inbox.after.clone(),
        };
        let filter = domain.inbox.filter;
        if cursor.is_some() || domain.inbox.more {
            ask_read(
                domain,
                10,
                Read::Inbox { project: filter.project, kind: filter.kind, goal: filter.goal, after: cursor },
                out,
            );
        }
    } else if window == Token::new(CONVERSATIONS_WINDOW) {
        let edge = match end {
            End::Before => &domain.conversation_rows.edges.before,
            End::After => &domain.conversation_rows.edges.after,
        };
        match edge {
            Edge::More(cursor) => {
                let project = match domain.address {
                    Address::Conversations { project } => project,
                    Address::Inbox { .. } | Address::Task { .. } | Address::Project { .. } | Address::Deployment => {
                        return;
                    }
                };
                ask_read(domain, 11, Read::Conversations { project, after: Some(cursor.clone()) }, out);
            }
            Edge::End => {}
        }
    }
}
/// Sorts a bounded list with live chats first and most recent activity first.
pub(crate) fn order(rows: &mut List<ConversationLine>) {
    for first in 0..rows.len() {
        for second in first.checked_add(1).expect("row index fits")..rows.len() {
            let a = rows.get(first).expect("row exists");
            let b = rows.get(second).expect("row exists");
            let a_live = live(a.chip.phase);
            let b_live = live(b.chip.phase);
            if (b_live && !a_live) || (a_live == b_live && b.changed > a.changed) {
                let b = b.clone();
                let a = core::mem::replace(rows.get_mut(first).expect("row exists"), b);
                *rows.get_mut(second).expect("row exists") = a;
            }
        }
    }
}
/// Whether the chat has a live phase, independent of the runtime's words.
#[must_use]
pub const fn live(phase: contract::Phase) -> bool {
    match phase {
        contract::Phase::Done | contract::Phase::Failed | contract::Phase::Cancelled => false,
        contract::Phase::Waiting
        | contract::Phase::Idle
        | contract::Phase::Due
        | contract::Phase::Preparing
        | contract::Phase::Claimed
        | contract::Phase::Running
        | contract::Phase::BackingOff
        | contract::Phase::Active
        | contract::Phase::Closing
        | contract::Phase::Held => true,
    }
}
/// Heap for inbox context, its cursors and the one chat form refusal.
#[must_use]
pub fn worst_case(limits: &crate::Limits) -> Option<u64> {
    List::<contract::InboxEntry>::worst_case(limits.rows)?
        .checked_add(u64::from(limits.rows).checked_mul(limits.document_bytes)?.checked_mul(2)?)?
        .checked_add(u64::from(limits.text).checked_mul(2)?)?
        .checked_add(limits.document_bytes)
}
