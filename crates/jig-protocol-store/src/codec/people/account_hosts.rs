//! Retained configured host identities (accounts.md, 3.4).
use super::{domain, wire};
use alloc::boxed::Box;
use skein_lib::List;
pub(super) fn encode(
    rows: &[domain::AccountHostFailure],
    limits: &wire::Limits,
) -> Result<List<wire::AccountHostFailure>, wire::Problem> {
    let Ok(count) = u32::try_from(rows.len()) else { return Err(count_problem_value()) };
    if count > 128 {
        return Err(count_problem_value());
    }
    let mut out = List::with_capacity(count);
    for row in rows {
        let why = match row.why {
            domain::AccountHostWhy::NotInContact => wire::AccountHostWhy::NotInContact,
            domain::AccountHostWhy::Lost => wire::AccountHostWhy::Lost,
            domain::AccountHostWhy::Failed => wire::AccountHostWhy::Failed,
        };
        let row = wire::AccountHostFailure::new(limits, wire::AccountHostFailureParts { name: row.name.clone(), why })?;
        if out.push(row).is_err() {
            return Err(count_problem_value());
        }
    }
    Ok(out)
}
fn count_problem_value() -> wire::Problem {
    wire::Problem { path: wire::Path::OutcomeAccountSignedOutFailedHosts, reason: skein_codec::Reason::Bound }
}

pub(super) fn decode(rows: &List<wire::AccountHostFailure>) -> Box<[domain::AccountHostFailure]> {
    let mut out = List::with_capacity(rows.len());
    for row in rows {
        let why = match row.why() {
            wire::AccountHostWhy::NotInContact => domain::AccountHostWhy::NotInContact,
            wire::AccountHostWhy::Lost => domain::AccountHostWhy::Lost,
            wire::AccountHostWhy::Failed => domain::AccountHostWhy::Failed,
        };
        out.push(domain::AccountHostFailure { name: skein_lib::bytes::copy_of(row.name()), why })
            .expect("one decoded host per wire row");
    }
    out.into_boxed()
}
