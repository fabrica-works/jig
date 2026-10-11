//! Recursive admission for deployment-owned documents (contract.md, 5.3).
use crate::{Bounds, payload::Payload};
use core::mem::size_of;
use jig_client::contract::{
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
            Self::Worker => Some(0),
            Self::Engine => Some(0),
        }
    }
}
impl Payload for OwnerAttemptState {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Waiting => Some(0),
            Self::Adopted => Some(0),
            Self::Claimed => Some(0),
            Self::Cancelled => Some(0),
            Self::Handed => Some(0),
            Self::Acknowledged => Some(0),
            Self::Stray => Some(0),
            Self::Kept => Some(0),
            Self::Fenced => Some(0),
            Self::Closed => Some(0),
        }
    }
}
impl Payload for OwnerRunLocation {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        match self {
            Self::None => Some(0),
            Self::Connected { .. } => Some(0),
            Self::Adrift { .. } => Some(0),
        }
    }
}
impl Payload for OwnerAccountState {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Fresh => Some(0),
            Self::Refreshing => Some(0),
            Self::Retrying => Some(0),
            Self::Revoked => Some(0),
            Self::Closing => Some(0),
        }
    }
}
impl Payload for OwnerHelloRefusal {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        match self {
            Self::ReservedChannel => Some(0),
            Self::Workers { .. } => Some(0),
            Self::StopBound { .. } => Some(0),
        }
    }
}
impl Payload for OwnerPersonCapacityAction {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Close => Some(0),
            Self::Application { .. } => Some(0),
            Self::Note { .. } => Some(0),
            Self::Service => Some(0),
            Self::Adoption => Some(0),
            Self::Roles => Some(0),
            Self::Policy => Some(0),
            Self::Pool { .. } => Some(0),
            Self::Chat => Some(0),
            Self::Goal => Some(0),
            Self::Escalation { .. } => Some(0),
            Self::Proposal { .. } => Some(0),
            Self::Words { .. } => Some(0),
            Self::Question { .. } => Some(0),
            Self::Priority => Some(0),
            Self::Amend { .. } => Some(0),
            Self::Move { .. } => Some(0),
            Self::TakePerson { .. } => Some(0),
            Self::HandBackPerson { .. } => Some(0),
            Self::AnswerPerson { .. } => Some(0),
            Self::Stop { .. } => Some(0),
            Self::Cancel { .. } => Some(0),
            Self::Release { .. } => Some(0),
            Self::MarkSeen { .. } => Some(0),
        }
    }
}
impl Payload for OwnerPersonCapacityCause {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Requests { .. } => Some(0),
            Self::Pending { .. } => Some(0),
            Self::Waiters { .. } => Some(0),
            Self::Limit { .. } => Some(0),
        }
    }
}
impl Payload for OwnerTaskCapacityAction {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Batch { .. } => Some(0),
            Self::Proposal { .. } => Some(0),
            Self::PersonProposal { .. } => Some(0),
            Self::Message { .. } => Some(0),
            Self::Source { .. } => Some(0),
            Self::Introduce { .. } => Some(0),
            Self::Subscription { .. } => Some(0),
            Self::Effect { .. } => Some(0),
            Self::Claim { .. } => Some(0),
            Self::Release => Some(0),
            Self::Move => Some(0),
        }
    }
}
impl Payload for OwnerTaskCapacityCause {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Tasks { .. } => Some(0),
            Self::CreationFlights { .. } => Some(0),
            Self::ResourceReports { .. } => Some(0),
            Self::ProjectTasks { .. } => Some(0),
            Self::Tree { .. } => Some(0),
            Self::Delegates { .. } => Some(0),
            Self::Inbox { .. } => Some(0),
            Self::Holdings { .. } => Some(0),
            Self::HoldWaiters { .. } => Some(0),
            Self::Proposals { .. } => Some(0),
            Self::References { .. } => Some(0),
            Self::Subscriptions { .. } => Some(0),
            Self::Funders { .. } => Some(0),
            Self::Writers { .. } => Some(0),
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
