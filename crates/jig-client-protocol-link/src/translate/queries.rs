#![expect(clippy::manual_map, reason = "step code matches optional query facts without closures")]
//! The fixed task query and the task owner's whole lines.
use super::{documents, facts, w};
use crate::Limits;
use jig_client_domain::contract as c;
use skein_lib::Wall;

pub(super) fn query(value: c::TaskQuery, limits: &w::Limits) -> Result<w::TaskQuery, w::Problem> {
    let requester = match value.requester {
        None => None,
        Some(c::OwnerRequester::Task(value)) => {
            Some(w::OwnerRequester::Task(w::OwnerRequesterTask::new(limits, w::OwnerRequesterTaskParts { value })?))
        }
        Some(c::OwnerRequester::Person(value)) => Some(w::OwnerRequester::Person(w::OwnerRequesterPerson::new(
            limits,
            w::OwnerRequesterPersonParts { value },
        )?)),
        Some(c::OwnerRequester::Deployment { project }) => Some(w::OwnerRequester::Deployment(
            w::OwnerRequesterDeployment::new(limits, w::OwnerRequesterDeploymentParts { project })?,
        )),
    };
    w::TaskQuery::new(
        limits,
        w::TaskQueryParts {
            project: value.project,
            requester,
            root: value.root,
            parent: value.parent,
            label: value.label,
            phase: match value.phase {
                None => None,
                Some(v) => Some(match v {
                    c::PhaseClass::Working => w::PhaseClass::Working,
                    c::PhaseClass::Waiting => w::PhaseClass::Waiting,
                    c::PhaseClass::Held => w::PhaseClass::Held,
                    c::PhaseClass::Closing => w::PhaseClass::Closing,
                    c::PhaseClass::Ended => w::PhaseClass::Ended,
                }),
            },
            executor: match value.executor {
                None => None,
                Some(v) => Some(match v {
                    c::ExecutorKind::Agent => w::ExecutorKind::Agent,
                    c::ExecutorKind::Procedure => w::ExecutorKind::Procedure,
                    c::ExecutorKind::Person => w::ExecutorKind::Person,
                }),
            },
            tracked: value.tracked,
            order: match value.order {
                c::QueryOrder::Number => w::QueryOrder::Number,
                c::QueryOrder::Changed => w::QueryOrder::Changed,
                c::QueryOrder::Activity => w::QueryOrder::Activity,
                c::QueryOrder::Priority => w::QueryOrder::Priority,
                c::QueryOrder::Ended => w::QueryOrder::Ended,
            },
        },
    )
}
pub(super) fn flags(value: &w::TaskFlags) -> c::TaskFlags {
    c::TaskFlags {
        conversation: value.conversation(),
        listed: value.listed(),
        retention: match value.retention() {
            w::Retention::Kept => c::Retention::Kept,
            w::Retention::ForgottenOnceRead => c::Retention::ForgottenOnceRead,
        },
    }
}
pub(super) fn waits(value: &w::WaitsOn) -> c::WaitsOn {
    match value {
        w::WaitsOn::Party(v) => c::WaitsOn::Party {
            whom: match v.whom() {
                w::Whom::Person(v) => c::Whom::Person(v.value()),
                w::Whom::Role(v) => c::Whom::Role { project: v.project(), role: v.role() },
            },
            entries: v.entries(),
            task: v.task(),
            kind: match v.kind() {
                w::WaitedFor::Question => c::WaitedFor::Question,
                w::WaitedFor::Proposal => c::WaitedFor::Proposal,
                w::WaitedFor::Escalation => c::WaitedFor::Escalation,
                w::WaitedFor::PersonTask => c::WaitedFor::PersonTask,
            },
        },
        w::WaitsOn::Dependencies(value) => {
            c::WaitsOn::Dependencies { pending: value.pending().clone(), count: value.count() }
        }
        w::WaitsOn::Delegates => c::WaitsOn::Delegates,
        w::WaitsOn::News => c::WaitsOn::News,
        w::WaitsOn::Nothing => c::WaitsOn::Nothing,
    }
}
pub(super) fn line(value: &w::TaskLine, limits: &Limits) -> Result<c::TaskLine, w::Problem> {
    Ok(c::TaskLine {
        chip: documents::chip_fields(value.chip(), limits)?,
        project: value.project(),
        requester: facts::owner_requester(value.requester(), limits),
        tracked: *value.tracked(),
        label: value.label(),
        flags: flags(value.flags()),
        changed: Wall::from_nanos(value.changed()),
        last_activity: Wall::from_nanos(value.last_activity()),
        ended_at: match *value.ended_at() {
            None => None,
            Some(v) => Some(Wall::from_nanos(v)),
        },
        budget: facts::owner_budget(value.budget(), limits),
        waits_on: waits(value.waits_on()),
        phases: documents::owner_phase_counts_fields(value.phases(), limits),
        held_below: value.held_below(),
        conversation: match value.conversation() {
            None => None,
            Some(v) => {
                Some(c::ConversationRead { seen: v.seen(), unread: documents::count_fields(v.unread(), limits) })
            }
        },
    })
}
