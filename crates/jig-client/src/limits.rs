//! The composition's bounded queues (client/domain/client.md, 14).
use alloc::boxed::Box;
use skein_lib::{List, Map, Queue, RequestKey};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub domain: crate::DomainLimits,
    pub exchanges: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unusable {
    Limits,
}
#[must_use]
pub fn max_out(limits: &Limits) -> u32 {
    let link = jig_client_protocol_link::Limits::new(limits.domain, limits.exchanges).expect("valid limits");
    jig_client_protocol_link::limits::max_out(&link)
        .expect("valid link")
        .checked_mul(jig_client_domain::max_out(&limits.domain).checked_add(2).expect("valid domain bound"))
        .expect("valid client output bound")
}
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    let link = jig_client_protocol_link::Limits::new(limits.domain, limits.exchanges).ok()?;
    let bound = jig_client_protocol_link::limits::max_out(&link)?
        .checked_mul(jig_client_domain::max_out(&limits.domain).checked_add(2)?)?;
    jig_client_domain::worst_case(&limits.domain)?
        .checked_add(jig_client_protocol_link::worst_case(&link)?)?
        .checked_add(Queue::<jig_client_domain::Request>::worst_case(bound)?)?
        .checked_add(Queue::<jig_client_protocol_link::Request>::worst_case(bound)?)?
        .checked_add(40)?
        .checked_add(Map::<RequestKey, u64>::worst_case(limits.domain.requests.requests)?)?
        .checked_add(List::<crate::Record>::worst_case(bound)?)?
        .checked_add(List::<Box<[u8]>>::worst_case(bound)?)?
        .checked_add(u64::from(bound).checked_mul(limits.domain.document_bytes.checked_mul(2)?)?)
}
