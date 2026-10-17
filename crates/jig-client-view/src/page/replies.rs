//! Reply cards use whole committed conversation lines, including inbox-only
//! receipts. The displayed window owns its names across watch and read slides;
//! no reply position, title or first words are invented. See pages.md, 3 and 4,
//! and objects.md, 8. Task documents and card policy stay in the shared objects.
use crate::{
    Domain, Request,
    objects::{Body, Context, Source},
};
use jig_client::contract::{ConversationLine, InboxGroup, Limit, ObjectKey, Subject};
use skein_lib::{Env, List, Queue};

/// A page watch owns the displayed window even when a read slides its rows.
/// This avoids retaining dropped rows under an obsolete, second source.
fn source(domain: &Domain, fallback: Source, conversations: bool) -> Source {
    for (id, ()) in &domain.watch_ids {
        let watch = domain.watches.get(*id).expect("registered watch exists");
        if watch.owner != (crate::page::shown::owner(domain)) {
            continue;
        }
        let matches = match watch.subject {
            Subject::Inbox { .. } => !conversations,
            Subject::Conversations => conversations,
            Subject::Frame
            | Subject::Task { .. }
            | Subject::Tree { .. }
            | Subject::Chips { .. }
            | Subject::Goals { .. }
            | Subject::Tasks { .. }
            | Subject::Run { .. }
            | Subject::Deployment => false,
        };
        if matches {
            return Source::Watch(*id);
        }
    }
    fallback
}

pub(crate) fn inbox(domain: &mut Domain, env: &Env<crate::Limits>, incoming: Source, out: &mut Queue<Request>) {
    let source = source(domain, incoming, false);
    let mut keys = List::with_capacity(domain.inbox.rows.capacity());
    let mut contexts = List::with_capacity(domain.inbox.rows.capacity());
    for entry in &domain.inbox.rows {
        if keys.push(entry.card_key()).is_err() {
            break;
        }
        if let Some(project) = entry.project() {
            contexts.push(Context { object: entry.card_key(), project }).expect("context fits its row");
        }
    }
    super::routing::names(domain, env, source, keys, out);
    crate::objects::apply_contexts(domain, contexts);
    for index in 0..domain.inbox.rows.len() {
        let entry = domain.inbox.rows.get(index).expect("bounded inbox row exists");
        let Some(line) = entry.reply() else {
            continue;
        };
        if entry.group() != InboxGroup::Replies
            || entry.card_key() != ObjectKey::Task(line.chip.task)
            || entry.project() != Some(line.project)
        {
            super::routing::refuse(domain, Limit::Objects);
            continue;
        }
        attach(domain, source, line.clone());
    }
}

pub(crate) fn conversations(domain: &mut Domain, env: &Env<crate::Limits>, incoming: Source, out: &mut Queue<Request>) {
    let source = source(domain, incoming, true);
    let mut keys = List::with_capacity(env.limits.rows);
    let mut contexts = List::with_capacity(env.limits.rows);
    for line in &domain.conversation_rows.rows {
        let object = ObjectKey::Task(line.chip.task);
        if keys.push(object).is_err() {
            break;
        }
        contexts.push(Context { object, project: line.project }).expect("context fits its row");
    }
    super::routing::names(domain, env, source, keys, out);
    crate::objects::apply_contexts(domain, contexts);
    for index in 0..domain.conversation_rows.rows.len() {
        let line = domain.conversation_rows.rows.get(index).expect("bounded chat row exists").clone();
        attach(domain, source, line);
    }
}

fn attach(domain: &mut Domain, source: Source, line: ConversationLine) {
    let key = ObjectKey::Task(line.chip.task);
    if !domain.objects.named(key, source) {
        return;
    }
    let chip = line.chip.clone();
    let unread = line.last_reply > line.seen;
    if domain.objects.reply(source, line).is_none() && unread {
        super::routing::refuse(domain, Limit::Objects);
    }
    let id = domain.objects.find(key).expect("named chat exists");
    let chip_only = match &domain.objects.get(id).expect("named chat exists").body {
        None | Some(Body::Historical(_) | Body::Chip(_)) => true,
        Some(
            Body::Task(_)
            | Body::Proposal(_)
            | Body::OwnerProposal(_)
            | Body::Escalation(_)
            | Body::OwnerEscalation(_)
            | Body::Question(_)
            | Body::PersonTask(_)
            | Body::OwnerPersonTask(_)
            | Body::Result(_)
            | Body::Note { .. },
        ) => false,
    };
    if chip_only {
        let stored = domain.objects.put(key, Body::Chip(chip));
        assert!(stored, "the named chat accepts its committed chip");
    }
}
