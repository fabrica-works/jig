//! Exact owner budget and pool ledger facts (client/domain/contract.md, 5.1).
//! The owner keeps all four numbers, availability and funding identity
//! (domain/authority.md, section 7). The client never recomputes them.
use super::Funder;
use skein_lib::Wall;

/// The owner's four separate accounting numbers, in the deployment's unit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OwnerNumbers {
    pub budget: u64,
    pub spent: u64,
    pub spent_below: u64,
    pub reserved: u64,
}

/// A task budget's exact owner document fields, independent of its authority.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OwnerBudget {
    pub numbers: OwnerNumbers,
    pub left: u64,
    pub funder: Funder,
    pub deadline: Option<Wall>,
}

/// One pool or period ledger, including its explicit parent and timing facts.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PoolLedger {
    pub source: Funder,
    pub parent: Option<Funder>,
    pub numbers: OwnerNumbers,
    pub left: u64,
    pub closed: bool,
    pub began: Option<Wall>,
    pub next: Option<Wall>,
}
