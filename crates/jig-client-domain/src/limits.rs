//! Bounded client handles and engine watch documents (client/domain/client.md, 14).
use crate::{Handle, domain::HandleState, watches::Watch};
use skein_lib::{Duration, List, Map, RequestLimits, RequestTable};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Backoff {
    pub first: Duration,
    pub most: Duration,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub requests: RequestLimits,
    pub ask_bytes: u64,
    pub handles: u32,
    pub watches: u32,
    pub reads: u32,
    pub document_bytes: u64,
    pub backoff: Backoff,
}
#[must_use]
pub fn max_out(limits: &Limits) -> u32 {
    limits
        .handles
        .checked_mul(2)
        .expect("validated handles")
        .checked_add(limits.watches)
        .expect("validated output bound")
        .checked_add(limits.requests.out)
        .expect("validated table output")
        .checked_add(limits.requests.requests.checked_mul(2).expect("validated request count"))
        .expect("validated progress output")
        .checked_add(1)
        .expect("validated drop output")
}
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    if limits.watches == 0 || limits.backoff.first == Duration::ZERO || limits.backoff.first > limits.backoff.most {
        return None;
    }
    if limits.requests.out == 0
        || limits.requests.first == Duration::ZERO
        || limits.requests.first > limits.requests.most
    {
        return None;
    }
    limits
        .handles
        .checked_mul(2)?
        .checked_add(limits.watches)?
        .checked_add(limits.requests.out)?
        .checked_add(limits.requests.requests.checked_mul(2)?)?
        .checked_add(1)?;
    let own = Map::<Handle, HandleState>::worst_case(limits.handles)?
        .checked_add(Map::<u64, Watch>::worst_case(limits.watches)?)?;
    let own = own.checked_add(Map::<Handle, skein_lib::Token>::worst_case(limits.reads)?)?;
    let own = own
        .checked_add(List::<Handle>::worst_case(limits.handles)?)?
        .checked_add(List::<u64>::worst_case(limits.watches)?)?;
    let own = own
        .checked_add(RequestTable::<crate::Key, crate::Held>::worst_case(&limits.requests)?)?
        .checked_add(Map::<crate::Key, crate::requests::PendingRequest>::worst_case(limits.requests.requests)?)?
        .checked_add(limits.requests.bytes.checked_mul(2)?)?;
    own.checked_add(
        u64::from(limits.handles.checked_add(limits.watches)?).checked_mul(limits.document_bytes.checked_mul(2)?)?,
    )
}
