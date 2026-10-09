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
        Page::Starting | Page::Elsewhere | Page::SigningIn | Page::Missing | Page::OutOfDate | Page::SomeoneElse => {}
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
    domain.close_page(out);
    domain.reads.abandon(domain.generation);
    out.push(Request::PartsLeft { generation: domain.generation });
    domain.objects.clear();
    domain.object_rows.rows.clear();
    domain.object_rows.edges = blank_edges();
    domain.conversation_rows.rows.clear();
    domain.conversation_rows.edges = blank_edges();
    domain.turn_rows.rows.clear();
    domain.turn_rows.edges = blank_edges();
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
}
/// Opens only what the selected work page shows; services have no page.
pub(crate) fn enter(domain: &mut Domain, env: &Env<crate::Limits>, out: &mut Queue<Request>) {
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
    domain.page =
        Page::Work { address: domain.address, generation: domain.generation, behind: false, excess: 0, refused: None };
    domain.changed();
    match domain.address {
        Address::Inbox { .. } => open(domain, env, Subject::Inbox { window: env.limits.rows }, out),
        Address::Conversations { .. } => open(domain, env, Subject::Conversations, out),
        Address::Task { task, section } => {
            open(domain, env, Subject::Task { task }, out);
            match section {
                TaskSection::Overview | TaskSection::Budget => {}
                TaskSection::Activity => {
                    open(domain, env, Subject::Run { task, turn: None, offset: 0 }, out);
                    read(domain, 1, Read::Activity { task, before: None }, out);
                }
                TaskSection::Plan => open(domain, env, Subject::Tree { task, window: env.limits.rows }, out),
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
    }
}
fn names(
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
        if domain.objects.find(*key).is_some() {
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
fn parts(domain: &mut Domain, env: &Env<crate::Limits>, values: &[Part], out: &mut Queue<Request>) {
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
        Delivery::Frame(_) | Delivery::Run(_) => {}
        Delivery::Task(task) => {
            parts(domain, env, task.parts.as_slice(), out);
            parts(domain, env, task.resources.as_slice(), out);
            let key = ObjectKey::Task(task.chip.task);
            one(domain, source, key, Body::Task(task));
        }
        Delivery::Tree(tree) => {
            let mut keys = List::with_capacity(env.limits.rows);
            for node in &tree.nodes {
                match keys.push(ObjectKey::Task(node.chip.task)) {
                    Ok(()) => {}
                    Err(_) => break,
                }
            }
            names(domain, env, source, keys, out);
            for node in tree.nodes.into_boxed() {
                let key = ObjectKey::Task(node.chip.task);
                let stored = domain.objects.put(key, Body::Chip(node.chip));
                if stored {
                    domain.changed();
                }
            }
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
            let mut keys = List::with_capacity(env.limits.rows);
            for entry in &inbox.entries {
                match keys.push(entry.object) {
                    Ok(()) => {}
                    Err(_) => break,
                }
            }
            names(domain, env, source, keys, out);
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
            // A watch replaces its bounded live rows. Historical cursors come
            // from a read page, which the conversations owner requests.
            if domain.conversation_rows.edges.reaches(End::Before) {
                domain.conversation_rows.rows.clear();
                let dropped = domain.conversation_rows.slide(conversations.rows, blank_edges(), End::Before);
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
    match result {
        ReadResult::None(why) => match why {
            contract::ReadNone::Unknown | contract::ReadNone::Gone => {
                domain.page = Page::Missing;
                domain.changed();
            }
            contract::ReadNone::Refused | contract::ReadNone::Busy | contract::ReadNone::Unreachable => {
                refuse(domain, Limit::Reads);
            }
            contract::ReadNone::SignedOut => crate::domain::signed_out(domain, out),
        },
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
        ReadValue::Task(task) => {
            parts(domain, env, task.parts.as_slice(), out);
            let key = ObjectKey::Task(task.chip.task);
            read_body(domain, key, Body::Task(task));
        }
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
        ReadValue::Activity(page) => {
            let edges = page_edges(page.before, page.after);
            let dropped = domain.turn_rows.slide(page.rows, edges, End::Before);
            if dropped > 0 {
                domain.changed();
            }
            domain.changed();
        }
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
            let dropped = domain.conversation_rows.slide(page.rows, edges, End::After);
            if dropped > 0 {
                domain.changed();
            }
            domain.changed();
        }
        ReadValue::Inbox(page) => {
            let mut keys = List::with_capacity(env.limits.rows);
            for entry in &page.rows {
                match keys.push(entry.object) {
                    Ok(()) => {}
                    Err(_) => break,
                }
            }
            names(domain, env, source, keys, out);
        }
        ReadValue::Ended(page) => {
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
