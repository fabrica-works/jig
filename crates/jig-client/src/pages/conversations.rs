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
        self, Answer, Ask, ConversationLine, InboxGroup, InboxReadGroup, Limit, ObjectKey, Read, Refusal, Seen, Text,
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
        Some(Body::Note { project, document }) => (Some(*project), Some(document.description.bytes.clone())),
        None => (None, None),
    };
    let mut project = project.or(object.project);
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
        IntentPurpose::SetGoal => crate::pages::project::goals::answered(domain, env, request, generation, answer, out),
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
                Answer::Done(outcome) => match crate::outcome::route(&outcome) {
                    crate::outcome::Route::Chat { task } => {
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
                                    answer: Answer::Done(outcome),
                                },
                            );
                        }
                    }
                    crate::outcome::Route::Goal { .. }
                    | crate::outcome::Route::Words { .. }
                    | crate::outcome::Route::Proposed
                    | crate::outcome::Route::Other => {
                        crate::notices::show(
                            domain,
                            crate::notices::Kind::PageAnswer { answer: Answer::Done(outcome) },
                        );
                    }
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
    for entry in &domain.inbox.rows {
        if entry.group == InboxGroup::Replies
            && entry.object == ObjectKey::Task(task)
            && let Some(line) = &entry.reply
            && line.chip.task == task
            && line.project == entry.project
            && line.last_reply > line.seen
        {
            through = Some(line.last_reply);
        }
    }
    for row in &domain.conversation_rows.rows {
        if row.chip.task == task && row.last_reply > row.seen {
            through = Some(through.unwrap_or(0).max(row.last_reply));
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
    if domain.inbox.filter == filter
        && (domain.inbox.needs_window.reading.is_some() || domain.inbox.updates_window.reading.is_some())
    {
        return;
    }
    domain.inbox.filter(filter);
    start_inbox(domain, out);
    domain.changed();
}
pub(crate) fn start_inbox(domain: &mut Domain, out: &mut Queue<Request>) {
    let filter = domain.inbox.filter;
    for group in [InboxReadGroup::Needs, InboxReadGroup::Updates] {
        if let Some(kind) = filter.kind
            && InboxReadGroup::of(kind) != group
        {
            continue;
        }
        ask_read(
            domain,
            inbox_number(group),
            Read::Inbox { group, project: filter.project, kind: filter.kind, goal: filter.goal, after: None },
            out,
        );
    }
}
const fn inbox_number(group: InboxReadGroup) -> u16 {
    match group {
        InboxReadGroup::Needs => 10,
        InboxReadGroup::Updates => 12,
    }
}
pub(crate) const fn inbox_group(number: u16) -> Option<InboxReadGroup> {
    match number {
        10 => Some(InboxReadGroup::Needs),
        12 => Some(InboxReadGroup::Updates),
        _ => None,
    }
}
fn ask_read(domain: &mut Domain, number: u16, what: Read, out: &mut Queue<Request>) {
    for (id, ()) in &domain.reads.ids {
        let reading = domain.reads.slab.get(*id).expect("registered read exists");
        if reading.what == what && reading.generation == domain.generation && !reading.abandoned {
            if let Some(group) = inbox_group(number) {
                domain.inbox.window_mut(group).reading = Some(id.token());
            }
            return;
        }
    }
    match domain.reads.ask(domain.generation, Source::Window { generation: domain.generation, number }, what, out) {
        Some(id) => {
            if let Some(group) = inbox_group(number) {
                domain.inbox.window_mut(group).reading = Some(id.token());
            }
        }
        None => crate::pages::routing::refuse(domain, Limit::Reads),
    }
}
/// The Needs window's binding number.
pub const INBOX_WINDOW: u64 = 10;
/// The Updates window's independent binding number.
pub const INBOX_UPDATES_WINDOW: u64 = 12;
/// The conversations window's binding number.
pub const CONVERSATIONS_WINDOW: u64 = 11;
pub(crate) fn more(domain: &mut Domain, window: Token, end: End, out: &mut Queue<Request>) {
    if window == Token::new(INBOX_WINDOW) || window == Token::new(INBOX_UPDATES_WINDOW) {
        let group = if window == Token::new(INBOX_WINDOW) { InboxReadGroup::Needs } else { InboxReadGroup::Updates };
        let filter = domain.inbox.filter;
        if let Some(kind) = filter.kind
            && InboxReadGroup::of(kind) != group
        {
            return;
        }
        let selected = domain.inbox.window(group);
        let cursor = match end {
            End::Before => selected.before.clone(),
            End::After => selected.after.clone(),
        };
        if cursor.is_some() || selected.more {
            ask_read(
                domain,
                inbox_number(group),
                Read::Inbox { group, project: filter.project, kind: filter.kind, goal: filter.goal, after: cursor },
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
                match domain.address {
                    Address::Conversations { .. } => {}
                    Address::Inbox { .. } | Address::Task { .. } | Address::Project { .. } | Address::Deployment => {
                        return;
                    }
                }
                ask_read(domain, 11, Read::Conversations { after: Some(cursor.clone()) }, out);
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
/// Heap for inbox lineage and whole reply-line context, cursors and the chat form.
#[must_use]
pub fn worst_case(limits: &crate::Limits) -> Option<u64> {
    limits.rows.checked_mul(3)?;
    List::<contract::InboxEntry>::worst_case(limits.rows)?
        .checked_mul(7)?
        .checked_add(u64::from(limits.rows).checked_mul(limits.document_bytes)?.checked_mul(6)?)?
        .checked_add(List::<ObjectKey>::worst_case(limits.rows.checked_mul(3)?)?)?
        .checked_add(List::<crate::objects::Context>::worst_case(limits.rows.checked_mul(3)?)?)?
        .checked_add(u64::from(limits.text).checked_mul(4)?)?
        .checked_add(limits.document_bytes)
}
