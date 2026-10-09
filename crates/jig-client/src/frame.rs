//! The shared frame, last delivered facts and selected address.
//! The frame confirms party and retention, and answers siblings within a step.
//! See client/domain/client.md, 3 and 8.
use crate::{address::Address, contract::Frame, link::Link};
/// A sibling hand off of who is signed in and where.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FrameShown {
    pub party: Option<u64>,
    pub address: Option<Address>,
    pub link: Link,
}

/// The current committed frame document.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct FrameState {
    pub document: Option<Frame>,
}

impl FrameState {
    /// The authenticated party from the latest frame, never from kept work.
    #[must_use]
    pub fn party(&self) -> Option<u64> {
        let Some(frame) = &self.document else {
            return None;
        };
        Some(frame.party.number)
    }
}

/// Checks each allocation this spine itself keeps from the frame. Other whole
/// documents arrive within the decoding owner's `document_bytes` budget.
#[must_use]
pub fn fits(frame: &Frame, limits: &crate::Limits) -> bool {
    if !frame.party.name.fits(limits.text) || frame.projects.capacity() > limits.projects {
        return false;
    }
    for project in &frame.projects {
        if !project.name.fits(limits.text)
            || project.roles.capacity() > limits.roles
            || project.chats.capacity() > limits.charters
            || project.goals.capacity() > limits.charters
        {
            return false;
        }
        for charter in &project.chats {
            if !charter.name.fits(limits.text) {
                return false;
            }
        }
        for charter in &project.goals {
            if !charter.name.fits(limits.text) {
                return false;
            }
        }
    }
    true
}
