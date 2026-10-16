//! A party's remaining batch budget (domain/authority.md, 7 and 8.4).
use crate::{Domain, PersonAsk, left};

/// Resolve what fits after named amounts, within the role's spend and an optional cap.
/// Unknown projects and roles have no budget. This query changes no funding numbers.
#[must_use]
pub fn as_allowed(domain: &Domain, ask: &PersonAsk, named: u64, cap: Option<u64>) -> u64 {
    let Some(role) = domain.role(ask.project, ask.role) else { return 0 };
    let given = left(ask.pool).min(role.authority.budget.spend).saturating_sub(named);
    match cap {
        Some(cap) => given.min(cap),
        None => given,
    }
}
