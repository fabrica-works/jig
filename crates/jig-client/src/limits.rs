//! Container capacities and byte bounds owned by the client.
//! Every container is allocated from these at new. Jig 20.5 completes the spine
//! terms; jig 21–23 add their own entities. See client/domain/client.md, 14.
use crate::{
    contract::{Charter, Frame, ProjectRole},
    domain::Timer,
    store::Record,
    watches::Watch,
};
use core::mem::size_of;
use skein_lib::{Deadlines, Duration, Id, List, Map, Slab};

/// Backoff spans: double the first up to most, with seeded bounded jitter.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Backoff {
    pub first: Duration,
    pub most: Duration,
}

/// Configured limits for the frame and watch spine.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Limits {
    pub watches: u32,
    pub intents: u32,
    pub fields: u32,
    pub providers: u32,
    pub objects: u32,
    pub sources: u32,
    pub reads: u32,
    pub parts: u32,
    /// Total owned heap within any one decoded document, including its containers.
    pub document_bytes: u64,
    pub records: u32,
    pub projects: u32,
    pub roles: u32,
    pub charters: u32,
    pub text: u32,
    pub rows: u32,
    pub backoff: Backoff,
    pub heartbeat: Duration,
}

/// The heap held by the implemented spine, including frame and record payloads.
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    if limits.watches == 0
        || limits.heartbeat == Duration::ZERO
        || limits.backoff.first == Duration::ZERO
        || limits.backoff.first > limits.backoff.most
    {
        return None;
    }
    limits
        .watches
        .checked_mul(2)?
        .checked_add(limits.objects)?
        .checked_add(limits.parts.checked_mul(2)?)?
        .checked_add(16)?;
    limits.objects.checked_mul(2)?;
    if limits.sources == 0 {
        return None;
    }
    let mut total = Slab::<Watch>::worst_case(limits.watches)?;
    total = total.checked_add(crate::intents::worst_case(limits)?)?;
    total = total.checked_add(crate::fields::worst_case(limits)?)?;
    total = total.checked_add(crate::signin::worst_case(limits)?)?;
    total = total.checked_add(crate::objects::worst_case(limits)?)?;
    total = total.checked_add(Slab::<crate::reads::Reading>::worst_case(limits.reads)?)?;
    total = total.checked_add(Map::<Id<crate::reads::Reading>, ()>::worst_case(limits.reads)?)?;
    total = total.checked_add(u64::from(limits.reads).checked_mul(limits.document_bytes)?)?;
    total = total.checked_add(crate::windows::worst_case(limits)?)?;
    total = total.checked_add(Map::<Id<Watch>, ()>::worst_case(limits.watches)?)?;
    total = total.checked_add(Deadlines::<Timer>::worst_case(limits.watches)?)?;
    total = total.checked_add(List::<Record>::worst_case(limits.records)?)?;
    total = total.checked_add(u64::from(limits.records).checked_mul(u64::from(limits.text))?)?;
    total = total.checked_add(u64::try_from(size_of::<Frame>()).ok()?)?;
    total = total.checked_add(u64::from(limits.text))?;
    total = total.checked_add(List::<ProjectRole>::worst_case(limits.projects)?)?;
    let charters = List::<Charter>::worst_case(limits.charters)?
        .checked_add(u64::from(limits.charters).checked_mul(u64::from(limits.text))?)?
        .checked_mul(2)?;
    let project = charters.checked_add(List::<u16>::worst_case(limits.roles)?)?.checked_add(u64::from(limits.text))?;
    total = total.checked_add(u64::from(limits.projects).checked_mul(project)?)?;
    // Chips subjects own at most one configured task-number list each.
    total.checked_add(u64::from(limits.watches).checked_mul(List::<u64>::worst_case(limits.rows)?)?)
}
