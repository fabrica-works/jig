//! Container capacities and byte bounds owned by the client.
//! Every retained container is sized from these at new; jig 21–23 add their
//! own entities. See client/domain/client.md, 14.
use crate::{
    contract::{Charter, Frame, ProjectRole},
    domain::Timer,
    store::Record,
    watches::Watch,
};
use core::mem::size_of;
use skein_lib::{Deadlines, Duration, Id, List, Map, Slab};

/// Backoff spans: double the first up to most, with seeded bounded jitter.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Backoff {
    pub first: Duration,
    pub most: Duration,
}

/// Configured capacities, payload bounds and durations for the client spine.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Limits {
    pub watches: u32,
    pub notices: u32,
    pub notice_life: Duration,
    pub linger: Duration,
    pub save_delay: Duration,
    pub intents: u32,
    pub fields: u32,
    pub providers: u32,
    pub objects: u32,
    pub sources: u32,
    pub reads: u32,
    pub parts: u32,
    /// Live delegates that one close confirmation may dispose of.
    pub delegates: u32,
    /// Whole payload budget enforced by the decoding owner: boxes (including
    /// the root box), list capacities, texts and nested documents.
    pub document_bytes: u64,
    pub records: u32,
    /// Distinct scopes of restored requests counted until their retirement.
    pub kept: u32,
    /// Per-task composers kept across navigation and restart.
    pub drafts: u32,
    pub projects: u32,
    pub roles: u32,
    pub charters: u32,
    pub text: u32,
    pub rows: u32,
    /// Whole pending or sent word snapshots kept for the displayed conversation.
    pub conversation_words: u32,
    /// Maximum configured navigation destinations.
    pub navigation_entries: u32,
    /// The largest unread update count displayed without a lower-bound mark.
    pub updates_count: u32,
    pub backoff: Backoff,
    pub heartbeat: Duration,
}

/// The heap held by the implemented spine, including frame and record payloads.
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    if limits.watches == 0
        || limits.notice_life == Duration::ZERO
        || limits.linger == Duration::ZERO
        || limits.save_delay == Duration::ZERO
        || limits.heartbeat == Duration::ZERO
        || limits.backoff.first == Duration::ZERO
        || limits.backoff.first > limits.backoff.most
    {
        return None;
    }
    limits
        .watches
        .checked_mul(2)?
        .checked_add(limits.objects)?
        .checked_add(limits.parts.checked_mul(2)?)?
        .checked_add(limits.records)?
        .checked_add(16)?;
    limits.objects.checked_mul(2)?;
    let timer_capacity = timers(limits)?;
    if limits.sources == 0 || limits.conversation_words < limits.intents {
        return None;
    }
    let mut total = Slab::<Watch>::worst_case(limits.watches)?;
    total = total.checked_add(List::<crate::contract::ObjectKey>::worst_case(tree_names(limits)?)?)?;
    total = total.checked_add(crate::application::worst_case(limits)?)?;
    total = total.checked_add(crate::conversation::worst_case(limits)?)?;
    total = total.checked_add(List::<crate::pages::navigation::Destination>::worst_case(limits.navigation_entries)?)?;
    total = total.checked_add(crate::pages::conversations::worst_case(limits)?)?;
    total = total.checked_add(crate::pages::task::worst_case(limits)?)?;
    total = total.checked_add(crate::pages::project::worst_case(limits)?)?;
    total = total.checked_add(crate::pages::deployment::worst_case(limits)?)?;
    total = total.checked_add(crate::intents::worst_case(limits)?)?;
    total = total.checked_add(crate::notices::worst_case(limits)?)?;
    total = total.checked_add(crate::fields::worst_case(limits)?)?;
    total = total.checked_add(crate::drafts::worst_case(limits)?)?;
    total = total.checked_add(List::<u64>::worst_case(limits.records)?)?;
    total = total.checked_add(Map::<u64, u32>::worst_case(limits.kept)?)?;
    total = total.checked_add(crate::confirm::worst_case(limits)?)?;
    total = total.checked_add(List::<crate::objects::Context>::worst_case(limits.rows)?)?;
    total = total.checked_add(crate::signin::worst_case(limits)?)?;
    total = total.checked_add(crate::objects::worst_case(limits)?)?;
    total = total.checked_add(Slab::<crate::reads::Reading>::worst_case(limits.reads)?)?;
    total = total.checked_add(Map::<Id<crate::reads::Reading>, ()>::worst_case(limits.reads)?)?;
    total = total.checked_add(u64::from(limits.reads).checked_mul(limits.document_bytes)?)?;
    total = total.checked_add(crate::windows::worst_case(limits)?)?;
    total = total.checked_add(Map::<Id<Watch>, ()>::worst_case(limits.watches)?)?;
    total = total.checked_add(Deadlines::<Timer>::worst_case(timer_capacity)?)?;
    total = total.checked_add(List::<Record>::worst_case(limits.records)?.checked_mul(2)?)?;
    total = total.checked_add(u64::from(limits.records).checked_mul(u64::from(limits.text))?)?;
    total = total.checked_add(u64::try_from(size_of::<Frame>()).ok()?)?;
    total = total.checked_add(u64::from(limits.text).checked_mul(2)?)?;
    total = total.checked_add(List::<ProjectRole>::worst_case(limits.projects)?)?;
    total = total.checked_add(List::<crate::contract::OfferedCharters>::worst_case(limits.projects)?)?;
    let charters = List::<Charter>::worst_case(limits.charters)?
        .checked_add(u64::from(limits.charters).checked_mul(u64::from(limits.text))?)?
        .checked_mul(2)?;
    let project = charters.checked_add(u64::from(limits.text))?;
    total = total.checked_add(u64::from(limits.projects).checked_mul(project)?)?;
    // Chips subjects own at most one configured task-number list each.
    total.checked_add(u64::from(limits.watches).checked_mul(List::<u64>::worst_case(limits.rows)?)?)
}

/// One deadline per watch, lingering object, visible notice and saved record,
/// plus the minute tick. Checked before any container is allocated.
#[must_use]
pub fn timers(limits: &Limits) -> Option<u32> {
    limits.watches.checked_add(limits.objects)?.checked_add(limits.notices)?.checked_add(limits.records)?.checked_add(1)
}

/// One task and every bounded proposal name per retained tree row.
pub(crate) fn tree_names(limits: &Limits) -> Option<u32> {
    limits.rows.checked_mul(limits.rows.checked_add(1)?)
}
