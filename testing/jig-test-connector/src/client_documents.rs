//! Connector-owned client documents (`domain/connectors.md`, 5.5).
//! The core checks roles; this owner checks its prepared typed subject against
//! the caller's supplied scope. A root mints each token without reading it.
use crate::{ApplicationObject, Domain, Limits, Request};
use skein_lib::{Env, Queue, Token};

/// Scope translated from the client's role scope, never inferred from a token.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ClientScope {
    Project(u32),
    Deployment,
}

/// The application's typed subject handed to this owner with a fresh token.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ClientSubject {
    Object { project: u32, object: u64 },
}

/// Why this owner cannot provide a document.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DocumentMissing {
    Unknown,
    NotAllowed,
}

/// A whole connector-owned document, made from current state at delivery.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ClientDocument {
    Object { object: u64, now: ApplicationObject },
}

pub(crate) fn prepare(domain: &mut Domain, token: Token, subject: ClientSubject) {
    // A full owner table leaves the token unknown; document construction gives
    // the caller one terminal and never retains an unbounded binding.
    let _prepared = domain.client_subjects.insert(token, subject);
}

fn make(domain: &Domain, scope: ClientScope, token: Token, _party: u64) -> Result<ClientDocument, DocumentMissing> {
    let subject = domain.client_subjects.get(&token).ok_or(DocumentMissing::Unknown)?;
    match subject {
        ClientSubject::Object { project, object } => {
            if scope != ClientScope::Project(*project) {
                return Err(DocumentMissing::Unknown);
            }
            let resource = domain.specification_resource(*object).ok_or(DocumentMissing::Unknown)?;
            match domain.resource_role(*project, &resource.path) {
                Some(
                    crate::ResourceRole::Owned | crate::ResourceRole::Participating | crate::ResourceRole::Context,
                ) => {}
                Some(crate::ResourceRole::Unavailable) | None => return Err(DocumentMissing::Unknown),
            }
            let now = domain.application.object(*object);
            Ok(ClientDocument::Object { object: *object, now })
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

pub(crate) fn changed(domain: &Domain, env: &Env<Limits>, object: u64, out: &mut Queue<Request>) {
    let mut subjects = skein_lib::List::with_capacity(env.limits.values);
    for (token, subject) in &domain.client_subjects {
        let touched = match subject {
            ClientSubject::Object { object: named, .. } => *named == object,
        };
        if touched {
            subjects.push(*token).expect("bounded prepared subjects");
        }
    }
    if !subjects.is_empty() {
        out.push(Request::Marked { subjects });
    }
}

/// Allocation bound for an owned client document in a root's held output.
#[must_use]
pub fn document_worst_case(_limits: &Limits) -> Option<u64> {
    u64::try_from(size_of::<ClientDocument>()).ok()
}
