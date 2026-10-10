//! Result metadata and linked task identities (client/domain/objects.md, 8;
//! client/domain/contract.md, 5). Missing stored facts stay absent.
use super::{Chip, OwnerExecutor, ProposalKey, TaskContractDocument};
use skein_lib::{List, Wall};

/// The owner that requested a task, including deployment work.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OwnerRequester {
    /// A task with this never-reused number.
    Task(u64),
    /// A person with this party number.
    Person(u64),
    /// Deployment work for this project.
    Deployment { project: u32 },
}

/// An ended task's supplied status, without requiring its stored result.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OwnerTaskStatus {
    /// Completed successfully.
    Done,
    /// Failed.
    Failed,
    /// Cancelled.
    Cancelled,
}

/// A task reference with only the identity and history its owner supplied.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum OwnerTaskLink {
    /// A live, visible task with its whole chip.
    Live(Chip),
    /// An ended task whose chip is retained; its wall clock may be unavailable.
    Retained { chip: Chip, at: Option<Wall> },
    /// A stored task number, with a status only when supplied.
    Stored { number: u64, status: Option<OwnerTaskStatus> },
}

/// Result owner facts, separate from its value, ending, timestamp and acts.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResultDetails {
    pub contract: TaskContractDocument,
    pub requester: OwnerRequester,
    pub executor: OwnerExecutor,
    pub chosen_by: Option<u64>,
    pub delegates: List<OwnerTaskLink>,
    /// None means unavailable; Some(empty) supplies an empty reference list.
    pub followups: Option<List<OwnerTaskLink>>,
    pub proposal: Option<ProposalKey>,
    /// Aggregate owner list loss; it does not identify a particular list.
    pub cut: bool,
}
