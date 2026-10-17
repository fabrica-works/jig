//! The core's query and line facts, preserved at the joint boundary.
use super::{chips, owner, results, rows};
use jig_client_domain::contract as c;
use jig_client_view::Limits;
use jig_domain_core::documents as d;
use jig_domain_people as p;
use jig_domain_tasks as t;

pub(crate) fn query(value: &c::TaskQuery) -> Result<d::TaskQuery, super::watches::Invalid> {
    let requester = match value.requester {
        None => None,
        Some(c::OwnerRequester::Task(v)) => Some(t::Party::Task(v)),
        Some(c::OwnerRequester::Person(v)) => Some(t::Party::Person(v)),
        Some(c::OwnerRequester::Deployment { .. }) => return Err(super::watches::Invalid::DeploymentRequester),
    };
    Ok(d::TaskQuery {
        project: value.project,
        requester,
        root: value.root,
        parent: value.parent,
        label: value.label,
        phase: value.phase.map(|v| match v {
            c::PhaseClass::Working => d::PhaseClass::Working,
            c::PhaseClass::Waiting => d::PhaseClass::Waiting,
            c::PhaseClass::Held => d::PhaseClass::Held,
            c::PhaseClass::Closing => d::PhaseClass::Closing,
            c::PhaseClass::Ended => d::PhaseClass::Ended,
        }),
        executor: value.executor.map(|v| match v {
            c::ExecutorKind::Agent => d::ExecutorKind::Agent,
            c::ExecutorKind::Procedure => d::ExecutorKind::Procedure,
            c::ExecutorKind::Person => d::ExecutorKind::Person,
        }),
        tracked: value.tracked,
        order: match value.order {
            c::QueryOrder::Number => d::QueryOrder::Number,
            c::QueryOrder::Changed => d::QueryOrder::Changed,
            c::QueryOrder::Activity => d::QueryOrder::Activity,
            c::QueryOrder::Priority => d::QueryOrder::Priority,
            c::QueryOrder::Ended => d::QueryOrder::Ended,
        },
    })
}
pub(super) fn flags(value: t::Flags) -> c::TaskFlags {
    c::TaskFlags {
        conversation: value.conversation,
        listed: value.listed,
        retention: match value.retention {
            t::Retention::Kept => c::Retention::Kept,
            t::Retention::ForgottenOnceRead => c::Retention::ForgottenOnceRead,
        },
    }
}
pub(super) fn waits(value: d::WaitsOn) -> c::WaitsOn {
    match value {
        d::WaitsOn::Party { whom, entries, task, kind } => c::WaitsOn::Party {
            whom: match whom {
                p::Whom::Person(v) => c::Whom::Person(v),
                p::Whom::Role { project, role } => c::Whom::Role { project, role },
            },
            entries,
            task,
            kind: match kind {
                p::EntryKind::Question { .. } => c::WaitedFor::Question,
                p::EntryKind::Proposal { .. } => c::WaitedFor::Proposal,
                p::EntryKind::Escalation { .. } => c::WaitedFor::Escalation,
                p::EntryKind::PersonTask => c::WaitedFor::PersonTask,
                p::EntryKind::Reply { .. } | p::EntryKind::ChatReply { .. } => {
                    unreachable!("not a waiting party entry")
                }
            },
        },
        d::WaitsOn::Delegates => c::WaitsOn::Delegates,
        d::WaitsOn::News => c::WaitsOn::News,
        d::WaitsOn::Nothing => c::WaitsOn::Nothing,
    }
}
pub(super) fn line(value: &d::TaskLine, limits: &Limits) -> Result<c::TaskLine, owner::Invalid> {
    Ok(c::TaskLine {
        chip: chips::chip(&value.chip, limits)?,
        project: value.project,
        requester: results::requester(value.requester),
        tracked: value.tracked,
        label: value.label,
        flags: flags(value.flags),
        changed: value.changed,
        last_activity: value.last_activity,
        ended_at: value.ended_at,
        budget: owner::budget(&value.budget),
        waits_on: waits(value.waits_on),
        phases: rows::phases(value.phases),
        held_below: value.held_below,
        conversation: value.conversation.map(|v| c::ConversationRead {
            seen: v.seen,
            unread: match v.unread {
                d::Count::Known(value) => c::Count { value, limited: false },
                d::Count::AtLeast(value) => c::Count { value, limited: true },
            },
        }),
    })
}
