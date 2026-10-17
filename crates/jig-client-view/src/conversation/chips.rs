//! Calls share one chip watch for their named tasks, independently of immutable
//! object reads and the page header's task watch. See client/domain/objects.md, 10.
use crate::{Domain, Request};
use jig_client::contract::{Limit, ObjectKey, Subject};
use skein_lib::{Env, List, Queue};

/// Changes membership after closing the previous watch obligation.
/// A final unavailable subject waits until the named set changes.
pub(crate) fn sync(domain: &mut Domain, env: &Env<crate::Limits>, out: &mut Queue<Request>) {
    match domain.page {
        crate::page::Page::Work { .. } => {}
        crate::page::Page::Elsewhere => {
            if !domain.shows(crate::page::Section::Task {
                task: domain.conversation.task,
                section: crate::address::TaskSection::Activity,
            }) {
                return;
            }
        }
        crate::page::Page::Starting
        | crate::page::Page::SigningIn
        | crate::page::Page::Missing
        | crate::page::Page::OutOfDate
        | crate::page::Page::SomeoneElse => return,
    }
    let keys = domain.conversation.names(env.limits.objects);
    let mut tasks = List::with_capacity(env.limits.rows);
    for key in keys.into_boxed() {
        match key {
            ObjectKey::Task(task) => {
                if task != domain.conversation.task && tasks.push(task).is_err() {
                    crate::page::routing::refuse(domain, Limit::Objects);
                    break;
                }
            }
            ObjectKey::Proposal(_)
            | ObjectKey::Escalation(_)
            | ObjectKey::Question { .. }
            | ObjectKey::PersonTask(_)
            | ObjectKey::Result(_)
            | ObjectKey::Note { .. } => {}
        }
    }
    let mut same = tasks.len() == domain.conversation.chips.len();
    for task in &tasks {
        if !domain.conversation.chips.as_slice().contains(task) {
            same = false;
        }
    }
    if !same {
        if let Some(id) = domain.conversation.chips_watch
            && let Some(watch) = domain.watches.get_mut(id)
        {
            crate::ui::close_watch(watch, id, out);
        }
        domain.conversation.chips = tasks;
        domain.conversation.chips_pending = true;
    }
    if let Some(id) = domain.conversation.chips_watch
        && domain.watch_ids.get(&id).is_none()
    {
        domain.conversation.chips_watch = None;
    }
    if domain.conversation.chips_watch.is_some() || !domain.conversation.chips_pending {
        return;
    }
    if domain.conversation.chips.is_empty() {
        domain.conversation.chips_pending = false;
        return;
    }
    if domain.watches.is_full() {
        return;
    }
    let subject = Subject::Chips { tasks: domain.conversation.chips.clone() };
    let owner = crate::page::shown::owner(domain);
    if let Some(id) = domain.open(subject, owner, out) {
        domain.conversation.chips_watch = Some(id);
        domain.conversation.chips_pending = false;
    }
}
