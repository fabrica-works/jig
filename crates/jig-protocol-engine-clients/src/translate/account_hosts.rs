//! Retained configured host identities (accounts.md, 3.4).
use super::w;
use jig_domain_people as p;
use skein_lib::List;
pub(super) fn encode(
    rows: &[p::AccountHostFailure],
    limits: &w::Limits,
) -> Result<List<w::AccountHostFailure>, w::Problem> {
    let Ok(count) = u32::try_from(rows.len()) else { return Err(count_problem_value()) };
    if count > 128 {
        return Err(count_problem_value());
    }
    let mut out = List::with_capacity(count);
    for row in rows {
        let why = match row.why {
            p::AccountHostWhy::NotInContact => w::AccountHostWhy::NotInContact,
            p::AccountHostWhy::Lost => w::AccountHostWhy::Lost,
            p::AccountHostWhy::Failed => w::AccountHostWhy::Failed,
        };
        let row = w::AccountHostFailure::new(limits, w::AccountHostFailureParts { name: row.name.clone(), why })?;
        if out.push(row).is_err() {
            return Err(count_problem_value());
        }
    }
    Ok(out)
}
fn count_problem_value() -> w::Problem {
    w::Problem { path: w::Path::OwnerOutcomeAccountSignedOutFailedHosts, reason: skein_codec::Reason::Bound }
}
