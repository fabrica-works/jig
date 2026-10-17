//! Keep: the child routing shape and exhaustive connector variants.
//! Replace: Ops connector sides, family addresses and application vocabulary.
use crate::{Request, infrastructure};
use skein_lib::Queue;
/// Application-configured child and client bounds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub jig: jig_client_view::Limits,
    pub infrastructure: infrastructure::Limits,
    pub observability: crate::observability::Limits,
    pub client: jig_client::Limits,
}
#[must_use]
pub fn max_out(limits: &Limits) -> u32 {
    checked_max_out(limits).expect("root output capacities are checked")
}
fn checked_max_out(limits: &Limits) -> Option<u32> {
    jig_client_view::max_out(&limits.jig)
        .checked_mul(5)?
        .checked_add(infrastructure::checked_max_out(&limits.infrastructure)?)?
        .checked_add(crate::observability::checked_max_out(&limits.observability)?)?
        .checked_add(jig_client::max_out(&limits.client))?
        .checked_add(16)
}

#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    if limits.jig.kept < limits.client.domain.requests.requests {
        return None;
    }
    let mut total = jig_client_view::worst_case(&limits.jig)?;
    total = total.checked_add(infrastructure::worst_case(&limits.infrastructure)?)?;
    total = total.checked_add(crate::observability::worst_case(&limits.observability)?)?;
    total = total.checked_add(jig_client::worst_case(&limits.client)?)?;
    // Synchronous frame hand-offs nest at most three child output queues.
    let jig_out = jig_client_view::max_out(&limits.jig);
    let root_out = checked_max_out(limits)?;
    total = total.checked_add(Queue::<jig_client_view::Request>::worst_case(jig_out)?.checked_mul(3)?)?;
    total = total.checked_add(
        Queue::<infrastructure::Request>::worst_case(infrastructure::max_out(&limits.infrastructure))?
            .checked_mul(3)?,
    )?;
    total = total.checked_add(u64::from(jig_out).checked_mul(limits.jig.document_bytes)?.checked_mul(3)?)?;
    total = total.checked_add(u64::from(root_out).checked_mul(limits.jig.document_bytes)?)?;
    let family_payload =
        skein_lib::List::<jig_client::contract::ObjectKey>::worst_case(limits.infrastructure.rows.checked_add(1)?)?
            .checked_add(u64::from(limits.infrastructure.text).checked_mul(2)?)?;
    total = total.checked_add(
        u64::from(infrastructure::max_out(&limits.infrastructure)).checked_mul(family_payload)?.checked_mul(3)?,
    )?;
    let observability_payload =
        skein_lib::List::<jig_client::contract::ObjectKey>::worst_case(limits.observability.rows.checked_add(1)?)?
            .max(skein_lib::List::<jig_client_view::page::Section>::worst_case(1)?);
    total = total.checked_add(
        u64::from(crate::observability::max_out(&limits.observability))
            .checked_mul(observability_payload)?
            .checked_mul(3)?,
    )?;
    total = total.checked_add(
        Queue::<crate::observability::Request>::worst_case(crate::observability::max_out(&limits.observability))?
            .checked_mul(3)?,
    )?;
    total = total.checked_add(
        u64::from(crate::observability::max_out(&limits.observability))
            .checked_mul(limits.observability.document_bytes)?
            .checked_mul(3)?,
    )?;
    total = total.checked_add(
        u64::from(infrastructure::max_out(&limits.infrastructure))
            .checked_mul(limits.infrastructure.document_bytes)?
            .checked_mul(3)?,
    )?;
    total.checked_add(Queue::<Request>::worst_case(root_out)?.checked_mul(2)?)
}
