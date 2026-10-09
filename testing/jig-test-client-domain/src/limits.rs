//! Checked composition of child, queue, and table capacities (root.md, 3).
use crate::{Ask, Request, first, owner::Owner};
use skein_lib::{Queue, RequestLimits, RequestTable};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub jig: jig_client::Limits,
    pub first: first::Limits,
    pub second_parts: u32,
    pub table: RequestLimits,
}
#[must_use]
pub fn max_out(limits: &Limits) -> u32 {
    checked_max_out(limits).expect("root output capacities are checked")
}
fn checked_max_out(limits: &Limits) -> Option<u32> {
    jig_client::max_out(&limits.jig)
        .checked_mul(4)?
        .checked_add(first::max_out(&limits.first))?
        .checked_add(limits.table.out)?
        .checked_add(16)
}

#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    if limits.table.out == 0
        || limits.table.first == skein_lib::Duration::ZERO
        || limits.table.first > limits.table.most
    {
        return None;
    }
    let mut total = jig_client::worst_case(&limits.jig)?;
    total = total.checked_add(first::worst_case(&limits.first)?)?;
    total = total.checked_add(crate::second::worst_case(limits.second_parts, limits.first.text)?)?;
    total = total.checked_add(RequestTable::<Owner, Ask>::worst_case(&limits.table)?)?;
    total = total.checked_add(limits.table.bytes.checked_mul(3)?)?;
    // Synchronous frame hand-offs nest at most three child output queues.
    let jig_out = jig_client::max_out(&limits.jig);
    let root_out = checked_max_out(limits)?;
    total = total.checked_add(Queue::<jig_client::Request>::worst_case(jig_out)?.checked_mul(3)?)?;
    total = total.checked_add(Queue::<first::Request>::worst_case(first::max_out(&limits.first))?.checked_mul(3)?)?;
    total = total.checked_add(u64::from(jig_out).checked_mul(limits.jig.document_bytes)?.checked_mul(3)?)?;
    total = total.checked_add(u64::from(root_out).checked_mul(limits.jig.document_bytes)?)?;
    total.checked_add(Queue::<Request>::worst_case(root_out)?)
}
