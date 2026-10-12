//! The root selects the inbox's restart range (store.md 7.2).
use crate::{Domain, Limits};
#[must_use]
pub fn inbox_census(domain: &mut Domain, limits: &Limits) -> jig_domain_core::Census {
    domain.core.census(&limits.core)
}
