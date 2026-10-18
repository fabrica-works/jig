//! Project goal form over the creating bounds in me; pages.md, 6.2.
use crate::JigBindings;
use crate::words::Words;
use skein_ui::Builder;

use crate::Borrowed;
use jig_client::contract::{Admission, Bounds, OwnerProject};

/// The goal renderer traverses bounded choices and recipients; both have independent row bounds.
pub(super) fn offer_fits(value: &Admission, limits: &crate::Limits) -> Option<()> {
    use jig_client::contract::{Beyond, Recipient};
    let recipient = match value {
        Admission::Bounded { within, beyond } => {
            match within {
                Bounds::Create(jig_client::contract::CreateBounds { charters, procedures, .. }) => {
                    if charters.capacity() > limits.rows || procedures.capacity() > limits.rows {
                        return None;
                    }
                }
                Bounds::Amend { .. } => {}
            }
            match beyond {
                Beyond::Proposed { to } => Some(to),
                Beyond::Refused(_) => None,
            }
        }
        Admission::Proposed { to } => Some(to),
        Admission::Allowed | Admission::Refused(_) => None,
    };
    match recipient {
        Some(Recipient::Roles { roles, .. }) if roles.capacity() > limits.rows => None,
        None | Some(Recipient::Task(_) | Recipient::Party(_) | Recipient::Role { .. } | Recipient::Roles { .. }) => {
            Some(())
        }
    }
}

pub(super) fn form(
    b: &mut Builder,
    client: &Borrowed<'_>,
    doc: &OwnerProject,
    words: &Words,
    bindings: &mut JigBindings,
) {
    super::goals::form(b, client, doc.number, words, bindings);
}
