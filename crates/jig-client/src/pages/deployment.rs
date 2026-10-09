//! Deployment facts and connector part references, only for its admitted page.
//! Hosts, accounts and capacity are whole owner documents. No connector value is
//! kept. See client/domain/pages.md, 7, and client/domain/root.md, 6.
use crate::{
    Domain, Request,
    contract::{Deployment, Part},
};
use alloc::boxed::Box;
use skein_lib::{Env, List, Queue};
/// Latest deployment document for the admitted page.
#[derive(Debug)]
pub struct DeploymentPage {
    pub document: Option<Box<Deployment>>,
}
impl DeploymentPage {
    pub(crate) const fn new() -> Self {
        Self { document: None }
    }
}
pub(crate) fn document(domain: &mut Domain, env: &Env<crate::Limits>, doc: Box<Deployment>, out: &mut Queue<Request>) {
    if domain.address != crate::address::Address::Deployment {
        return;
    }
    let mut parts = List::<Part>::with_capacity(env.limits.parts.saturating_add(1));
    for part in &doc.parts {
        if parts.room() == 0 {
            break;
        }
        parts.push(part.clone()).expect("part fits");
    }
    domain.deployment_page.document = Some(doc);
    crate::pages::project::show(domain, env, parts, out);
    domain.changed();
}
/// Deployment's decoded whole document remains within its document-byte bound.
#[must_use]
pub const fn worst_case(limits: &crate::Limits) -> Option<u64> {
    Some(limits.document_bytes)
}
