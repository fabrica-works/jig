//! Connector-owned client documents (`domain/connectors.md`, 5.5).
use crate::{Domain, Limits, Request};
use skein_lib::{Env, List, Queue, Token};

/// The client's declared role scope, translated by the root.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ClientScope {
    Project(u32),
    Deployment,
}

/// A typed subject kept by this connector under a root-minted opaque name.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ClientSubject {
    Environment { project: u32, environment: crate::Environment },
}

/// A document this connector cannot provide.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DocumentMissing {
    Unknown,
    NotAllowed,
}

/// This owner's whole current client document.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ClientDocument {
    Environment { environment: crate::Environment, fact: crate::EnvironmentFact, end: Option<crate::EnvironmentEnd> },
}

pub(crate) fn prepare(domain: &mut Domain, limits: &Limits, token: Token, subject: ClientSubject) {
    let valid = match &subject {
        ClientSubject::Environment { environment, .. } => {
            crate::domain::resource_valid(&crate::Resource::Environment(environment.clone()), limits)
        }
    };
    if valid {
        let _prepared = domain.client_subjects.insert(token, subject);
    }
}

fn make(domain: &Domain, scope: ClientScope, token: Token, _party: u64) -> Result<ClientDocument, DocumentMissing> {
    let subject = domain.client_subjects.get(&token).ok_or(DocumentMissing::Unknown)?;
    match subject {
        ClientSubject::Environment { project, environment } => {
            if scope != ClientScope::Project(*project) {
                return Err(DocumentMissing::Unknown);
            }
            let fact = match domain.environments.get(environment) {
                Some(Some(fact)) => *fact,
                Some(None) | None => return Err(DocumentMissing::Unknown),
            };
            match domain.reliance.get(&fact.created_by.task) {
                Some((owner, _)) if *owner == *project => {}
                Some(_) | None => return Err(DocumentMissing::Unknown),
            }
            Ok(ClientDocument::Environment {
                environment: environment.clone(),
                fact,
                end: domain.application.end(environment),
            })
        }
    }
}

pub(crate) fn document(
    domain: &Domain,
    watcher: Token,
    scope: ClientScope,
    subject: Token,
    party: u64,
    out: &mut Queue<Request>,
) {
    let document = make(domain, scope, subject, party);
    out.push(Request::Document { watcher, subject, document });
}

pub(crate) fn read(domain: &mut Domain, scope: ClientScope, read: Token, party: u64, out: &mut Queue<Request>) {
    let answer = make(domain, scope, read, party);
    domain.client_subjects.remove(&read);
    out.push(Request::ReadAnswered { read, answer });
}

pub(crate) fn changed(domain: &Domain, env: &Env<Limits>, environment: &crate::Environment, out: &mut Queue<Request>) {
    let mut subjects = List::with_capacity(env.limits.proposals);
    for (token, subject) in &domain.client_subjects {
        let touched = match subject {
            ClientSubject::Environment { environment: named, .. } => named == environment,
        };
        if touched {
            subjects.push(*token).expect("bounded prepared client subjects");
        }
    }
    if !subjects.is_empty() {
        out.push(Request::Marked { subjects });
    }
}

/// Allocation bound for an owned document held by the application's journal.
#[must_use]
pub fn document_worst_case(limits: &Limits) -> Option<u64> {
    u64::try_from(size_of::<ClientDocument>()).ok()?.checked_add(u64::from(limits.name_bytes).checked_mul(2)?)
}
