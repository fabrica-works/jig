//! The general operation targets and template overrides' complete payloads.
use super::{Bounds, Payload};
use jig_client_domain::contract::{AnswerOn, DecideOn, TemplateOverride, Tracking};
impl Payload for AnswerOn {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Question { words, .. } => words.heap(bounds),
            Self::Person { answer, .. } => answer.heap(bounds),
        }
    }
}
impl Payload for DecideOn {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Proposal { decision, .. } | Self::Escalation { decision, .. } => decision.heap(bounds),
        }
    }
}
impl Payload for Tracking {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        Some(0)
    }
}
impl Payload for TemplateOverride {
    fn heap(&self, _bounds: &Bounds) -> Option<u64> {
        Some(0)
    }
}

impl Payload for jig_client_domain::contract::Operation {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Message { words, .. } => words.heap(bounds),
            Self::Amend { change, reason, .. } => change.heap(bounds)?.checked_add(reason.heap(bounds)?),
            Self::Cancel { reason, .. } | Self::Move { reason, .. } => reason.heap(bounds),
            Self::Release { .. } | Self::Stop { .. } => Some(0),
        }
    }
}
