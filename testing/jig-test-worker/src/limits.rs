//! Child limits remain in their owners' vocabulary (hosts.md, section 11).
use crate::{domain::Call, hand_out::Pending, workspace::Content};
use alloc::boxed::Box;
use jig_domain_host as host;
use skein_lib::{Duration, Map, Queue, Token};
use smith_host_domain as smith;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Limits {
    pub hub: host::Limits,
    pub agent: smith::Limits,
    pub bounds: host::Bounds,
    pub stop_bound: Duration,
    pub workstreams: Box<[u64]>,
}

#[must_use]
pub fn max_out(limits: &Limits) -> u32 {
    host::max_out(&limits.hub)
        .checked_mul(smith::max_out(&limits.agent))
        .expect("validated hand-offs")
        .checked_mul(2)
        .expect("validated round trip")
        .checked_add(limits.agent.messages)
        .expect("validated unread fanout")
        .checked_add(limits.hub.told)
        .expect("validated activity")
        .checked_add(replays(limits).expect("validated retained records"))
        .expect("validated replays")
        .checked_add(4)
        .expect("validated stop")
}

#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    smith::check(&limits.agent).ok()?;
    if !host::valid_program_endpoints(&limits.bounds.program_endpoints) {
        return None;
    }
    let hello_endpoints = u64::try_from(limits.bounds.program_endpoints.len()).ok()?.checked_mul(4)?;
    if limits.hub.slots > limits.agent.agents {
        return None;
    }
    host::max_out(&limits.hub)
        .checked_mul(smith::max_out(&limits.agent))?
        .checked_mul(2)?
        .checked_add(limits.agent.messages)?
        .checked_add(limits.hub.told)?
        .checked_add(replays(limits)?)?
        .checked_add(4)?;
    let calls = limits.hub.slots.checked_mul(limits.agent.calls)?;
    let call =
        limits.hub.event_bytes.checked_add(limits.agent.call_bytes)?.checked_add(smith::Delivered::worst_case())?;
    let scratch = Queue::<host::Request>::worst_case(host::max_out(&limits.hub))?
        .checked_add(Queue::<smith::Output>::worst_case(smith::max_out(&limits.agent))?)?
        .checked_add(limits.agent.turn_bytes)?
        .checked_add(limits.agent.outcome_bytes)?;
    host::worst_case(&limits.hub)?
        .checked_add(smith::worst_case(&limits.agent)?)?
        .checked_add(Map::<Token, Token>::worst_case(limits.hub.slots)?)?
        .checked_add(Map::<(Token, Token), Call>::worst_case(calls)?)?
        .checked_add(u64::from(calls).checked_mul(call)?)?
        .checked_add(Queue::<Pending>::worst_case(pending(limits)?)?)?
        .checked_add(Map::<Token, Box<[u8]>>::worst_case(limits.hub.slots)?)?
        .checked_add(u64::from(limits.hub.slots).checked_mul(u64::from(limits.agent.detail_bytes))?)?
        .checked_add(Map::<Token, Option<Content>>::worst_case(limits.hub.slots)?)?
        .checked_add(u64::from(limits.hub.slots).checked_mul(limits.agent.call_bytes)?)?
        .checked_add(hello_endpoints)?
        .checked_add(scratch)
}

pub(crate) fn pending(limits: &Limits) -> Option<u32> {
    limits.hub.slots.checked_mul(limits.agent.calls.checked_add(limits.agent.turns)?.checked_add(2)?)
}

fn replays(limits: &Limits) -> Option<u32> {
    limits.hub.slots.checked_mul(limits.hub.turns.checked_mul(2)?.checked_add(1)?)
}
