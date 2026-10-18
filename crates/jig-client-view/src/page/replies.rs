//! Reply cards use whole committed conversation lines, including inbox-only
//! receipts. The displayed window owns its names across watch and read slides;
//! no reply position, title or first words are invented. See pages.md, 3 and 4,
//! and objects.md, 8. Task documents and card policy stay in the shared objects.
use crate::{
    Request, Ui,
    objects::{Context, Source},
};
use jig_client::contract::{ObjectKey, Subject};
use skein_lib::{Env, List, Queue};

/// A page watch owns the displayed window even when a read slides its rows.
/// This avoids retaining dropped rows under an obsolete, second source.
fn source(domain: &Ui, fallback: Source, conversations: bool, lent: &jig_client::Client) -> Source {
    for (id, watch) in &domain.handles.watches {
        if watch.owner != (crate::page::shown::owner(domain)) {
            continue;
        }
        let matches = match watch.subject {
            Subject::Inbox { .. } => !conversations,
            Subject::Tasks { query, .. } => match domain.frame(lent).document {
                Some(me) => conversations && crate::queries::chats(me, query.project) == Some(query),
                None => false,
            },
            Subject::Me
            | Subject::Task { .. }
            | Subject::Tree { .. }
            | Subject::Chips { .. }
            | Subject::Run { .. }
            | Subject::Deployment => false,
        };
        if matches {
            return Source::Watch(*id);
        }
    }
    fallback
}

pub(crate) fn inbox(
    domain: &mut Ui,
    env: &Env<crate::Limits>,
    incoming: Source,
    out: &mut Queue<Request>,
    lent: &jig_client::Client,
) {
    let source = source(domain, incoming, false, lent);
    let projected = domain.borrow(lent).inbox();
    let mut keys = List::with_capacity(projected.rows.capacity());
    let mut contexts = List::with_capacity(projected.rows.capacity());
    for entry in &projected.rows {
        if keys.push(entry.card_key()).is_err() {
            break;
        }
        if let Some(project) = entry.project() {
            contexts.push(Context { object: entry.card_key(), project }).expect("context fits its row");
        }
    }
    super::routing::names(domain, env, source, keys, out, lent);
    crate::objects::apply_contexts(domain, contexts);
}

pub(crate) fn conversations(
    domain: &mut Ui,
    env: &Env<crate::Limits>,
    incoming: Source,
    out: &mut Queue<Request>,
    lent: &jig_client::Client,
) {
    let source = source(domain, incoming, true, lent);
    let projected = domain.borrow(lent).conversation_rows();
    let mut keys = List::with_capacity(env.limits.rows);
    let mut contexts = List::with_capacity(env.limits.rows);
    for line in &projected.rows {
        let object = ObjectKey::Task(line.chip.task);
        if keys.push(object).is_err() {
            break;
        }
        contexts.push(Context { object, project: line.project }).expect("context fits its row");
    }
    super::routing::names(domain, env, source, keys, out, lent);
    crate::objects::apply_contexts(domain, contexts);
}
