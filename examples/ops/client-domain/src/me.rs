//! The family side watches its own `me`; the root routes only by asker.
use jig_client::{
    Client, Handle, Latest, Link, Told, Watchable,
    contract::{Delivery, Subject},
};
use jig_ops_domain_connectors::Connector;
pub(crate) struct Update {
    pub change: Change,
    pub asked: Option<jig_client::Event>,
}
pub(crate) enum Change {
    Link(Link),
    Party(Option<u64>),
    Unchanged,
}
#[must_use]
pub(crate) fn handle(connector: Connector) -> Handle {
    Handle { asker: jig_client::Asker(u32::from(jig_ops_domain_connectors::number(connector))), number: u64::MAX }
}
/// Consume only this side's link and me notifications.
pub(crate) fn told(connector: Connector, watching: &mut bool, client: &Client, told: &Told) -> Option<Update> {
    let handle = handle(connector);
    match told {
        Told::Link { link } => {
            let asked = match link {
                Link::Starting | Link::Live | Link::Offline if !*watching => {
                    *watching = true;
                    Some(jig_client::Event::Watch { handle, subject: Watchable::Jig(Subject::Me) })
                }
                Link::SignedOut | Link::OutOfDate if *watching => {
                    *watching = false;
                    Some(jig_client::Event::Unwatch { handle })
                }
                Link::Starting | Link::Live | Link::Offline | Link::SignedOut | Link::OutOfDate => None,
            };
            Some(Update { change: Change::Link(*link), asked })
        }
        Told::Changed { handle: value } if *value == handle => {
            let Some(document) = client.document(handle) else {
                return Some(Update { change: Change::Unchanged, asked: None });
            };
            let Some(Latest::Jig(Delivery::Me(frame))) = document.latest else {
                return Some(Update { change: Change::Unchanged, asked: None });
            };
            Some(Update { change: Change::Party(Some(frame.party.number)), asked: None })
        }
        Told::Ended { handle: value, .. } if *value == handle => {
            *watching = false;
            Some(Update { change: Change::Party(None), asked: None })
        }
        Told::Changed { .. }
        | Told::Ended { .. }
        | Told::Page { .. }
        | Told::Text { .. }
        | Told::Pending { .. }
        | Told::Answered { .. }
        | Told::Kept { .. }
        | Told::Dropped { .. } => None,
    }
}
