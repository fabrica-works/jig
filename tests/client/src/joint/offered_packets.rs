//! Opt-in original offered lists from admitted pending-card reads.
//! These ordinary observations never decide or manufacture an offer.
use super::packets::Incoming;
use crate::ClientId;
use jig_client_domain::contract as client;
use jig_client_test_domain::Event;
use jig_domain_core::{documents, reads};
use skein_lib::List;
use std::collections::BTreeMap;

/// Both lists from the same whole admitted owner value.
pub struct CardOffers {
    /// The engine's unchanged supplied list.
    pub source: List<jig_domain_core::acts::Offered>,
    /// The receiving client's unchanged supplied list.
    pub native: List<client::Offered>,
}

pub(super) type Captured = BTreeMap<(ClientId, client::ObjectKey), CardOffers>;

pub(super) fn read(captured: &mut Option<Captured>, source: &reads::Answer, incoming: &Incoming) {
    let Some(captured) = captured else { return };
    let Incoming::Callback {
        client,
        event: Event::Jig(jig_client_view::Event::Read { result: client::ReadResult::Value(native), .. }),
    } = incoming
    else {
        return;
    };
    match (source, native) {
        (reads::Answer::Document(source), native) => {
            let (key, original, received) = match (source.as_ref(), native) {
                (documents::Document::Proposal(source), client::ReadValue::OwnerProposal(native)) => (
                    client::ObjectKey::Proposal(super::wire::proposals::key(source.proposer, source.number)),
                    &source.acts,
                    &native.acts,
                ),
                (documents::Document::Escalation(source), client::ReadValue::OwnerEscalation(native)) => {
                    (client::ObjectKey::Escalation(source.task.chip.number), &source.acts, &native.acts)
                }
                (documents::Document::PersonTask(source), client::ReadValue::OwnerPersonTask(native)) => {
                    (client::ObjectKey::PersonTask(source.task.number), &source.acts, &native.acts)
                }
                (documents::Document::Question(source), client::ReadValue::Question(native)) => (
                    client::ObjectKey::Question { task: source.task.number, message: source.message },
                    &source.acts,
                    &native.acts,
                ),
                (documents::Document::Result(source), client::ReadValue::Result(native)) => {
                    (client::ObjectKey::Result(source.task.number), &source.acts, &native.acts)
                }
                _ => return,
            };
            captured.insert((*client, key), CardOffers { source: original.clone(), native: received.clone() });
        }
        (reads::Answer::Page(page), client::ReadValue::Notes(native)) => {
            for row in &page.rows {
                let reads::PageRow::Note(source) = row else { continue };
                let project = match source.scope {
                    documents::NoteLocation::Deployment => 0,
                    documents::NoteLocation::Project(project)
                    | documents::NoteLocation::Goal { project, .. }
                    | documents::NoteLocation::Resources { project, .. } => project,
                };
                let received =
                    native.rows.iter().find(|row| row.number == source.number).expect("same admitted note identity");
                captured.insert(
                    (*client, client::ObjectKey::Note { project, note: source.number }),
                    CardOffers { source: source.acts.clone(), native: received.acts.clone() },
                );
            }
        }
        _ => {}
    }
}
