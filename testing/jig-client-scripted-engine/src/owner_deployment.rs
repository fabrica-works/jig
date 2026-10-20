//! Recursive admission for deployment-owned documents (contract.md, 5.3).
use crate::{Bounds, payload::Payload};
use core::mem::size_of;
use jig_client_domain::contract::{
    OwnerAccount, OwnerAccountState, OwnerAttemptState, OwnerCapacity, OwnerDeployment, OwnerHelloRefusal,
    OwnerHelloRefusalLine, OwnerHost, OwnerHostKind, OwnerHostRun, OwnerPersonCapacityAction, OwnerPersonCapacityCause,
    OwnerPersonCapacityRefusal, OwnerProjectCapacity, OwnerRequestKey, OwnerRunLocation, OwnerTaskCapacityAction,
    OwnerTaskCapacityCause, OwnerTaskCapacityRefusal, OwnerTaskCapacityRequest,
};
impl Payload for OwnerDeployment {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        let mut total = 0_u64;
        total = total.checked_add(self.hosts.heap(bounds)?)?;
        total = total.checked_add(self.attempts.heap(bounds)?)?;
        total = total.checked_add(self.accounts.heap(bounds)?)?;
        total = total.checked_add(self.projects.heap(bounds)?)?;
        total = total.checked_add(self.hello_refusals.heap(bounds)?)?;
        total = total.checked_add(self.person_room_refusals.heap(bounds)?)?;
        total = total.checked_add(self.task_room_refusals.heap(bounds)?)?;
        total = total.checked_add(self.connectors.heap(bounds)?)?;
        Some(total)
    }
}
impl Payload for OwnerCapacity {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        Some(0)
    }
}
impl Payload for OwnerProjectCapacity {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        Some(0)
    }
}
impl Payload for OwnerHost {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        self.runs.heap(bounds)
    }
}
impl Payload for OwnerHostRun {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        self.task.heap(bounds)
    }
}
impl Payload for OwnerAccount {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        let mut total = 0_u64;
        total = total.checked_add(self.runs.heap(bounds)?)?;
        total = total.checked_add(self.waiting.heap(bounds)?)?;
        total = total.checked_add(self.display.heap(bounds)?)?;
        total = total.checked_add(self.provider_name.heap(bounds)?)?;
        total = total.checked_add(self.manage_usage.heap(&Bounds { text: 2048, ..*bounds })?)?;
        total = total.checked_add(self.sources.heap(bounds)?)?;
        total = total.checked_add(self.uses.heap(&Bounds { rows: 16, ..*bounds })?)?;
        Some(total)
    }
}
impl Payload for OwnerHelloRefusalLine {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        Some(0)
    }
}
impl Payload for OwnerRequestKey {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        Some(0)
    }
}
impl Payload for OwnerPersonCapacityRefusal {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        Some(0)
    }
}
impl Payload for OwnerTaskCapacityRefusal {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        Some(0)
    }
}
impl Payload for OwnerTaskCapacityRequest {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        Some(0)
    }
}
impl Payload for OwnerHostKind {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Worker | Self::Engine => Some(0),
        }
    }
}
impl Payload for OwnerAttemptState {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Waiting
            | Self::Adopted
            | Self::Claimed
            | Self::Cancelled
            | Self::Handed
            | Self::Acknowledged
            | Self::Stray
            | Self::Kept
            | Self::Fenced
            | Self::Closed => Some(0),
        }
    }
}
impl Payload for OwnerRunLocation {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        match self {
            Self::None | Self::Connected { .. } | Self::Adrift { .. } => Some(0),
        }
    }
}
impl Payload for OwnerAccountState {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Asking
            | Self::Granted
            | Self::Later { .. }
            | Self::SignInAgain
            | Self::Misconfigured
            | Self::Unloaded { .. }
            | Self::Cancelled
            | Self::Closing => Some(0),
        }
    }
}
impl Payload for OwnerHelloRefusal {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        match self {
            Self::ReservedChannel | Self::Workers { .. } | Self::StopBound { .. } | Self::Agent { .. } => Some(0),
        }
    }
}
impl Payload for OwnerPersonCapacityAction {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        match self {
            Self::AccountSignIn
            | Self::AccountSignOut
            | Self::AccountChange
            | Self::Create
            | Self::Compound
            | Self::Application { .. }
            | Self::Note { .. }
            | Self::Service
            | Self::Adoption
            | Self::Roles
            | Self::Policy
            | Self::Pool { .. }
            | Self::Escalation { .. }
            | Self::Proposal { .. }
            | Self::Words { .. }
            | Self::Question { .. }
            | Self::Amend { .. }
            | Self::Move { .. }
            | Self::TakePerson { .. }
            | Self::HandBackPerson { .. }
            | Self::AnswerPerson { .. }
            | Self::Stop { .. }
            | Self::Cancel { .. }
            | Self::Release { .. }
            | Self::MarkSeen { .. } => Some(0),
        }
    }
}
impl Payload for OwnerPersonCapacityCause {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Requests { .. } | Self::Pending { .. } | Self::Waiters { .. } | Self::Limit { .. } => Some(0),
        }
    }
}
impl Payload for OwnerTaskCapacityAction {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Batch { .. }
            | Self::Proposal { .. }
            | Self::PersonProposal { .. }
            | Self::Message { .. }
            | Self::Source { .. }
            | Self::Introduce { .. }
            | Self::Subscription { .. }
            | Self::Effect { .. }
            | Self::Claim { .. }
            | Self::Release
            | Self::Move => Some(0),
        }
    }
}
impl Payload for OwnerTaskCapacityCause {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Tasks { .. }
            | Self::CreationFlights { .. }
            | Self::ResourceReports { .. }
            | Self::ProjectTasks { .. }
            | Self::Tree { .. }
            | Self::Delegates { .. }
            | Self::Inbox { .. }
            | Self::Holdings { .. }
            | Self::HoldWaiters { .. }
            | Self::Proposals { .. }
            | Self::References { .. }
            | Self::Subscriptions { .. }
            | Self::Funders { .. }
            | Self::Writers { .. } => Some(0),
        }
    }
}

/// Whole inline owner value and every reserved nested allocation.
#[must_use]
pub fn bytes(value: &OwnerDeployment, bounds: &Bounds) -> Option<u64> {
    u64::try_from(size_of::<OwnerDeployment>()).ok()?.checked_add(value.heap(bounds)?)
}
/// Reject a whole document before retaining it.
#[must_use]
pub fn fits(value: &OwnerDeployment, bounds: &Bounds) -> bool {
    crate::payload::fits(value, bounds)
}
/// Owner documents stay under the configured whole byte ceiling.
#[must_use]
pub fn worst_case(bounds: &Bounds) -> Option<u64> {
    if bounds.document_bytes < u64::try_from(size_of::<OwnerDeployment>()).ok()? {
        None
    } else {
        Some(bounds.document_bytes)
    }
}

impl Payload for jig_client_domain::contract::OwnerAccountUse {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        Some(0)
    }
}
impl Payload for jig_client_domain::contract::OwnerAccountSource {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        Some(0)
    }
}
impl Payload for jig_client_domain::contract::OwnerAccountSignIn {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        Some(0)
    }
}
impl Payload for jig_client_domain::contract::OwnerAccountLine {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        self.display
            .heap(bounds)?
            .checked_add(self.provider_name.heap(bounds)?)?
            .checked_add(self.manage_usage.heap(&Bounds { text: 2048, ..*bounds })?)?
            .checked_add(self.sources.heap(bounds)?)?
            .checked_add(self.uses.heap(&Bounds { rows: 16, ..*bounds })?)
    }
}
impl Payload for jig_client_domain::contract::OwnerAccounts {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        self.accounts.heap(bounds)?.checked_add(self.signing.heap(bounds)?)
    }
}
