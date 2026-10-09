//! Spine routing of page generations, watches, reads, names and parts.
//! Detailed page policies are jig 22's modules. See client/domain/client.md, 5–7.
use crate::{
    Domain, Request,
    action::End,
    address::{Address, Place, ProjectSection, TaskSection},
    contract::{self, Delivery, Limit, ObjectKey, Part, PartyKind, Read, ReadResult, ReadValue, Subject},
    facts::Fact,
    objects::{Body, Source},
    pages::Page,
    watches::{Owner, Watch},
    windows::{Edge, Edges},
};
use alloc::boxed::Box;
use skein_lib::{Env, Id, List, Queue, Token};

fn blank_edges() -> Edges {
    Edges { before: Edge::End, after: Edge::End }
}
fn page_edges(before: Option<contract::Cursor>, after: Option<contract::Cursor>) -> Edges {
    let before = match before {
        Some(cursor) => Edge::More(cursor),
        None => Edge::End,
    };
    let after = match after {
        Some(cursor) => Edge::More(cursor),
        None => Edge::End,
    };
    Edges { before, after }
}
pub(crate) fn refuse(domain: &mut Domain, which: Limit) {
    match &mut domain.page {
        Page::Work { refused, .. } => *refused = Some(which),
        Page::Starting | Page::Elsewhere | Page::SigningIn | Page::Missing | Page::OutOfDate | Page::SomeoneElse => {
            crate::notices::show(domain, crate::notices::Kind::Refused(which));
        }
    }
    domain.changed();
}
fn excess(domain: &mut Domain, count: u32) {
    match &mut domain.page {
        Page::Work { excess, .. } => *excess = count,
        Page::Starting | Page::Elsewhere | Page::SigningIn | Page::Missing | Page::OutOfDate | Page::SomeoneElse => {}
    }
    domain.changed();
}
fn open(domain: &mut Domain, env: &Env<crate::Limits>, subject: Subject, out: &mut Queue<Request>) {
    match domain.open(env, subject, Owner::Page { generation: domain.generation }, out) {
        Some(_) => {}
        None => refuse(domain, Limit::Watches),
    }
}
fn read(domain: &mut Domain, number: u16, what: Read, out: &mut Queue<Request>) {
    let source = Source::Window { generation: domain.generation, number };
    match domain.reads.ask(domain.generation, source, what, out) {
        Some(_) => {}
        None => refuse(domain, Limit::Reads),
    }
}
/// Leaves in the order the contract requires; shell places replace their current entry.
pub(crate) fn navigate(
    domain: &mut Domain,
    env: &Env<crate::Limits>,
    place: Place,
    replace: bool,
    out: &mut Queue<Request>,
) {
    if domain.blocked {
        return;
    }
    let Some(generation) = domain.generation.checked_add(1) else {
        refuse(domain, Limit::Reads);
        return;
    };
    match domain.page {
        Page::SigningIn => domain.signin.returning = true,
        Page::Starting | Page::Work { .. } | Page::Elsewhere | Page::Missing | Page::OutOfDate | Page::SomeoneElse => {}
    }
    domain.close_page(out);
    domain.reads.abandon(domain.generation);
    out.push(Request::PartsLeft { generation: domain.generation });
    domain.objects.clear();
    domain.task_page.clear();
    domain.clock.page_times = false;
    domain.inbox.rows.clear();
    domain.inbox.reading = None;
    domain.inbox.before = None;
    domain.inbox.after = None;
    domain.inbox.more = false;
    domain.inbox.opening_updates = false;
    domain.object_rows.rows.clear();
    domain.object_rows.edges = blank_edges();
    domain.conversation_rows.rows.clear();
    domain.conversation_rows.edges = blank_edges();
    let rows = domain.conversation.lines.rows.capacity();
    let words = domain.conversation.words.rows.capacity();
    domain.conversation = crate::conversation::Conversation::new(0, rows);
    domain.conversation.words = crate::conversation::KeptWords::new(words);
    domain.history_rows.rows.clear();
    domain.history_rows.edges = blank_edges();
    domain.generation = generation;
    match place {
        Place::Jig(address) => {
            domain.address = address;
            match domain.frame.document {
                Some(_) => enter(domain, env, out),
                None => {
                    domain.page = Page::Starting;
                    domain.changed();
                }
            }
            out.push(Request::Show { address, replace });
            crate::store::place(domain, out);
        }
        Place::Elsewhere => {
            domain.page = Page::Elsewhere;
            domain.changed();
        }
        Place::Missing => {
            domain.page = Page::Missing;
            domain.changed();
        }
    }
    crate::signin::returned(domain, env, out);
}
/// Opens only what the selected work page shows; services have no page.
pub(crate) fn enter(domain: &mut Domain, env: &Env<crate::Limits>, out: &mut Queue<Request>) {
    if domain.confirmed.is_none() {
        domain.page = Page::SomeoneElse;
        domain.changed();
        return;
    }
    if let Some(frame) = &domain.frame.document {
        match frame.party.kind {
            PartyKind::Service => {
                domain.page = Page::Elsewhere;
                domain.changed();
                return;
            }
            PartyKind::Person => {}
        }
    }
    if domain.address == Address::Deployment {
        let allowed = match &domain.frame.document {
            Some(frame) => frame.deployment_role.is_some(),
            None => false,
        };
        if !allowed {
            domain.page = Page::Missing;
            domain.changed();
            return;
        }
    }
    domain.page =
        Page::Work { address: domain.address, generation: domain.generation, behind: false, excess: 0, refused: None };
    domain.changed();
    match domain.address {
        Address::Inbox { project } => {
            domain.inbox.filter = crate::pages::inbox::Filter { project, kind: None, goal: None };
            open(domain, env, Subject::Inbox { window: env.limits.rows }, out);
        }
        Address::Conversations { project } => {
            open(domain, env, Subject::Conversations, out);
            read(domain, 11, Read::Conversations { project, after: None }, out);
        }
        Address::Task { task, section } => {
            domain.conversation.task = task;
            open(domain, env, Subject::Task { task }, out);
            match section {
                TaskSection::Overview
                | TaskSection::Budget
                | TaskSection::Panels
                | TaskSection::Activity
                | TaskSection::Plan => {}
                TaskSection::History => read(domain, 2, Read::History { task, subtree: false, after: None }, out),
            }
        }
        Address::Project { project, section } => {
            match section {
                ProjectSection::Overview | ProjectSection::Goals | ProjectSection::Tasks => {
                    open(domain, env, Subject::Goals { project }, out);
                }
                ProjectSection::Notes => read(
                    domain,
                    3,
                    Read::Notes { scope: contract::NoteScope::Project(project), description: None, after: None },
                    out,
                ),
                ProjectSection::Resources
                | ProjectSection::Policy
                | ProjectSection::Roles
                | ProjectSection::Spend
                | ProjectSection::Recurring => {}
            }
            read(domain, 0, Read::Project { project }, out);
        }
        Address::Deployment => open(domain, env, Subject::Deployment, out),
    }
}
fn document_read(key: ObjectKey) -> Read {
    match key {
        ObjectKey::Task(task) => Read::Task { task },
        ObjectKey::Proposal(proposal) => Read::Proposal { proposal },
        ObjectKey::Escalation(task) => Read::Escalation { task },
        ObjectKey::Question { task, message } => Read::Question { task, message },
        ObjectKey::PersonTask(task) => Read::PersonTask { task },
        ObjectKey::Result(task) => Read::Result { task },
        ObjectKey::Note { project, .. } => {
            Read::Notes { scope: contract::NoteScope::Project(project), description: None, after: None }
        }
    }
}
pub(crate) fn names(
    domain: &mut Domain,
    env: &Env<crate::Limits>,
    source: Source,
    keys: List<ObjectKey>,
    out: &mut Queue<Request>,
) {
    let missed = domain.objects.replace(source, keys.as_slice());
    excess(domain, missed);
    domain.object_rows.rows.clear();
    for key in &keys {
        if domain.objects.named(*key, source) {
            match domain.object_rows.rows.push(*key) {
                Ok(()) => {}
                Err(_) => break,
            }
            let id = domain.objects.find(*key).expect("named object is present");
            let object = domain.objects.get(id).expect("indexed object exists");
            if object.body.is_none() {
                let what = document_read(*key);
                if !domain.reads.pending(&what) {
                    match domain.reads.ask(domain.generation, source, what, out) {
                        Some(_) => {}
                        None => refuse(domain, Limit::Reads),
                    }
                }
            }
        }
    }
    if missed > 0 {
        refuse(domain, Limit::Objects);
        out.push(Request::Fact { fact: Fact::Refused { which: Limit::Objects } });
    }
    // The page's names are bounded even when the incoming document has more rows.
    assert!(domain.object_rows.rows.len() <= env.limits.rows, "page rows fit their window");
}
pub(crate) fn parts(domain: &mut Domain, env: &Env<crate::Limits>, values: &[Part], out: &mut Queue<Request>) {
    let mut count = 0_u32;
    for part in values {
        if count == env.limits.parts {
            refuse(domain, Limit::Objects);
            break;
        }
        out.push(Request::ShowPart { part: part.clone(), generation: domain.generation });
        count = count.saturating_add(1);
    }
}
fn one(domain: &mut Domain, source: Source, key: ObjectKey, body: Body) {
    let missed = domain.objects.replace(source, &[key]);
    if missed > 0 {
        refuse(domain, Limit::Objects);
        return;
    }
    match domain.objects.find(key) {
        Some(_) => {
            let stored = domain.objects.put(key, body);
            assert!(stored, "named object receives its facts");
        }
        None => refuse(domain, Limit::Objects),
    }
    domain.changed();
}
/// Applies one whole delivery to its source, never to an old generation.
pub(crate) fn delivery(
    domain: &mut Domain,
    env: &Env<crate::Limits>,
    watch: Id<Watch>,
    value: Delivery,
    out: &mut Queue<Request>,
) {
    let source = Source::Watch(watch);
    match value {
        Delivery::Frame(_) => {}
        Delivery::Run(value) => crate::conversation::routing::stream(domain, env, watch, *value, out),
        Delivery::Task(task) => {
            crate::conversation::routing::document(domain, &task, out);
            let key = ObjectKey::Task(task.chip.task);
            one(domain, source, key, Body::Task(task));
            crate::pages::task::document(domain, env, out);
        }
        Delivery::Tree(tree) => {
            crate::pages::task::tree(domain, *tree);
            let mut keys = List::with_capacity(env.limits.rows);
            for node in &domain.task_page.plan.nodes {
                match keys.push(ObjectKey::Task(node.chip.task)) {
                    Ok(()) => {}
                    Err(_) => break,
                }
            }
            names(domain, env, source, keys, out);
            for node in &domain.task_page.plan.nodes {
                let key = ObjectKey::Task(node.chip.task);
                let stored = domain.objects.put(key, Body::Chip(node.chip.clone()));
                if !stored {
                    break;
                }
            }
            domain.changed();
        }
        Delivery::Chips(chips) => {
            let mut keys = List::with_capacity(env.limits.rows);
            for chip in &chips {
                match keys.push(ObjectKey::Task(chip.task)) {
                    Ok(()) => {}
                    Err(_) => break,
                }
            }
            names(domain, env, source, keys, out);
            for chip in chips.into_boxed() {
                let key = ObjectKey::Task(chip.task);
                let stored = domain.objects.put(key, Body::Chip(chip));
                if stored {
                    domain.changed();
                }
            }
        }
        Delivery::Inbox(inbox) => {
            domain.inbox.needs = inbox.needs;
            domain.inbox.updates = inbox.updates;
            domain.inbox.marked_results = domain.inbox.marked_results.max(inbox.results_seen);
            domain.inbox.more = inbox.more;
            if domain.inbox.before.is_none() {
                domain.inbox.replace(inbox.entries);
                let mut keys = List::with_capacity(env.limits.rows);
                for entry in &domain.inbox.rows {
                    if keys.push(entry.object).is_err() {
                        break;
                    }
                }
                names(domain, env, source, keys, out);
                crate::pages::conversations::mark_results(domain, out);
            }
            domain.timed_page(!domain.inbox.rows.is_empty());
            domain.changed();
        }
        Delivery::Goals(goals) => {
            let mut keys = List::with_capacity(env.limits.rows);
            for row in &goals.rows {
                match keys.push(ObjectKey::Task(row.chip.task)) {
                    Ok(()) => {}
                    Err(_) => break,
                }
            }
            names(domain, env, source, keys, out);
            for row in goals.rows.into_boxed() {
                let key = ObjectKey::Task(row.chip.task);
                let stored = domain.objects.put(key, Body::Chip(row.chip));
                if stored {
                    domain.changed();
                }
            }
        }
        Delivery::Conversations(conversations) => {
            domain.chats.more = conversations.more;
            if domain.conversation_rows.edges.reaches(End::Before) {
                domain.conversation_rows.rows.clear();
                let mut rows = conversations.rows;
                filter_chats(domain.address, &mut rows);
                crate::pages::conversations::order(&mut rows);
                let dropped = domain.conversation_rows.slide(rows, blank_edges(), End::Before);
                if dropped > 0 {
                    refuse(domain, Limit::Objects);
                }
            }
            domain.changed();
        }
        Delivery::Deployment(deployment) => parts(domain, env, deployment.parts.as_slice(), out),
    }
}
fn read_body(domain: &mut Domain, key: ObjectKey, body: Body) {
    // Reading does not name objects: a source may have stopped naming this
    // object while its read was in flight in the same generation.
    if domain.objects.put(key, body) {
        domain.changed();
    }
}
/// Consumes one read terminal and drops old-generation values before applying them.
pub(crate) fn read_result(
    domain: &mut Domain,
    env: &Env<crate::Limits>,
    token: Token,
    result: ReadResult,
    out: &mut Queue<Request>,
) {
    let Some(receipt) = domain.reads.terminal(token) else {
        return;
    };
    if receipt.abandoned || receipt.generation != domain.generation {
        return;
    }
    match &receipt.what {
        Read::History { subtree, .. } => {
            if *subtree != domain.task_page.subtree_history {
                return;
            }
        }
        Read::Ended { task, .. } => {
            if Some(*task) != domain.task_page.ended_task {
                return;
            }
        }
        Read::Frame
        | Read::Task { .. }
        | Read::Proposal { .. }
        | Read::Escalation { .. }
        | Read::Question { .. }
        | Read::PersonTask { .. }
        | Read::Result { .. }
        | Read::Activity { .. }
        | Read::Conversations { .. }
        | Read::Inbox { .. }
        | Read::InboxGroup { .. }
        | Read::Goals { .. }
        | Read::Notes { .. }
        | Read::Project { .. }
        | Read::Deployment => {}
    }
    match receipt.source {
        Source::Window { number: 10, .. } => {
            if domain.inbox.reading != Some(token) {
                return;
            }
            domain.inbox.reading = None;
        }
        Source::Window { .. } | Source::Watch(_) | Source::Card(_) => {}
    }
    match result {
        ReadResult::None(why) => {
            match receipt.source {
                Source::Window { number: 1, .. } => domain.conversation.failed(),
                Source::Window { .. } | Source::Watch(_) | Source::Card(_) => {}
            }
            match why {
                contract::ReadNone::Unknown | contract::ReadNone::Gone | contract::ReadNone::Refused => {
                    domain.page = Page::Missing;
                    domain.changed();
                }
                contract::ReadNone::Busy | contract::ReadNone::Unreachable => {
                    refuse(domain, Limit::Reads);
                }
                contract::ReadNone::SignedOut => crate::domain::signed_out(domain, out),
            }
        }
        ReadResult::Value(value) => read_value(domain, env, receipt.source, value, out),
    }
}
fn read_value(
    domain: &mut Domain,
    env: &Env<crate::Limits>,
    source: Source,
    value: ReadValue,
    out: &mut Queue<Request>,
) {
    match value {
        ReadValue::Frame(_) | ReadValue::Notes(_) => {}
        ReadValue::Task(task) => task_read(domain, env, task, out),
        ReadValue::Proposal(proposal) => {
            let key = ObjectKey::Proposal(proposal.key);
            read_body(domain, key, Body::Proposal(proposal));
        }
        ReadValue::Escalation(escalation) => {
            let key = ObjectKey::Escalation(escalation.task.task);
            read_body(domain, key, Body::Escalation(escalation));
        }
        ReadValue::Question(question) => {
            let key = ObjectKey::Question { task: question.task.task, message: question.message };
            read_body(domain, key, Body::Question(question));
        }
        ReadValue::PersonTask(task) => {
            let key = ObjectKey::PersonTask(task.task.task);
            read_body(domain, key, Body::PersonTask(task));
        }
        ReadValue::Result(result) => {
            let key = ObjectKey::Result(result.task.task);
            read_body(domain, key, Body::Result(result));
        }
        ReadValue::Activity(page) => crate::conversation::routing::page(domain, env, *page, out),
        ReadValue::History(page) => {
            let edges = page_edges(page.before, page.after);
            let dropped = domain.history_rows.slide(page.rows, edges, End::After);
            if dropped > 0 {
                domain.changed();
            }
            domain.changed();
        }
        ReadValue::Conversations(page) => {
            let edges = page_edges(page.before, page.after);
            let mut rows = page.rows;
            filter_chats(domain.address, &mut rows);
            crate::pages::conversations::order(&mut rows);
            let dropped = domain.conversation_rows.slide(rows, edges, End::After);
            if dropped > 0 {
                domain.changed();
            }
            crate::pages::conversations::order(&mut domain.conversation_rows.rows);
            domain.changed();
        }
        ReadValue::Inbox(page) => {
            domain.inbox.before = page.before;
            domain.inbox.after = page.after;
            domain.inbox.more = domain.inbox.after.is_some();
            domain.inbox.replace(page.rows);
            let mut keys = List::with_capacity(env.limits.rows);
            for entry in &domain.inbox.rows {
                if keys.push(entry.object).is_err() {
                    break;
                }
            }
            names(domain, env, source, keys, out);
            crate::pages::conversations::mark_results(domain, out);
            domain.changed();
        }
        ReadValue::Ended(page) => {
            crate::pages::task::ended(domain, page.before, page.after);
            let mut keys = List::with_capacity(env.limits.rows);
            for result in &page.rows {
                match keys.push(ObjectKey::Result(result.task.task)) {
                    Ok(()) => {}
                    Err(_) => break,
                }
            }
            names(domain, env, source, keys, out);
            for result in page.rows.into_boxed() {
                let key = ObjectKey::Result(result.task.task);
                let stored = domain.objects.put(key, Body::Result(Box::new(result)));
                if stored {
                    domain.changed();
                }
            }
        }
        ReadValue::Goals(page) => {
            let mut keys = List::with_capacity(env.limits.rows);
            for goal in &page.rows {
                match keys.push(ObjectKey::Task(goal.chip.task)) {
                    Ok(()) => {}
                    Err(_) => break,
                }
            }
            names(domain, env, source, keys, out);
        }
        ReadValue::Project(project) => parts(domain, env, project.parts.as_slice(), out),
        ReadValue::Deployment(deployment) => parts(domain, env, deployment.parts.as_slice(), out),
    }
}

fn filter_chats(address: Address, rows: &mut List<contract::ConversationLine>) {
    let project = match address {
        Address::Conversations { project } => project,
        Address::Inbox { .. } | Address::Task { .. } | Address::Project { .. } | Address::Deployment => None,
    };
    let mut filtered = List::with_capacity(rows.capacity());
    let incoming = core::mem::replace(rows, List::with_capacity(0));
    for row in incoming.into_boxed() {
        if project.is_none() || project == Some(row.project) {
            filtered.push(row).expect("filter does not add rows");
        }
    }
    *rows = filtered;
}

fn task_read(domain: &mut Domain, env: &Env<crate::Limits>, task: Box<contract::Task>, out: &mut Queue<Request>) {
    crate::conversation::routing::document(domain, &task, out);
    let key = ObjectKey::Task(task.chip.task);
    read_body(domain, key, Body::Task(task));
    crate::pages::task::document(domain, env, out);
}
