//! Bounded content-free observations (`domain/engine.md`, 11).
//! Dropping a fact changes no watch or delivery.
use crate::{End, Refusal};
use skein_lib::Queue;

/// One observation of pacing or admission.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Fact {
    /// A watch was admitted.
    Watching,
    /// A watch was refused at a bound.
    Refused { refusal: Refusal },
    /// A document was requested.
    Making,
    /// A document was delivered.
    Delivered,
    /// Subjects changed for this many watches.
    Changed { watchers: u32 },
    /// A watch ended.
    Ended { end: End },
}

/// The best effort observations waiting for their parent.
#[derive(Debug)]
pub(crate) struct Facts {
    queue: Queue<Fact>,
    lost: u64,
}
impl Facts {
    pub(crate) fn new(capacity: u32) -> Self {
        Self { queue: Queue::with_capacity(capacity), lost: 0 }
    }
    pub(crate) fn push(&mut self, fact: Fact) {
        if self.queue.try_push(fact).is_err() {
            self.lost = self.lost.saturating_add(1);
        }
    }
    pub(crate) fn pop(&mut self) -> Option<Fact> {
        self.queue.pop()
    }
    pub(crate) fn lost(&self) -> u64 {
        self.lost
    }
}
