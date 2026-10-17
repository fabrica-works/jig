//! The shared frame, last delivered facts and selected address.
//! The frame confirms party and retention, and answers siblings within a step.
//! See client/domain/client.md, 3 and 8.
use crate::address::Address;
use jig_client::Link;
use jig_client::contract::Frame;
/// A sibling hand off of who is signed in and where.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrameShown {
    pub party: Option<u64>,
    pub address: Option<Address>,
    pub link: Link,
}

/// The current committed frame document.
#[derive(Debug, PartialEq, Eq)]
pub struct FrameState<'a> {
    pub document: Option<&'a Frame>,
}

impl FrameState<'_> {
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
    if !frame.party.name.bytes.fits(limits.text)
        || !frame.party.login.bytes.fits(limits.text)
        || frame.projects.capacity() > limits.projects
        || frame.charters.capacity() > limits.projects
    {
        return false;
    }
    for project in &frame.projects {
        if !name_fits(project.name.as_ref(), limits.text) {
            return false;
        }
    }
    for project in &frame.charters {
        if project.chats.capacity() > limits.charters || project.goals.capacity() > limits.charters {
            return false;
        }
        for charter in &project.chats {
            if !name_fits(charter.name.as_ref(), limits.text) {
                return false;
            }
        }
        for charter in &project.goals {
            if !name_fits(charter.name.as_ref(), limits.text) {
                return false;
            }
        }
    }
    true
}
fn name_fits(name: Option<&jig_client::contract::DocumentText>, text: u32) -> bool {
    match name {
        Some(name) => name.bytes.fits(text),
        None => true,
    }
}

/// Facts on every page's frame, with exact needs and bounded unread updates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Summary {
    pub party: Option<u64>,
    pub needs: u64,
    /// Independently supplied owner total, alongside the sum of supplied kinds.
    pub needs_total: u32,
    pub updates: jig_client::contract::Count,
    pub project: Option<u32>,
    pub link: Link,
    pub behind: bool,
}
impl FrameState<'_> {
    /// Derives display facts from the committed frame; never guesses unread counts.
    #[must_use]
    pub fn summary(&self, address: Address, link: Link, behind: bool, limit: u32) -> Summary {
        let mut summary = Summary {
            party: None,
            needs: 0,
            needs_total: 0,
            updates: jig_client::contract::Count { value: 0, limited: false },
            project: None,
            link,
            behind,
        };
        if let Some(frame) = &self.document {
            summary.party = Some(frame.party.number);
            summary.needs_total = frame.needs_total;
            summary.needs = u64::from(frame.needs.questions)
                .checked_add(u64::from(frame.needs.proposals))
                .expect("two u32 counts fit u64")
                .checked_add(u64::from(frame.needs.escalations))
                .expect("three u32 counts fit u64")
                .checked_add(u64::from(frame.needs.person_tasks))
                .expect("four u32 counts fit u64");
            summary.updates = jig_client::contract::Count {
                value: frame.updates.value.min(limit),
                limited: frame.updates.limited || frame.updates.value > limit,
            };
            summary.project = match address {
                Address::Inbox { project } | Address::Conversations { project } => project,
                Address::Project { project, .. } => Some(project),
                Address::Task { .. } | Address::Deployment => None,
            };
        }
        summary
    }
}
