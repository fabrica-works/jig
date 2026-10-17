//! Deployment facts and connector part references, only for its admitted page.
//! Hosts, accounts and capacity are whole owner documents. No connector value is
//! kept. See client/domain/pages.md, 7, and client/domain/root.md, 6.
use crate::{Domain, Request};
use alloc::boxed::Box;
use jig_client::contract::{Deployment, OwnerDeployment, Part};
use skein_lib::{Env, List, Queue};
/// Latest deployment document for the admitted page.
#[derive(Debug)]
pub struct DeploymentPage {
    pub document: Option<Box<Deployment>>,
    /// The latest deployment owner facts, independent of the synthetic parts path.
    pub owner: Option<Box<OwnerDeployment>>,
}
impl DeploymentPage {
    pub(crate) const fn new() -> Self {
        Self { document: None, owner: None }
    }
}
pub(crate) fn document(domain: &mut Domain, env: &Env<crate::Limits>, doc: Box<Deployment>, out: &mut Queue<Request>) {
    if !domain.shows(crate::page::Section::Deployment) {
        return;
    }
    let mut parts = List::<Part>::with_capacity(env.limits.parts.saturating_add(1));
    for part in &doc.parts {
        if parts.room() == 0 {
            break;
        }
        parts.push(part.clone()).expect("part fits");
    }
    domain.deployment_page.owner = None;
    domain.deployment_page.document = Some(doc);
    crate::page::project::show(domain, env, parts, out);
    domain.changed();
}
/// Deployment's decoded whole document remains within its document-byte bound.
#[must_use]
pub const fn worst_case(limits: &crate::Limits) -> Option<u64> {
    Some(limits.document_bytes)
}

/// Keep the whole owner value only for the admitted current deployment page.
pub(crate) fn owner_document(
    domain: &mut Domain,
    env: &Env<crate::Limits>,
    doc: Box<OwnerDeployment>,
    out: &mut Queue<Request>,
) {
    if !domain.shows(crate::page::Section::Deployment) {
        return;
    }
    let mut parts = List::with_capacity(env.limits.parts.saturating_add(1));
    for connector in &doc.connectors {
        if parts.room() == 0 {
            break;
        }
        parts.push(Part::Deployment { connector: *connector }).expect("configured reference fits");
    }
    domain.deployment_page.document = None;
    domain.deployment_page.owner = Some(doc);
    crate::page::project::show(domain, env, parts, out);
    domain.changed();
}
