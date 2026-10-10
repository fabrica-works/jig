//! One keyed edit to project policy; domain/people.md, 5.2.
//! These are whole owner values, with no policy inferred from display text.
use super::{OwnerAuthority, OwnerGrant, OwnerPattern};
use skein_lib::{Duration, List};

/// One mutable part of project policy, sent in one request.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PolicyEdit {
    ProjectSpend { period_spend: u64 },
    Role(PolicyEditRole),
    Requirements { requirements: List<PolicyRequirement> },
    Projections { grants: List<OwnerGrant> },
    Permissions { mappings: List<PolicyPermission> },
}

/// A role's actual authority, period allotment and decision flags.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PolicyEditRole {
    pub number: u32,
    pub authority: OwnerAuthority,
    pub period_spend: u64,
    pub requests: u16,
    pub decides: u8,
}

/// One connector permission mapped to a project role.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PolicyPermission {
    pub connector: u16,
    pub permission: u16,
    pub role: u32,
}

/// The connector-owned judge and its parameter identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PolicyJudge {
    pub connector: u16,
    pub requirement: u16,
    pub parameters: u32,
}

/// Application-time guard or an observation's allowed age.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PolicyGuard {
    Guarded,
    Observed { freshness: Duration },
}

/// The complete effect requirement, including literal pattern boundaries.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PolicyRequirement {
    pub connector: u16,
    pub kind: u16,
    pub pattern: OwnerPattern,
    pub judge: PolicyJudge,
    pub guard: PolicyGuard,
    pub must_be_guarded: bool,
}
