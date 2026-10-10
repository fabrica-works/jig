//! A client's observations are checked against independently submitted store rows.
use super::{Input, Peers, Store, core, infra, obs, root};
use skein_lib::Token;

pub fn submitted(peers: &mut Peers, store: &Store) {
    let pending = store.pending.back().expect("submitted transaction");
    for write in &pending.writes {
        if let jig_fake_store::Write::Save { row, .. } = write
            && matches!(
                row,
                root::Record::Observability(obs::Record::Incident(_))
                    | root::Record::Infrastructure(infra::Record::Procedure(_))
            )
        {
            peers.document_rows.push((pending.number, row.clone()));
        }
    }
}

pub fn leads_submitted(peers: &mut Peers, store: &Store) {
    let pending = store.pending.back().expect("submitted transaction");
    for write in &pending.writes {
        let jig_fake_store::Write::Save { row: root::Record::Observability(obs::Record::Incident(incident)), .. } =
            write
        else {
            continue;
        };
        let Some(lead) = incident.lead else { continue };
        let association = pending.writes.iter().find_map(|write| match write {
            jig_fake_store::Write::Save {
                row: root::Record::Observability(obs::Record::Delegation(value)), ..
            } if value.project == incident.project
                && value.kind == (obs::DelegationKind::Lead { incident: incident.number })
                && value.child == Some(lead) =>
            {
                Some(value)
            }
            jig_fake_store::Write::Save { .. } | jig_fake_store::Write::Erase(_) => None,
        });
        if let Some(association) = association {
            assert!(pending.writes.iter().any(|write| matches!(write,
                jig_fake_store::Write::Save { row: root::Record::Core(core::Record::Tasks(jig_core_tasks::Stored::Live(task))), .. }
                if task.number == lead && task.requester == jig_core_tasks::Party::Task(association.task)
            )), "lead association and actual child are submitted together");
            peers.lead_commits.push((pending.number, incident.clone(), association.clone()));
        }
    }
}

pub fn delivered(peers: &mut Peers, store: &Store, document: root::ClientDocument) {
    let commit = peers
        .document_rows
        .iter()
        .rev()
        .find_map(|(commit, row)| {
            let reflects = match (&document, row) {
                (
                    root::ClientDocument::Observability(document),
                    root::Record::Observability(obs::Record::Incident(saved)),
                ) => {
                    matches!(document.as_ref(), obs::ClientDocument::Incident { incident, .. } if incident == saved)
                }
                (
                    root::ClientDocument::Infrastructure(document),
                    root::Record::Infrastructure(infra::Record::Procedure(saved)),
                ) => {
                    matches!(document.as_ref(), infra::ClientDocument::Panel { state, .. } if state == saved)
                }
                _ => false,
            };
            reflects.then_some(*commit)
        })
        .expect("the owner's whole document reflects an independently submitted record");
    assert!(commit <= store.applied, "application document preceded its durable owner state");
    peers.documents.push((commit, document));
}

pub fn poll(peers: &mut Peers, store: &Store) -> Vec<Input<root::Event, root::Record>> {
    let Some(sign_in) = peers.sign_in else { return Vec::new() };
    let mut events = Vec::new();
    for row in store.rows.values() {
        let (watcher, subject) = match row {
            root::Record::Observability(obs::Record::Incident(incident)) if incident.number == 12 => (
                1100,
                root::ClientSubject::Observability(obs::ClientSubject::Incident {
                    project: incident.project,
                    incident: incident.number,
                }),
            ),
            root::Record::Infrastructure(infra::Record::Procedure(state))
                if matches!(state.procedure, infra::Procedure::Remediate { .. }) =>
            {
                (
                    1200 + state.task,
                    root::ClientSubject::Infrastructure(infra::ClientSubject::Panel { project: 1, task: state.task }),
                )
            }
            root::Record::Core(_) | root::Record::Observability(_) | root::Record::Infrastructure(_) => continue,
        };
        if peers.watched_documents.insert(watcher) {
            events.push(Input::Event(root::Event::WatchApplication {
                watcher: Token::new(watcher),
                sign_in,
                scope: core::documents::ApplicationScope::Project(1),
                subject,
                window: core::documents::Window { rows: 1, after: None },
            }));
        }
    }
    events
}

// Read only the outside client's delivered values, never connector state.
pub fn restarting(peers: &Peers, number: u64) -> bool {
    peers.documents.iter().any(|(_, document)| {
        matches!(document, root::ClientDocument::Infrastructure(document) if matches!(document.as_ref(), infra::ClientDocument::Panel { task, state, .. } if *task == number && state.phase == infra::ProcedurePhase::EffectOutstanding))
    })
}
