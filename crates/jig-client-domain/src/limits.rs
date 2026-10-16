//! Bounded client handles and engine watch documents (client/domain/client.md, 14).
use crate::{Handle, domain::HandleState, watches::Watch};
use skein_lib::{Duration, List, Map};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Backoff {
    pub first: Duration,
    pub most: Duration,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub handles: u32,
    pub watches: u32,
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
}
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    if limits.watches == 0 || limits.backoff.first == Duration::ZERO || limits.backoff.first > limits.backoff.most {
        return None;
    }
    limits.handles.checked_mul(2)?.checked_add(limits.watches)?;
    let own = Map::<Handle, HandleState>::worst_case(limits.handles)?
        .checked_add(Map::<u64, Watch>::worst_case(limits.watches)?)?;
    let own = own
        .checked_add(List::<Handle>::worst_case(limits.handles)?)?
        .checked_add(List::<u64>::worst_case(limits.watches)?)?;
    own.checked_add(
        u64::from(limits.handles.checked_add(limits.watches)?).checked_mul(limits.document_bytes.checked_mul(2)?)?,
    )
}
