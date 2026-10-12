//! Spine routing of page generations, watches, reads, names and parts.
//! Detailed page policies are jig 22's modules. See client/domain/client.md, 5–7.
use crate::{
    Domain, Request,
    action::End,
    address::{Address, Place, TaskSection},
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
pub(crate) fn excess(domain: &mut Domain, count: u32) {
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
    domain.project_page.clear();
    domain.deployment_page.document = None;
    domain.deployment_page.owner = None;
    domain.clock.page_times = false;
    domain.inbox.clear();
    domain.inbox.opening_updates = false;
    domain.object_rows.rows.clear();
    domain.object_rows.edges = blank_edges();
    domain.conversation_rows.rows.clear();
    domain.conversation_rows.cut = false;
    domain.conversation_rows.edges = blank_edges();
    domain.conversation_rows.owner_after = None;
    let rows = domain.conversation.lines.rows.capacity();
    let words = domain.conversation.words.rows.capacity();
    domain.conversation = crate::conversation::Conversation::new(0, rows);
    domain.conversation.words = crate::conversation::KeptWords::new(words);
    domain.history_rows.rows.clear();
    domain.history_rows.cut = false;
    domain.history_rows.edges = blank_edges();
    domain.generation = generation;
    match place {
        Place::Jig(address) => {
            domain.address = address;
            match domain.frame.document {
                Some(_) if domain.versions.is_some() => enter(domain, env, out),
                Some(_) | None => {
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
    let hidden = match domain.confirmed {
        Some(party) => crate::domain::foreign_kept_requests(domain, party),
        None => true,
    };
    if hidden {
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
            super::conversations::start_inbox(domain, out);
        }
        Address::Conversations { .. } => {
            open(domain, env, Subject::Conversations, out);
            read(domain, 11, Read::Conversations { after: None }, out);
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
        Address::Project { .. } => crate::pages::project::enter(domain, env, out),
        Address::Deployment => open(domain, env, Subject::Deployment, out),
    }
}
pub(crate) fn document_read(key: ObjectKey) -> Read {
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
                Ok(()) | Err(_) => {}
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
        let scope = crate::application::scope(domain, part);
        if let Some(scope) = scope {
            out.push(Request::ShowPart { part: part.clone(), generation: domain.generation, scope });
        } else {
            refuse(domain, Limit::Reads);
        }
        count = count.saturating_add(1);
    }
}

fn one(
    domain: &mut Domain,
    env: &Env<crate::Limits>,
    source: Source,
    key: ObjectKey,
    body: Body,
    out: &mut Queue<Request>,
) {
    let missed = domain.objects.replace(source, &[key]);
    if missed > 0 {
        refuse(domain, Limit::Objects);
        return;
    }
    match domain.objects.find(key) {
        Some(_) => {
            let stored = crate::application::put(domain, env, key, body, out);
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
        Delivery::Frame(_) | Delivery::OwnerProject(_) => {}
        Delivery::Run(value) => crate::conversation::routing::stream(domain, env, watch, *value, out),
        Delivery::RunState(value) => crate::conversation::routing::run_state(domain, env, watch, *value, out),
        Delivery::OwnerProposal(proposal) => {
            let key = ObjectKey::Proposal(proposal.key);
            one(domain, env, source, key, Body::OwnerProposal(proposal), out);
        }
        Delivery::Task(task) => {
            crate::conversation::routing::document(domain, &task, out);
            crate::pages::task::delivered(domain, source, task);
            crate::pages::task::document(domain, env, out);
        }
        Delivery::OwnerTree(tree) => owner_tree_delivery(domain, env, watch, source, tree, out),
        Delivery::Tree(tree) => {
            crate::pages::task::tree(domain, *tree);
            let mut keys = List::with_capacity(crate::limits::tree_names(&env.limits).expect("tree names are bounded"));
            for node in &domain.task_page.plan.nodes {
                if !keys.as_slice().contains(&ObjectKey::Task(node.chip.task))
                    && keys.push(ObjectKey::Task(node.chip.task)).is_err()
                {
                    break;
                }
                for proposal in &node.proposals {
                    let key = ObjectKey::Proposal(*proposal);
                    if !keys.as_slice().contains(&key) && keys.push(key).is_err() {
                        break;
                    }
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
        Delivery::OwnerInbox(inbox) => {
            domain.inbox.owner(inbox);
            crate::pages::replies::inbox(domain, env, source, out);
            crate::pages::conversations::mark_results(domain, out);
            domain.timed_page(!domain.inbox.rows.is_empty());
            domain.changed();
        }
        Delivery::Inbox(inbox) => {
            domain.inbox.needs = inbox.needs;
            domain.inbox.updates = inbox.updates;
            domain.inbox.results_seen = inbox.results_seen;
            domain.inbox.marked_results = domain.inbox.marked_results.max(inbox.results_seen);
            domain.inbox.document = Some(contract::InboxDocument::Entry(inbox.clone()));
            domain.inbox.replace(inbox.entries, inbox.more);
            crate::pages::replies::inbox(domain, env, source, out);
            crate::pages::conversations::mark_results(domain, out);
            domain.timed_page(!domain.inbox.rows.is_empty());
            domain.changed();
        }
        Delivery::Goals(goals) => crate::pages::project::live(domain, env, source, *goals, out),
        Delivery::Tasks(tasks) => crate::pages::project::task_rows(domain, env, source, *tasks, out),
        Delivery::Conversations(conversations) => {
            super::conversations::delivered(domain, env, source, *conversations, out);
        }
        Delivery::Deployment(deployment) => crate::pages::deployment::document(domain, env, deployment, out),
        Delivery::OwnerDeployment(deployment) => crate::pages::deployment::owner_document(domain, env, deployment, out),
    }
}
fn read_body(domain: &mut Domain, key: ObjectKey, body: Body) {
    // Reading does not name objects: a source may have stopped naming this
    // object while its read was in flight in the same generation.
    if domain.objects.put(key, body) {
        domain.changed();
    }
}
fn owner_proposal_read(
    domain: &mut Domain,
    env: &Env<crate::Limits>,
    proposal: Box<contract::OwnerProposal>,
    out: &mut Queue<Request>,
) {
    let key = ObjectKey::Proposal(proposal.key);
    if crate::application::put(domain, env, key, Body::OwnerProposal(proposal), out) {
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
    if !owner_escalation_read_valid(domain, token, &result)
        || !owner_project_read_valid(domain, token, &result)
        || !owner_inbox_read_valid(domain, env, token, &result)
        || !historical_read_valid(domain, env, token, &result)
    {
        refuse(domain, Limit::Reads);
        return;
    }
    let Some(receipt) = domain.reads.terminal(token) else {
        return;
    };
    if receipt.abandoned || receipt.generation != domain.generation {
        return;
    }
    match receipt.source {
        Source::Window { number: crate::application::WINDOW, .. } => {
            crate::application::read_result(domain, env, receipt.what, result, out);
            return;
        }
        Source::Window { .. } | Source::Watch(_) | Source::Card(_) => {}
    }
    let inbox_read = match &receipt.what {
        Read::History { subtree, .. } => {
            if *subtree != domain.task_page.subtree_history {
                return;
            }
            false
        }
        Read::Ended { task, .. } => {
            if Some(*task) != domain.task_page.ended_task {
                return;
            }
            false
        }
        Read::Notes { scope, description, .. } => {
            let valid = match &domain.project_page.note_filter {
                Some(filter) => filter.scope == *scope && filter.description == *description,
                None => false,
            };
            if !valid {
                return;
            }
            false
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
        | Read::Goals { .. }
        | Read::Spend { .. }
        | Read::Project { .. }
        | Read::Deployment => false,
        Read::Inbox { .. } => true,
    };
    match receipt.source {
        Source::Window { number, .. } if inbox_read => {
            let Some(group) = super::conversations::inbox_group(number) else {
                return;
            };
            let window = domain.inbox.window_mut(group);
            if window.reading != Some(token) {
                return;
            }
            window.reading = None;
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
        ReadResult::Value(value) => received_value(domain, env, receipt.source, receipt.what, value, out),
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
        ReadValue::Frame(_) | ReadValue::OwnerInbox(_) | ReadValue::OwnerEnded(_) | ReadValue::OwnerSpend(_) => {}
        ReadValue::OwnerProject(value) => crate::pages::project::owner_document(domain, value),
        ReadValue::Notes(page) => crate::pages::project::notes::page(domain, source, *page),
        ReadValue::Spend(page) => crate::pages::project::spend(domain, *page),
        ReadValue::Task(task) => task_read(domain, env, task, out),
        ReadValue::Historical(value) => read_body(domain, value.key, Body::Historical(value)),
        ReadValue::OwnerProposal(proposal) => owner_proposal_read(domain, env, proposal, out),
        ReadValue::Proposal(proposal) => {
            let key = ObjectKey::Proposal(proposal.key);
            read_body(domain, key, Body::Proposal(proposal));
        }
        ReadValue::OwnerEscalation(value) => owner_escalation_read(domain, value),
        ReadValue::Escalation(escalation) => {
            let key = ObjectKey::Escalation(escalation.task.task);
            read_body(domain, key, Body::Escalation(escalation));
        }
        ReadValue::Question(question) => {
            let key = ObjectKey::Question { task: question.task.task, message: question.message };
            read_body(domain, key, Body::Question(question));
        }
        ReadValue::OwnerPersonTask(task) => {
            let key = ObjectKey::PersonTask(task.task.task);
            read_body(domain, key, Body::OwnerPersonTask(task));
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
            domain.history_rows.cut |= page.cut;
            let edges = page_edges(page.before, page.after);
            let dropped = domain.history_rows.slide(page.rows, edges, End::After);
            if dropped > 0 {
                domain.changed();
            }
            domain.changed();
        }
        ReadValue::Conversations(page) => {
            domain.conversation_rows.cut |= page.cut;
            let edges = page_edges(page.before, page.after);
            let mut rows = page.rows;
            crate::pages::conversations::order(&mut rows);
            let end = super::conversations::read_end(source);
            let dropped = if super::conversations::is_replay(source) {
                domain.conversation_rows.replay(rows, edges)
            } else {
                domain.conversation_rows.slide(rows, edges, end)
            };
            if dropped > 0 {
                domain.changed();
            }
            crate::pages::conversations::order(&mut domain.conversation_rows.rows);
            crate::pages::replies::conversations(domain, env, source, out);
            domain.changed();
        }
        ReadValue::Inbox(page) => {
            let Source::Window { number, .. } = source else {
                return;
            };
            let Some(group) = super::conversations::inbox_group(number) else {
                return;
            };
            domain.inbox.page(group, *page);
            crate::pages::replies::inbox(domain, env, source, out);
            crate::pages::conversations::mark_results(domain, out);
            domain.changed();
        }
        ReadValue::Ended(page) => {
            crate::pages::task::ended(domain, page.before, page.after);
            domain.task_page.ended_cut = page.cut;
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
        ReadValue::Goals(page) => crate::pages::project::ended(domain, env, source, *page, out),
        ReadValue::Project(project) => crate::pages::project::document(domain, env, project, out),
        ReadValue::Deployment(deployment) => crate::pages::deployment::document(domain, env, deployment, out),
        ReadValue::OwnerDeployment(document) => crate::pages::deployment::owner_document(domain, env, document, out),
    }
}

fn owner_escalation_read(domain: &mut Domain, value: Box<contract::OwnerEscalation>) {
    let key = ObjectKey::Escalation(value.task.chip.task);
    read_body(domain, key, Body::OwnerEscalation(value));
}

fn task_read(domain: &mut Domain, env: &Env<crate::Limits>, task: Box<contract::Task>, out: &mut Queue<Request>) {
    crate::conversation::routing::document(domain, &task, out);
    let key = ObjectKey::Task(task.chip.task);
    read_body(domain, key, Body::Task(task));
    crate::pages::task::document(domain, env, out);
}

// The new carrier must match its original read before retiring the obligation.
fn owner_project_read_valid(domain: &Domain, token: Token, result: &ReadResult) -> bool {
    let value = match result {
        ReadResult::Value(value) => match value {
            ReadValue::OwnerProject(value) => value,
            ReadValue::Frame(_)
            | ReadValue::Historical(_)
            | ReadValue::Task(_)
            | ReadValue::Proposal(_)
            | ReadValue::OwnerProposal(_)
            | ReadValue::Escalation(_)
            | ReadValue::OwnerEscalation(_)
            | ReadValue::Question(_)
            | ReadValue::PersonTask(_)
            | ReadValue::OwnerPersonTask(_)
            | ReadValue::Result(_)
            | ReadValue::OwnerEnded(_)
            | ReadValue::OwnerSpend(_)
            | ReadValue::Ended(_)
            | ReadValue::History(_)
            | ReadValue::Activity(_)
            | ReadValue::Inbox(_)
            | ReadValue::OwnerInbox(_)
            | ReadValue::Conversations(_)
            | ReadValue::Goals(_)
            | ReadValue::Spend(_)
            | ReadValue::Project(_)
            | ReadValue::Notes(_)
            | ReadValue::Deployment(_)
            | ReadValue::OwnerDeployment(_) => return true,
        },
        ReadResult::None(_) => return true,
    };
    let Some(read) = domain.reads.slab.get(Id::from_token(token)) else {
        return true;
    };
    if read.abandoned || read.terminal || read.generation != domain.generation {
        return true;
    }
    match &read.what {
        Read::Project { project } => *project == value.number,
        Read::Frame
        | Read::Task { .. }
        | Read::Proposal { .. }
        | Read::Escalation { .. }
        | Read::Question { .. }
        | Read::PersonTask { .. }
        | Read::Result { .. }
        | Read::Ended { .. }
        | Read::History { .. }
        | Read::Activity { .. }
        | Read::Inbox { .. }
        | Read::Conversations { .. }
        | Read::Goals { .. }
        | Read::Spend { .. }
        | Read::Notes { .. }
        | Read::Deployment => false,
    }
}

// Validate the original query and entire group before consuming its terminal.
fn owner_inbox_read_valid(domain: &Domain, env: &Env<crate::Limits>, token: Token, result: &ReadResult) -> bool {
    let page = match result {
        ReadResult::Value(value) => match value {
            ReadValue::OwnerInbox(page) => page,
            ReadValue::Frame(_)
            | ReadValue::Task(_)
            | ReadValue::Proposal(_)
            | ReadValue::Historical(_)
            | ReadValue::OwnerProposal(_)
            | ReadValue::Escalation(_)
            | ReadValue::OwnerEscalation(_)
            | ReadValue::Question(_)
            | ReadValue::PersonTask(_)
            | ReadValue::OwnerPersonTask(_)
            | ReadValue::Result(_)
            | ReadValue::Ended(_)
            | ReadValue::History(_)
            | ReadValue::Activity(_)
            | ReadValue::Inbox(_)
            | ReadValue::Conversations(_)
            | ReadValue::Goals(_)
            | ReadValue::Spend(_)
            | ReadValue::OwnerEnded(_)
            | ReadValue::OwnerSpend(_)
            | ReadValue::OwnerProject(_)
            | ReadValue::Project(_)
            | ReadValue::Notes(_)
            | ReadValue::Deployment(_)
            | ReadValue::OwnerDeployment(_) => return true,
        },
        ReadResult::None(_) => return true,
    };
    let Some(read) = domain.reads.slab.get(Id::from_token(token)) else {
        return true;
    };
    if read.abandoned || read.terminal || read.generation != domain.generation {
        return true;
    }
    let group = match &read.what {
        Read::Inbox { group, .. } => *group,
        Read::Frame
        | Read::Task { .. }
        | Read::Proposal { .. }
        | Read::Escalation { .. }
        | Read::Question { .. }
        | Read::PersonTask { .. }
        | Read::Result { .. }
        | Read::Ended { .. }
        | Read::History { .. }
        | Read::Activity { .. }
        | Read::Conversations { .. }
        | Read::Goals { .. }
        | Read::Spend { .. }
        | Read::Project { .. }
        | Read::Notes { .. }
        | Read::Deployment => return false,
    };
    let number = match read.source {
        Source::Window { number, .. } => number,
        Source::Watch(_) | Source::Card(_) => return false,
    };
    // A superseded terminal retires its own old obligation; the routing receipt
    // will discard it without consuming or replacing the current query/window.
    if domain.inbox.window(group).reading != Some(token) {
        return true;
    }
    if super::conversations::inbox_group(number) != Some(group)
        || super::inbox::query_filter(Some(&read.what)) != Some(domain.inbox.filter)
        || page.rows.capacity() > env.limits.rows
    {
        return false;
    }
    for row in &page.rows {
        if contract::InboxReadGroup::of(row.group()) != group {
            return false;
        }
    }
    true
}

fn received_value(
    domain: &mut Domain,
    env: &Env<crate::Limits>,
    source: Source,
    query: Read,
    value: ReadValue,
    out: &mut Queue<Request>,
) {
    match value {
        ReadValue::OwnerEnded(page) => super::task::owner_ended(domain, page),
        ReadValue::OwnerSpend(page) => super::project::owner_spend(domain, page),
        ReadValue::OwnerProject(value) => match query {
            Read::Project { project } => {
                if project == value.number {
                    crate::pages::project::owner_document(domain, value);
                }
            }
            Read::Frame
            | Read::Task { .. }
            | Read::Proposal { .. }
            | Read::Escalation { .. }
            | Read::Question { .. }
            | Read::PersonTask { .. }
            | Read::Result { .. }
            | Read::Ended { .. }
            | Read::History { .. }
            | Read::Activity { .. }
            | Read::Inbox { .. }
            | Read::Conversations { .. }
            | Read::Goals { .. }
            | Read::Spend { .. }
            | Read::Notes { .. }
            | Read::Deployment => {}
        },
        ReadValue::OwnerDeployment(page) => {
            if query == Read::Deployment {
                crate::pages::deployment::owner_document(domain, env, page, out);
            }
        }
        ReadValue::OwnerInbox(page) => {
            let group = match &query {
                Read::Inbox { group, .. } => *group,
                Read::Frame
                | Read::Task { .. }
                | Read::Proposal { .. }
                | Read::Escalation { .. }
                | Read::Question { .. }
                | Read::PersonTask { .. }
                | Read::Result { .. }
                | Read::Ended { .. }
                | Read::History { .. }
                | Read::Activity { .. }
                | Read::Conversations { .. }
                | Read::Goals { .. }
                | Read::Spend { .. }
                | Read::Project { .. }
                | Read::Notes { .. }
                | Read::Deployment => return,
            };
            domain.inbox.owner_page(group, query, *page);
            crate::pages::replies::inbox(domain, env, source, out);
            crate::pages::conversations::mark_results(domain, out);
            domain.changed();
        }
        ReadValue::Frame(_)
        | ReadValue::Task(_)
        | ReadValue::Proposal(_)
        | ReadValue::Historical(_)
        | ReadValue::OwnerProposal(_)
        | ReadValue::Escalation(_)
        | ReadValue::OwnerEscalation(_)
        | ReadValue::Question(_)
        | ReadValue::PersonTask(_)
        | ReadValue::OwnerPersonTask(_)
        | ReadValue::Result(_)
        | ReadValue::Ended(_)
        | ReadValue::History(_)
        | ReadValue::Activity(_)
        | ReadValue::Inbox(_)
        | ReadValue::Conversations(_)
        | ReadValue::Goals(_)
        | ReadValue::Spend(_)
        | ReadValue::Project(_)
        | ReadValue::Notes(_)
        | ReadValue::Deployment(_) => read_value(domain, env, source, value, out),
    }
}

fn owner_tree_delivery(
    domain: &mut Domain,
    env: &Env<crate::Limits>,
    watch: Id<Watch>,
    source: Source,
    tree: Box<contract::OwnerTree>,
    out: &mut Queue<Request>,
) {
    let Some(watch) = domain.watches.get(watch) else {
        return;
    };
    match &watch.subject {
        Subject::Tree { task, .. } => {
            if *task != tree.root.task {
                return;
            }
        }
        Subject::Frame
        | Subject::Task { .. }
        | Subject::Chips { .. }
        | Subject::Inbox { .. }
        | Subject::Goals { .. }
        | Subject::Tasks { .. }
        | Subject::Conversations
        | Subject::Run { .. }
        | Subject::Deployment => return,
    }
    crate::pages::task::owner_tree(domain, tree);
    let owner = domain.task_page.plan.owner.as_ref().expect("whole tree retained");
    let mut keys = List::with_capacity(crate::limits::tree_names(&env.limits).expect("tree names bounded"));
    keys.push(ObjectKey::Task(owner.root.task)).expect("root fits");
    for row in &owner.rows {
        let key = ObjectKey::Task(row.chip.task);
        if !keys.as_slice().contains(&key) {
            keys.push(key).expect("tree row names fit");
        }
        for proposal in &row.proposals {
            let key = ObjectKey::Proposal(*proposal);
            if !keys.as_slice().contains(&key) {
                keys.push(key).expect("tree proposal names fit");
            }
        }
    }
    names(domain, env, source, keys, out);
    let owner = domain.task_page.plan.owner.as_ref().expect("whole tree retained");
    let root = ObjectKey::Task(owner.root.task);
    let mut put_root = true;
    if let Some(id) = domain.objects.find(root)
        && let Some(body) = &domain.objects.get(id).expect("named root exists").body
    {
        match body {
            Body::Task(_) => put_root = false,
            Body::Historical(_)
            | Body::Chip(_)
            | Body::Proposal(_)
            | Body::OwnerProposal(_)
            | Body::Escalation(_)
            | Body::OwnerEscalation(_)
            | Body::Question(_)
            | Body::PersonTask(_)
            | Body::OwnerPersonTask(_)
            | Body::Result(_)
            | Body::Note { .. } => {}
        }
    }
    if put_root {
        let _stored = domain.objects.put(root, Body::Chip(owner.root.clone()));
    }
    for row in &owner.rows {
        let _stored = domain.objects.put(ObjectKey::Task(row.chip.task), Body::Chip(row.chip.clone()));
    }
    domain.changed();
}

fn historical_funders_fit(page: &contract::OwnerSpendPage, project: u32) -> bool {
    for row in &page.rows {
        match row.funder {
            contract::Funder::Period { project: owner, .. } => {
                if owner != project {
                    return false;
                }
            }
            contract::Funder::Pool { .. } | contract::Funder::Task(_) | contract::Funder::Recurring { .. } => {
                return false;
            }
        }
    }
    true
}

/// Validate the retained original request before consuming its terminal.
fn historical_read_valid(domain: &Domain, env: &Env<crate::Limits>, token: Token, result: &ReadResult) -> bool {
    let value = match result {
        ReadResult::Value(value) => value,
        ReadResult::None(_) => return true,
    };
    let Some(read) = domain.reads.slab.get(Id::from_token(token)) else {
        return true;
    };
    if read.abandoned || read.terminal || read.generation != domain.generation {
        return true;
    }
    match value {
        ReadValue::OwnerEnded(page) => {
            let task = match &read.what {
                Read::Ended { task, .. } => *task,
                Read::Frame
                | Read::Task { .. }
                | Read::Proposal { .. }
                | Read::Escalation { .. }
                | Read::Question { .. }
                | Read::PersonTask { .. }
                | Read::Result { .. }
                | Read::History { .. }
                | Read::Activity { .. }
                | Read::Inbox { .. }
                | Read::Conversations { .. }
                | Read::Goals { .. }
                | Read::Spend { .. }
                | Read::Project { .. }
                | Read::Notes { .. }
                | Read::Deployment => return false,
            };
            let source = match read.source {
                Source::Window { number: 16, .. } => true,
                Source::Window { .. } | Source::Watch(_) | Source::Card(_) => false,
            };
            source
                && domain.task_page.ended_task == Some(task)
                && page.rows.capacity() <= env.limits.rows
                && historical_edges_fit(page.before.as_ref(), page.after.as_ref(), env.limits.text)
        }
        ReadValue::OwnerSpend(page) => {
            let project = match &read.what {
                Read::Spend { project, .. } => *project,
                Read::Frame
                | Read::Task { .. }
                | Read::Proposal { .. }
                | Read::Escalation { .. }
                | Read::Question { .. }
                | Read::PersonTask { .. }
                | Read::Result { .. }
                | Read::Ended { .. }
                | Read::History { .. }
                | Read::Activity { .. }
                | Read::Inbox { .. }
                | Read::Conversations { .. }
                | Read::Goals { .. }
                | Read::Project { .. }
                | Read::Notes { .. }
                | Read::Deployment => return false,
            };
            let source = match read.source {
                Source::Window { number: 23, .. } => true,
                Source::Window { .. } | Source::Watch(_) | Source::Card(_) => false,
            };
            if !source
                || super::project::number(domain) != Some(project)
                || page.rows.capacity() > env.limits.rows
                || !historical_edges_fit(page.before.as_ref(), page.after.as_ref(), env.limits.text)
            {
                return false;
            }
            historical_funders_fit(page, project)
        }
        ReadValue::Frame(_)
        | ReadValue::Task(_)
        | ReadValue::Proposal(_)
        | ReadValue::Historical(_)
        | ReadValue::OwnerProposal(_)
        | ReadValue::Escalation(_)
        | ReadValue::OwnerEscalation(_)
        | ReadValue::Question(_)
        | ReadValue::PersonTask(_)
        | ReadValue::OwnerPersonTask(_)
        | ReadValue::Result(_)
        | ReadValue::Ended(_)
        | ReadValue::History(_)
        | ReadValue::Activity(_)
        | ReadValue::Inbox(_)
        | ReadValue::OwnerInbox(_)
        | ReadValue::Conversations(_)
        | ReadValue::Goals(_)
        | ReadValue::Spend(_)
        | ReadValue::OwnerProject(_)
        | ReadValue::Project(_)
        | ReadValue::Notes(_)
        | ReadValue::OwnerDeployment(_)
        | ReadValue::Deployment(_) => true,
    }
}
fn historical_edges_fit(before: Option<&contract::Cursor>, after: Option<&contract::Cursor>, text: u32) -> bool {
    if let Some(cursor) = before
        && !cursor.value.fits(text)
    {
        return false;
    }
    if let Some(cursor) = after
        && !cursor.value.fits(text)
    {
        return false;
    }
    true
}

// Check the original read and supplied scope before consuming its one terminal.
fn owner_escalation_read_valid(domain: &Domain, token: Token, result: &ReadResult) -> bool {
    let document = match result {
        ReadResult::Value(ReadValue::OwnerEscalation(document)) => document,
        ReadResult::Value(
            ReadValue::Frame(_)
            | ReadValue::Task(_)
            | ReadValue::Proposal(_)
            | ReadValue::Historical(_)
            | ReadValue::OwnerProposal(_)
            | ReadValue::Escalation(_)
            | ReadValue::OwnerProject(_)
            | ReadValue::Question(_)
            | ReadValue::PersonTask(_)
            | ReadValue::OwnerPersonTask(_)
            | ReadValue::Result(_)
            | ReadValue::Ended(_)
            | ReadValue::OwnerEnded(_)
            | ReadValue::History(_)
            | ReadValue::Activity(_)
            | ReadValue::Inbox(_)
            | ReadValue::OwnerInbox(_)
            | ReadValue::Conversations(_)
            | ReadValue::Goals(_)
            | ReadValue::Spend(_)
            | ReadValue::OwnerSpend(_)
            | ReadValue::Project(_)
            | ReadValue::Notes(_)
            | ReadValue::Deployment(_)
            | ReadValue::OwnerDeployment(_),
        )
        | ReadResult::None(_) => return true,
    };
    let Some(read) = domain.reads.slab.get(Id::from_token(token)) else {
        return true;
    };
    if read.abandoned || read.terminal || read.generation != domain.generation {
        return true;
    }
    let task = match read.what {
        Read::Escalation { task } => task,
        Read::Frame
        | Read::Task { .. }
        | Read::Proposal { .. }
        | Read::Question { .. }
        | Read::PersonTask { .. }
        | Read::Result { .. }
        | Read::Ended { .. }
        | Read::History { .. }
        | Read::Activity { .. }
        | Read::Inbox { .. }
        | Read::Conversations { .. }
        | Read::Goals { .. }
        | Read::Spend { .. }
        | Read::Project { .. }
        | Read::Notes { .. }
        | Read::Deployment => return false,
    };
    if task != document.task.chip.task {
        return false;
    }
    let Some(id) = domain.objects.find(ObjectKey::Escalation(task)) else {
        return true;
    };
    match domain.objects.get(id) {
        Some(object) => match object.project {
            Some(project) => project == document.task.project,
            None => true,
        },
        None => true,
    }
}
