//! Fixed version refusals and family admission envelopes (protocol/client.md, 9 and 11).
use crate::{Settings, boundary::Served, translate};
use alloc::boxed::Box;
use jig_client_protocol_wire::v11 as w;
use jig_domain_core::documents;
use jig_domain_people::ApplicationScope;
use skein_lib::List;
pub(crate) fn not_served(settings: &Settings, owner: w::Owner, limits: &w::Limits) -> w::NotServed {
    let mut served = List::with_capacity(settings.served.len());
    for Served { family, first, last } in &settings.served {
        let owner = match family {
            None => w::Owner::Jig,
            Some(number) => {
                w::Owner::Family(w::Family::new(limits, w::FamilyParts { number: *number }).expect("family"))
            }
        };
        served
            .push(w::Served::new(limits, w::ServedParts { owner, first: *first, last: *last }).expect("range"))
            .expect("served bound");
    }
    w::NotServed::new(limits, w::NotServedParts { owner, served }).expect("settings served bound")
}
pub(crate) fn scope(scope: &w::Scope) -> ApplicationScope {
    match scope {
        w::Scope::Project(project) => ApplicationScope::Project(project.value()),
        w::Scope::Deployment => ApplicationScope::Deployment,
        w::Scope::Own => ApplicationScope::Own,
    }
}
pub(crate) fn window(window: &w::Window) -> Option<documents::Window> {
    let after = match window.after() {
        Some(cursor) => Some(translate::cursor::decode_window(cursor.value())?),
        None => None,
    };
    Some(documents::Window { rows: window.rows(), after })
}

pub(crate) struct Opening {
    pub request: crate::Request,
    pub project: u32,
    pub rows: u32,
}
pub(crate) fn watch(
    subject: w::OwnedParts,
    watcher: skein_lib::Token,
    sign_in: u64,
    rows: u32,
    limits: &w::Limits,
) -> Option<Opening> {
    match subject.owner {
        w::Owner::Jig => {
            let value = translate::watches::decode(&subject.record, limits)?;
            let event = translate::watches::watch(watcher, sign_in, value, rows)?;
            let project = 0;
            let mut rows = rows;
            if let jig_domain_core::Event::Watch { window, .. } = &event {
                rows = window.rows;
            }
            Some(Opening { request: crate::Request::Core(Box::new(event)), project, rows })
        }
        w::Owner::Family(value) => {
            let record = w::FamilyRecord::decode(limits, &mut skein_lib::Reader::new(&subject.record)).ok()?;
            let window = window(record.window())?;
            let request = crate::Request::Application(crate::ToApplication::Watch {
                watcher,
                sign_in,
                family: value.number(),
                version: subject.version,
                scope: scope(record.scope()),
                window,
                record: record.into_parts().record,
            });
            Some(Opening { request, project: 0, rows: window.rows })
        }
    }
}
