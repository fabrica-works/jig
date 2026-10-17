//! Checked composition of child, queue, and table capacities (root.md, 3).
use crate::{Request, first};
use skein_lib::Queue;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub jig: jig_client_view::Limits,
    pub first: first::Limits,
    pub second_parts: u32,
    pub client: jig_client::Limits,
}
#[must_use]
pub fn max_out(limits: &Limits) -> u32 {
    checked_max_out(limits).expect("root output capacities are checked")
}
fn checked_max_out(limits: &Limits) -> Option<u32> {
    jig_client_view::max_out(&limits.jig)
        .checked_mul(5)?
        .checked_add(first::checked_max_out(&limits.first)?)?
        .checked_add(jig_client::max_out(&limits.client))?
        .checked_add(16)
}

#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    if limits.jig.kept < limits.client.domain.requests.requests {
        return None;
    }
    let mut total = jig_client_view::worst_case(&limits.jig)?;
    total = total.checked_add(first::worst_case(&limits.first)?)?;
    total =
        total.checked_add(crate::second::worst_case(limits.second_parts, limits.first.segments, limits.first.text)?)?;
    total = total.checked_add(jig_client::worst_case(&limits.client)?)?;
    // Named -> ShowPart/PartsLeft still nests first, jig, first: three child queues.
    let jig_out = jig_client_view::max_out(&limits.jig);
    let root_out = checked_max_out(limits)?;
    total = total.checked_add(Queue::<jig_client_view::Request>::worst_case(jig_out)?.checked_mul(3)?)?;
    total = total.checked_add(Queue::<first::Request>::worst_case(first::max_out(&limits.first))?.checked_mul(3)?)?;
    total = total.checked_add(u64::from(jig_out).checked_mul(limits.jig.document_bytes)?.checked_mul(3)?)?;
    total = total.checked_add(u64::from(root_out).checked_mul(limits.jig.document_bytes)?)?;
    let family_payload =
        skein_lib::List::<jig_client::contract::ObjectKey>::worst_case(limits.first.rows.checked_add(1)?)?
            .max(skein_lib::List::<jig_client_view::page::Section>::worst_case(2)?);
    total = total.checked_add(u64::from(first::max_out(&limits.first)).checked_mul(family_payload)?.checked_mul(3)?)?;
    total.checked_add(Queue::<Request>::worst_case(root_out)?.checked_mul(2)?)
}
