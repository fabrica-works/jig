//! Fixed pacing capacities (`domain/engine.md`, 11 and 13).
//! There is no additional per-subject limit (`QUESTIONS.md`, item 17).
use crate::facts::Fact;
use crate::watch::Watcher;
use skein_lib::{Id, List, Map, Queue, Slab, Token};

/// Capacities fixed when the child is constructed.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    /// Runs whose provisional text can be followed.
    pub runs: u32,
    /// Open watches.
    pub watchers: u32,
    /// Open watches belonging to one opaque party.
    pub watchers_per_party: u32,
    /// Subjects named by one change event.
    pub subjects_per_change: u32,
    /// Retained provisional bytes per followed run.
    pub text_bytes: u32,
    /// Best effort observations waiting to be drained.
    pub facts: u32,
}
pub(crate) fn slots(limits: &Limits) -> Option<u32> {
    limits.watchers.checked_mul(2)
}

/// Maximum pacing memory, including one bounded change event and scan in hand.
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    if limits.runs == 0
        || limits.watchers == 0
        || limits.watchers_per_party == 0
        || limits.subjects_per_change == 0
        || limits.text_bytes == 0
    {
        return None;
    }
    Slab::<Watcher>::worst_case(slots(limits)?)?
        .checked_add(Map::<Token, Id<Watcher>>::worst_case(limits.watchers)?)?
        .checked_add(Queue::<Fact>::worst_case(limits.facts)?)?
        .checked_add(List::<Token>::worst_case(limits.subjects_per_change)?)?
        .checked_add(List::<Id<Watcher>>::worst_case(limits.watchers)?)
}
