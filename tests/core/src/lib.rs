//! The core's scripted domain world (`domain/engine.md`, section 15).

#![forbid(unsafe_code)]

mod charters;
pub mod world;

mod application_observations;
pub mod client_peer;
pub mod effects;
mod historical;
pub mod peers;
pub mod store_peer;

/// The testing application's store, with the fake's opaque row parameters.
pub type Store = jig_fake_store::Store<jig_test_domain::Key, jig_test_domain::Record>;

/// Translate one testing-root transaction without interpreting its records.
#[must_use]
pub fn store_writes(
    writes: Vec<jig_test_domain::Write>,
) -> Box<[jig_fake_store::Write<jig_test_domain::Key, jig_test_domain::Record>]> {
    writes
        .into_iter()
        .map(|write| match write {
            jig_test_domain::Write::Save(row) => jig_fake_store::Write::Save { key: row.key(), row },
            jig_test_domain::Write::Erase(key) => jig_fake_store::Write::Erase(key),
        })
        .collect()
}

pub mod observations;
pub mod referee;

pub mod faults;

pub mod watches;

pub mod reads;

pub mod stories;

pub mod fixtures;
