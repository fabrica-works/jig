//! Future batch members' contracts and funding (domain/tasks.md, 3 and 10;
//! client/domain/contract.md, 5). Contract codes and fields belong to their owner.
use super::{FieldRule, OwnerPart};
use alloc::boxed::Box;
use skein_lib::List;

/// One admitted result choice, with no invented human-facing label.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VerdictChoice {
    pub code: u32,
    pub words: u32,
    pub followups: u32,
    pub fields: List<FieldRule>,
}

/// Result form, owner bounds and explicit projection completeness.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TaskContractDocument {
    Approval { state: Box<[u8]>, part: OwnerPart },
    Report { words: u32 },
    Verdict { choices: List<VerdictChoice>, cut: bool },
    Change { connector: u16, kind: u16, words: u32 },
}

/// Financial source, independent of requester and executor topology.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Funder {
    Task(u64),
    Pool { project: u32, person: u64, period: u64 },
    Recurring { project: u32, task: u64, period: u64 },
    Period { project: u32, period: u64 },
}
