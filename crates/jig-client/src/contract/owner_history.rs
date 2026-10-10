//! Whole immutable semantic history (client/domain/objects.md, 11;
//! domain/tasks.md, 7). Revision is not an activity commit position.
use super::{
    AddressChange, DocumentText, OwnerAmendmentState, OwnerAuthorityDocument, OwnerMessageProposalKind, OwnerPart,
    OwnerRequester, OwnerWakePolicy, ProposedTask, SpecificationDocument,
};
use alloc::boxed::Box;
use skein_lib::{List, Wall};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OwnerHistoryLine {
    pub task: u64,
    pub revision: u64,
    pub by: OwnerRequester,
    pub change: OwnerHistoryChange,
    pub reason: DocumentText,
    pub proposal: Option<OwnerProposalHistory>,
    /// Stored Settings snapshots, not a reconstruction from the selected delta.
    /// Absent instructions mean no stored override, not today's effective text.
    pub amendment: Option<(Box<OwnerAmendmentState>, Box<OwnerAmendmentState>)>,
    pub at: Option<Wall>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OwnerHistoryChange {
    Cancelled,
    Released,
    Amended,
    Prioritised { priority: u32 },
    Moved,
    Proposed,
    ProposalPassed,
    ProposalAccepted,
    ProposalRejected,
    ProposalWithdrawn,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OwnerProposalHistory {
    pub action: OwnerRetainedAction,
    pub number: u64,
    /// Raw task-origin proposer number; the source supplies no maker discriminator.
    pub proposer: u64,
    pub as_holder: bool,
    pub state: OwnerProposalHistoryState,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum OwnerProposalHistoryState {
    Pending { holder: OwnerHistoryHolder, since: Wall },
    Accepted { by: OwnerRequester },
    Rejected { by: OwnerRequester, reason: DocumentText },
    Withdrawn,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OwnerHistoryHolder {
    Task(u64),
    Person(u64),
    Policy { project: u32, kind: OwnerMessageProposalKind },
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum OwnerRetainedAction {
    /// Whole admitted batch; no synthetic outer loss marker or clipped batch.
    Batch(List<ProposedTask>),
    /// Raw owner reference; application scope is not supplied by this source.
    Effect {
        part: OwnerPart,
        authority: OwnerAuthorityDocument,
        attempt: u64,
        completion: u32,
        position: u32,
        purpose: u64,
    },
    Amend {
        task: u64,
        change: Box<OwnerRetainedAmendment>,
    },
    Widen {
        task: u64,
        authority: OwnerAuthorityDocument,
    },
    Release {
        task: u64,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OwnerRetainedAmendment {
    pub addressed: Option<AddressChange>,
    pub instructions: Option<DocumentText>,
    /// Named hold revision, independent of the person address and history revisions.
    pub release: Option<u64>,
    pub spec: Option<SpecificationDocument>,
    pub authority: Option<OwnerAuthorityDocument>,
    pub wake: Option<OwnerWakePolicy>,
    pub dependencies: Option<List<u64>>,
    pub reason: DocumentText,
    /// Dependency-list omission only; nested text/spec/authority loss stays independent.
    pub cut: bool,
}
