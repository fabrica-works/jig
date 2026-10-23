//! The test application's store family (protocol/store.md, 12 and 15).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
use alloc::boxed::Box;
use jig_test_connector as connector;
pub mod keys;
pub mod records;
pub mod route;
#[rustfmt::skip]
#[path = "generated/v2.rs"]
pub mod v2;
/// The test application's durable format.
pub const STORE_VERSION: u32 = 2;
/// A record outside its format or configured bounds.
#[derive(Debug, PartialEq, Eq)]
pub struct Undecodable;
/// One connector's live rows in the application's key space.
#[must_use]
pub fn live(connector: u16) -> skein_kv::Range {
    skein_kv::Range::prefix(&u64::from(connector).to_be_bytes())
}
/// Checked largest encodings supplied to store admission.
#[must_use]
pub fn application(limits: &connector::Limits) -> jig_protocol_store::sizes::Application {
    let bounds = bounds(limits).expect("connector bounds fit schema");
    let path =
        limits.path_segments.saturating_mul(limits.segment_bytes.saturating_mul(2).saturating_add(3)).saturating_add(1);
    jig_protocol_store::sizes::Application {
        version: STORE_VERSION,
        key: path.saturating_add(17).max(83),
        value: u32::try_from(v2::Row::worst_case_bytes(&bounds).expect("checked codec bounds"))
            .expect("row fits store length"),
    }
}
fn bounds(limits: &connector::Limits) -> Option<v2::Limits> {
    let mut bounds = v2::CEILINGS;
    bounds.record_procedure_delegate_outcome_revisions = limits.tasks;
    bounds.segment_bytes = limits.segment_bytes;
    bounds.resource_path_segments = limits.path_segments;
    bounds.effect_resources = limits.resources_per_effect;
    bounds.record_task_resources = limits.resources_per_task;
    bounds.record_made_resources = limits.resources_per_effect;
    v2::check_limits(&bounds).ok()?;
    Some(bounds)
}
#[cfg(test)]
mod tests;

/// Root outputs served by this transport.
#[derive(Debug)]
pub enum StoreOut {
    Commit { number: u64, writes: Box<[jig_test_domain::Write]> },
    Restart { step: jig_domain_core::RestartStep, census: Option<jig_domain_core::Census> },
    ReadLoad { load: skein_lib::Token, range: Box<jig_domain_core::reads::Range> },
    NotesLoad { owner: skein_lib::Token, range: jig_domain_notes::Range },
    Now(jig_domain_core::Now),
}
/// A restored row, a root terminal or the next store request.
#[derive(Debug)]
#[expect(clippy::large_enum_variant, reason = "store replies carry the core's bounded proposal event whole")]
pub enum Up {
    Restore(jig_test_domain::Record),
    Root(jig_test_domain::Event),
    Ask(jig_protocol_store::Request),
}
