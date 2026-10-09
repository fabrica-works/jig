//! Notes are handles into the shared objects. Corrections and deletions use jig
//! 21's revision-checked confirmations; moved reads update the same handle.
//! Scope and description are source filters. See client/domain/pages.md, 6.4.
use crate::{
    Domain, Request,
    contract::{Limit, ObjectKey, Read},
    objects::Source,
};
use skein_lib::{List, Queue};
pub(crate) fn filter(domain: &mut Domain, filter: super::NoteFilter, out: &mut Queue<Request>) {
    if super::number(domain).is_none() {
        return;
    }
    domain.project_page.note_writes = domain.project_page.note_writes.checked_add(1).expect("note writes remain");
    domain.project_page.note_filter = Some(filter.clone());
    domain.project_page.notes.clear();
    domain.project_page.notes_before = None;
    domain.project_page.notes_after = None;
    super::ask(domain, 3, Read::Notes { scope: filter.scope, description: filter.description, after: None }, out);
    domain.changed();
}
pub(crate) fn page(domain: &mut Domain, source: Source, page: crate::contract::NotePage) {
    let Some(project) = super::number(domain) else {
        return;
    };
    domain.project_page.notes_before = page.before;
    domain.project_page.notes_after = page.after;
    let mut keys = List::with_capacity(domain.project_page.notes.capacity());
    for note in &page.rows {
        if keys.room() == 0 {
            break;
        }
        keys.push(ObjectKey::Note { project, note: note.number }).expect("note key fits");
    }
    let missed = domain.objects.replace(source, keys.as_slice());
    if missed > 0 {
        crate::pages::routing::refuse(domain, Limit::Objects);
    }
    domain.project_page.notes.clear();
    for note in page.rows.into_boxed() {
        if domain.project_page.notes.room() == 0 {
            break;
        }
        match domain.objects.note(project, source, note) {
            Some(id) => domain.project_page.notes.push(id).expect("note handle fits"),
            None => {
                crate::pages::routing::refuse(domain, Limit::Objects);
                break;
            }
        }
    }
    domain.changed();
}
