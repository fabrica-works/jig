//! Domain records translated to generated store families (protocol/store.md, 5.1).
pub mod accounts;
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
    pub core: crate::core_v4::Limits,
    pub tasks: crate::tasks_v5::Limits,
    pub people: crate::people_v5::Limits,
    pub notes: crate::notes_v1::Limits,
}

#[must_use]
pub fn limits(domain: &domain::Limits) -> Option<Limits> {
    let core::Limits { core, tasks, notes } = core::limits(domain)?;
    Some(Limits { core, tasks, notes, people: people::limits(domain)? })
}

/// The configured models' names in the configuration's order: the people family keeps a role's
/// models by name (protocol/store.md, 5.1).
#[must_use]
pub fn names(models: &[domain::Model]) -> Box<[Box<[u8]>]> {
    let mut names = skein_lib::List::with_capacity(u32::try_from(models.len()).expect("configured models fit u32"));
    for model in models {
        names.push(model.name.clone()).expect("one name per model");
    }
    names.into_boxed()
}

/// Encode a record through its owning family; `names` are [`names`] of today's configuration.
#[must_use]
pub fn encode(row: &domain::Record, limits: &Limits, names: &[Box<[u8]>]) -> Option<Box<[u8]>> {
    match row {
        domain::Record::Core(row) => core::encode(row, &core_limits(limits)).ok(),
        domain::Record::Tasks(row) => tasks::encode(row, &limits.tasks, names).ok(),
        domain::Record::People(row) => people::encode(row, &limits.people, names).ok(),
        domain::Record::Notes(row) => notes::encode(row, &limits.notes).ok(),
        domain::Record::Accounts(row) => accounts::encode(row).ok(),
    }
}

/// The key's child selects the one decoder for its value; `names` are [`names`] of today's
/// configuration.
#[must_use]
pub fn decode(key: &domain::Key, bytes: &[u8], limits: &Limits, names: &[Box<[u8]>]) -> Option<domain::Record> {
    match key {
        domain::Key::Core(_) => Some(domain::Record::Core(core::decode(bytes, &core_limits(limits)).ok()?)),
        domain::Key::Tasks(_) => Some(domain::Record::Tasks(tasks::decode(bytes, &limits.tasks, names).ok()?)),
        domain::Key::People(_) => Some(domain::Record::People(people::decode(bytes, &limits.people, names).ok()?)),
        domain::Key::Notes(_) => Some(domain::Record::Notes(notes::decode(bytes, &limits.notes).ok()?)),
        domain::Key::Accounts(_) => Some(domain::Record::Accounts(accounts::decode(bytes).ok()?)),
    }
}

pub(crate) fn core_limits(limits: &Limits) -> core::Limits {
    core::Limits { core: limits.core, tasks: limits.tasks, notes: limits.notes }
}
