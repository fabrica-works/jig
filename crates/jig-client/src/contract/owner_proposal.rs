//! Exact pending proposal facts (client/domain/objects.md, 2 and 4;
//! contract.md, 3.2 and 5). Historical routing and terminal decisions are
//! separate documents; missing funding observations remain absent.
use super::{
    AddressChange, DocumentText, Funder, Offered, OwnerAmendmentState, OwnerAuthorityDocument, OwnerChipHold,
    OwnerHistoryHolder, OwnerPart, OwnerTaskLink, ProposalKey, ProposedTask,
};
use alloc::boxed::Box;
use skein_lib::{List, Wall};
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OwnerProposal {
    pub key: ProposalKey,
    pub project: u32,
    pub reason: DocumentText,
    pub action: OwnerProposalAction,
    pub as_holder: bool,
    pub waiting: OwnerProposalWaiting,
    pub funding: List<OwnerFundingOffer>,
    pub acts: List<Offered>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OwnerProposalWaiting {
    pub holder: OwnerHistoryHolder,
    pub since: Wall,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OwnerFundingOffer {
    pub source: Funder,
    pub needed: u64,
    pub left: Option<u64>,
    pub after: Option<u64>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum OwnerProposalAction {
    Batch(List<ProposedTask>),
    Effect {
        part: OwnerPart,
        authority: OwnerAuthorityDocument,
        attempt: u64,
        completion: u32,
        position: u32,
        purpose: u64,
    },
    Amend {
        task: OwnerTaskLink,
        addressed: Option<AddressChange>,
        before: Box<OwnerAmendmentState>,
        after: Box<OwnerAmendmentState>,
        reason: DocumentText,
        release: Option<u64>,
    },
    Widen {
        task: OwnerTaskLink,
        before: OwnerAuthorityDocument,
        after: OwnerAuthorityDocument,
    },
    Release {
        task: OwnerTaskLink,
        hold: Option<Box<OwnerChipHold>>,
    },
}
