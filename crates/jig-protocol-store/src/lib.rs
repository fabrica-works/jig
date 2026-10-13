//! The engine's store component (protocol/store.md, sections 2 and 3).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub mod boundary;
pub mod codec;
mod commits;
pub mod format;
mod gets;
pub mod historical;
pub mod intervals;
mod loads;
mod open;
mod pages;
mod reads;
mod restore;
pub mod sizes;
use alloc::boxed::Box;
use skein_lib::{Env, Queue, Time, Wall};

/// The store component's bounds (protocol/store.md, section 13).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Limits {
    pub core: jig_domain_core::Limits,
    pub store: skein_kv::Limits,
    pub secrets: skein_kv::Limits,
    pub journal: skein_lib::JournalLimits,
    /// Point gets available to a party page's joins (store.md, 13).
    pub joins: u32,
}

/// Stores made by the caller, kept across a world's crash (store.md, 14).
#[derive(Debug)]
pub struct Stores {
    pub store: skein_kv::Store,
    pub secrets: skein_kv::Store,
}

/// The component's open and stopped states (store.md, 6 and 10).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    Closed,
    Ready { next: u64, format_due: bool },
    Stopped,
}

#[derive(Debug)]
pub struct Store {
    limits: Limits,
    application: sizes::Application,
    codec: codec::Limits,
    stores: Stores,
    phase: Phase,
}

impl Store {
    #[must_use]
    pub fn over(limits: &Limits, application: sizes::Application, stores: Stores) -> Store {
        assert!(limits.store.is_usable() && limits.secrets.is_usable(), "both stores' limits are usable");
        let codec = codec::limits(&limits.core).expect("the domain limits fit their families' ceilings");
        Store { limits: *limits, application, codec, stores, phase: Phase::Closed }
    }

    #[must_use]
    pub fn into_stores(self) -> Stores {
        self.stores
    }
}

/// One terminal from a memory step.
pub const MAX_OUT_DOWN: u32 = 1;

/// Owned stores, component state and bounded temporary rows and operations.
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    let state = u64::try_from(size_of::<Store>()).ok()?;
    let rows = u64::from(limits.store.value).checked_add(u64::from(limits.store.key))?.checked_mul(2)?;
    let codec = codec::limits(&limits.core)?;
    let decoded = core_v1::Row::worst_case_heap(&codec.core)?
        .checked_add(tasks_v1::Row::worst_case_heap(&codec.tasks)?)?
        .checked_add(people_v1::Row::worst_case_heap(&codec.people)?)?
        .checked_add(notes_v1::Row::worst_case_heap(&codec.notes)?)?
        .checked_add(u64::try_from(size_of::<jig_domain_tasks::TaskRecord>()).ok()?)?
        .checked_mul(2)?;
    // One byte per encoded row bounds carrier counts. Count wire values and
    // their translated records together, including the census's fetched task.
    let page = u64::from(limits.store.page.bytes)
        .checked_mul(decoded.checked_add(u64::from(limits.store.key))?)?
        .checked_add(u64::from(limits.store.page.bytes))?
        .checked_add(skein_lib::List::<pages::WalkRow>::worst_case(limits.store.page.bytes)?)?
        .checked_add(skein_lib::List::<Row>::worst_case(limits.store.page.bytes)?)?;
    skein_kv::worst_case(&limits.store)?
        .checked_add(skein_kv::worst_case(&limits.secrets)?)?
        .checked_add(state)?
        .checked_add(rows)?
        .checked_add(page)?
        .checked_add(decoded.checked_mul(u64::from(limits.joins))?)?
        .checked_add(skein_lib::List::<Box<jig_domain_tasks::TaskRecord>>::worst_case(limits.joins)?)?
        .checked_add(skein_lib::List::<u64>::worst_case(limits.core.tasks.depth)?)?
        .checked_add(skein_lib::Set::<u64>::worst_case(limits.joins)?)?
        .checked_add(u64::from(limits.store.commit))?
        .checked_add(skein_lib::List::<skein_kv::Op>::worst_case(limits.journal.writes.checked_add(1)?)?)?
        .checked_add(Queue::<skein_kv::Event>::worst_case(skein_kv::MAX_OUT.above)?)?
        .checked_add(Queue::<skein_io::file::Request>::worst_case(skein_kv::MAX_OUT.below)?)?
        .checked_add(Queue::<Event>::worst_case(MAX_OUT_DOWN)?)
}

/// The memory component's requests (store.md, section 10).
pub fn down(store: &mut Store, request: Request, above: &mut Queue<Event>) {
    if store.phase == Phase::Stopped {
        return;
    }
    match request {
        Request::Open { owner, root: _ } => open::open(store, owner, above),
        Request::Commit { number, writes } => commits::commit(store, number, writes, above),
        Request::Restore { owner, range, after, rows, bytes } => {
            restore::restore(store, owner, range, after, rows, bytes, above);
        }
        Request::Load { owner, range, after, rows, bytes } => {
            loads::load(store, owner, range, after, rows, bytes, above);
        }
        Request::Get { owner, key } => gets::get(store, owner, key, above),
        Request::Read { owner, range } => reads::read(store, owner, range, above),
        // Each remaining handler joins in its owning increment, before routing uses it.
        Request::Transcript { .. } | Request::Secrets { .. } => {}
    }
}

fn memory_env(limits: skein_kv::Limits) -> Env<skein_kv::Limits> {
    // Memory gets and commits use no clock or file operation.
    Env { now: Time::ZERO, wall: Wall::EPOCH, limits }
}

#[rustfmt::skip]
#[path = "generated/component_v1.rs"]
pub mod component_v1;
#[rustfmt::skip]
#[path = "generated/notes_v1.rs"]
pub mod notes_v1;
#[rustfmt::skip]
#[path = "generated/people_v1.rs"]
pub mod people_v1;
#[rustfmt::skip]
#[path = "generated/tasks_v1.rs"]
pub mod tasks_v1;
#[rustfmt::skip]
#[path = "generated/core_v1.rs"]
pub mod core_v1;
pub mod keys;
pub use boundary::{
    Cursor, Event, Family, Get, Load, OpenRefusal, Request, Restore, Row, SecretOwner, Secrets, TranscriptRead, Write,
};

#[cfg(test)]
mod tests;
