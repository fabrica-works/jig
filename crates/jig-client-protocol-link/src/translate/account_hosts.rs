//! Configured names retained in an operation answer.
use super::w;
use alloc::boxed::Box;
use jig_client_domain::contract as c;
use skein_lib::List;

pub(super) fn decode(
    rows: &List<w::AccountHostFailure>,
    confirmed: bool,
    limits: &crate::Limits,
) -> Result<Box<[c::AccountHostFailure]>, w::Problem> {
    if confirmed && !rows.is_empty() {
        return Err(super::piece_problem(skein_codec::Reason::Malformed));
    }
    let mut room = limits.client.document_bytes.checked_mul(2).ok_or(super::too_large())?;
    super::documents::charge(&mut room, List::<c::AccountHostFailure>::worst_case(rows.len()))?;
    let mut out = List::with_capacity(rows.len());
    for row in rows {
        let name = super::programs::name(row.name(), &mut room)?;
        let why = match row.why() {
            w::AccountHostWhy::NotInContact => c::AccountHostWhy::NotInContact,
            w::AccountHostWhy::Lost => c::AccountHostWhy::Lost,
            w::AccountHostWhy::Failed => c::AccountHostWhy::Failed,
        };
        out.push(c::AccountHostFailure { name, why }).expect("one decoded host per wire row");
    }
    Ok(out.into_boxed())
}
