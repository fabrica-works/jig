//! Recursive exact Inbox admission (client/domain/contract.md, 5.3).
//! All allocated capacities, links, reply boxes and opaque bytes are charged.
use crate::{Bounds, payload::Payload};
use core::mem::size_of;
use jig_client::contract::{
    InboxDocument, InboxRow, OwnerInbox, OwnerInboxCursor, OwnerInboxEntry, OwnerInboxObject, OwnerInboxPage,
};
impl Payload for OwnerInboxObject {
    fn heap(&self, _: &Bounds) -> Option<u64> {
        Some(0)
    }
}
impl Payload for OwnerInboxCursor {
    fn heap(&self, _: &Bounds) -> Option<u64> {
        Some(0)
    }
}
impl Payload for OwnerInboxEntry {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        self.reply.heap(bounds)?.checked_add(self.task.heap(bounds)?)
    }
}
impl Payload for OwnerInbox {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        self.needs.heap(bounds)?.checked_add(self.updates.heap(bounds)?)
    }
}
impl Payload for OwnerInboxPage {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        self.rows.heap(bounds)?.checked_add(self.before.heap(bounds)?)?.checked_add(self.after.heap(bounds)?)
    }
}
impl Payload for InboxRow {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Entry(value) => value.heap(bounds),
            Self::Owner(value) => value.heap(bounds),
        }
    }
}
impl Payload for InboxDocument {
    fn heap(&self, bounds: &Bounds) -> Option<u64> {
        match self {
            Self::Entry(value) => value.heap(bounds),
            Self::Owner(value) => value.heap(bounds),
        }
    }
}
#[must_use]
pub fn bytes(value: &OwnerInbox, bounds: &Bounds) -> Option<u64> {
    u64::try_from(size_of::<OwnerInbox>()).ok()?.checked_add(value.heap(bounds)?)
}
#[must_use]
pub fn fits(value: &OwnerInbox, bounds: &Bounds) -> bool {
    crate::payload::fits(value, bounds)
}
#[must_use]
pub fn page_bytes(value: &OwnerInboxPage, bounds: &Bounds) -> Option<u64> {
    u64::try_from(size_of::<OwnerInboxPage>()).ok()?.checked_add(value.heap(bounds)?)
}
#[must_use]
pub fn page_fits(value: &OwnerInboxPage, bounds: &Bounds) -> bool {
    crate::payload::fits(value, bounds)
}
#[must_use]
pub fn worst_case(bounds: &Bounds) -> Option<u64> {
    if bounds.document_bytes < u64::try_from(size_of::<OwnerInbox>()).ok()? {
        None
    } else {
        Some(bounds.document_bytes)
    }
}
#[cfg(test)]
mod tests;
