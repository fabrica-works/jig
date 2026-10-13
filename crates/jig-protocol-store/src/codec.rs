//! Domain records translated to generated store families (protocol/store.md, 5.1).
pub mod component;
pub mod core;
pub mod notes;
pub mod people;
mod pieces;
pub mod tasks;
use alloc::boxed::Box;
use jig_domain_core as domain;

/// All store families' bounds, validated once by their owners.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub core: crate::core_v1::Limits,
    pub tasks: crate::tasks_v2::Limits,
    pub people: crate::people_v2::Limits,
    pub notes: crate::notes_v1::Limits,
}

#[must_use]
pub fn limits(domain: &domain::Limits) -> Option<Limits> {
    let core::Limits { core, tasks, notes } = core::limits(domain)?;
    Some(Limits { core, tasks, notes, people: people::limits(domain)? })
}

/// Encode a record through its owning family.
#[must_use]
pub fn encode(row: &domain::Record, limits: &Limits) -> Option<Box<[u8]>> {
    match row {
        domain::Record::Core(row) => core::encode(row, &core_limits(limits)).ok(),
        domain::Record::Tasks(row) => tasks::encode(row, &limits.tasks).ok(),
        domain::Record::People(row) => people::encode(row, &limits.people).ok(),
        domain::Record::Notes(row) => notes::encode(row, &limits.notes).ok(),
    }
}

/// The key's child selects the one decoder for its value.
#[must_use]
pub fn decode(key: &domain::Key, bytes: &[u8], limits: &Limits) -> Option<domain::Record> {
    match key {
        domain::Key::Core(_) => Some(domain::Record::Core(core::decode(bytes, &core_limits(limits)).ok()?)),
        domain::Key::Tasks(_) => Some(domain::Record::Tasks(tasks::decode(bytes, &limits.tasks).ok()?)),
        domain::Key::People(_) => Some(domain::Record::People(people::decode(bytes, &limits.people).ok()?)),
        domain::Key::Notes(_) => Some(domain::Record::Notes(notes::decode(bytes, &limits.notes).ok()?)),
    }
}

pub(crate) fn core_limits(limits: &Limits) -> core::Limits {
    core::Limits { core: limits.core, tasks: limits.tasks, notes: limits.notes }
}
