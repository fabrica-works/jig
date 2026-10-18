//! Shared task phase facts for the independent joint translation.
use jig_client_domain::contract as client;
use jig_domain_core::documents;
#[must_use]
pub fn phases(value: documents::PhaseCounts) -> client::OwnerPhaseCounts {
    client::OwnerPhaseCounts {
        waiting: value.waiting,
        idle: value.idle,
        starting: value.starting,
        running: value.running,
        backing_off: value.backing_off,
        closing: value.closing,
        held: value.held,
        done: value.done,
        failed: value.failed,
        cancelled: value.cancelled,
    }
}
#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    pub(crate) fn limits() -> jig_client_view::Limits {
        jig_client_view::Limits { rows: 4, text: 200, document_bytes: 100_000, ..crate::test::limits().jig }
    }
    pub(crate) fn phase_source() -> documents::PhaseCounts {
        documents::PhaseCounts {
            waiting: 1,
            idle: 2,
            starting: 3,
            running: 4,
            backing_off: 5,
            closing: 6,
            held: 7,
            done: 8,
            failed: 9,
            cancelled: u32::MAX,
        }
    }
    pub(crate) fn task_source() -> documents::TaskLine {
        let mut chip = super::super::chips::tests::source_chip();
        chip.title = documents::Text { bytes: Box::from(b"~\0".as_slice()), cut: true };
        documents::TaskLine {
            budget: chip.details.budget,
            chip,
            project: u32::MAX,
            requester: jig_domain_tasks::Party::Person(1),
            tracked: None,
            label: 1,
            flags: jig_domain_tasks::Flags {
                conversation: true,
                listed: false,
                retention: jig_domain_tasks::Retention::ForgottenOnceRead,
            },
            changed: skein_lib::Wall::EPOCH,
            last_activity: skein_lib::Wall::EPOCH,
            ended_at: None,
            waits_on: documents::WaitsOn::Nothing,
            phases: phase_source(),
            held_below: 0,
            conversation: None,
        }
    }
}
