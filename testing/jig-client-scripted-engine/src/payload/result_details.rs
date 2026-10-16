//! Whole result metadata pricing (client/domain/contract.md, 5.3).
use super::{Payload, sum};
use crate::Bounds;
use jig_client_domain::contract::{OwnerRequester, OwnerTaskLink, OwnerTaskStatus, ResultDetails};

impl Payload for OwnerRequester {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Task(_) | Self::Person(_) | Self::Deployment { .. } => Some(0),
        }
    }
}
impl Payload for OwnerTaskStatus {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Done | Self::Failed | Self::Cancelled => Some(0),
        }
    }
}
impl Payload for OwnerTaskLink {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Live(chip) => chip.heap(bounds),
            Self::Retained { chip, at } => sum(&[chip.heap(bounds), at.heap(bounds)]),
            Self::Stored { number, status } => sum(&[number.heap(bounds), status.heap(bounds)]),
        }
    }
}
impl Payload for ResultDetails {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        let Self { contract, requester, executor, chosen_by, delegates, followups, proposal, cut } = self;
        sum(&[
            contract.heap(bounds),
            requester.heap(bounds),
            executor.heap(bounds),
            chosen_by.heap(bounds),
            delegates.heap(bounds),
            followups.heap(bounds),
            proposal.heap(bounds),
            cut.heap(bounds),
        ])
    }
}

#[cfg(test)]
mod tests;
