#![expect(clippy::manual_map, reason = "step code matches optional query facts without closures")]
//! Task queries and lines preserve the engine's keys and supplied order.
use super::{chips, live, me, owner, w};
use jig_domain_core::documents::{self as d, WaitsOn};
use jig_domain_people as p;
use jig_domain_tasks as t;
use skein_lib::{List, Writer};

pub(super) fn query(value: w::TaskQuery) -> Option<d::TaskQuery> {
    let value = value.into_parts();
    let requester = match value.requester {
        None => None,
        Some(w::OwnerRequester::Task(v)) => Some(t::Party::Task(v.value())),
        Some(w::OwnerRequester::Person(v)) => Some(t::Party::Person(v.value())),
        Some(w::OwnerRequester::Deployment(_)) => return None,
    };
    Some(d::TaskQuery {
        project: value.project,
        requester,
        root: value.root,
        parent: value.parent,
        label: value.label,
        phase: match value.phase {
            None => None,
            Some(v) => Some(match v {
                w::PhaseClass::Working => d::PhaseClass::Working,
                w::PhaseClass::Waiting => d::PhaseClass::Waiting,
                w::PhaseClass::Held => d::PhaseClass::Held,
                w::PhaseClass::Closing => d::PhaseClass::Closing,
                w::PhaseClass::Ended => d::PhaseClass::Ended,
            }),
        },
        executor: match value.executor {
            None => None,
            Some(v) => Some(match v {
                w::ExecutorKind::Agent => d::ExecutorKind::Agent,
                w::ExecutorKind::Procedure => d::ExecutorKind::Procedure,
                w::ExecutorKind::Person => d::ExecutorKind::Person,
            }),
        },
        tracked: value.tracked,
        order: match value.order {
            w::QueryOrder::Number => d::QueryOrder::Number,
            w::QueryOrder::Changed => d::QueryOrder::Changed,
            w::QueryOrder::Activity => d::QueryOrder::Activity,
            w::QueryOrder::Priority => d::QueryOrder::Priority,
            w::QueryOrder::Ended => d::QueryOrder::Ended,
        },
    })
}
pub(super) fn waits(value: WaitsOn, limits: &w::Limits) -> Result<w::WaitsOn, w::Problem> {
    Ok(match value {
        WaitsOn::Party { whom, entries, task, kind } => w::WaitsOn::Party(w::WaitsOnParty::new(
            limits,
            w::WaitsOnPartyParts {
                whom: match whom {
                    p::Whom::Person(value) => {
                        w::Whom::Person(w::WhomPerson::new(limits, w::WhomPersonParts { value })?)
                    }
                    p::Whom::Role { project, role } => {
                        w::Whom::Role(w::WhomRole::new(limits, w::WhomRoleParts { project, role })?)
                    }
                },
                entries,
                task,
                kind: match kind {
                    p::EntryKind::Question { .. } => w::WaitedFor::Question,
                    p::EntryKind::Proposal { .. } => w::WaitedFor::Proposal,
                    p::EntryKind::Escalation { .. } => w::WaitedFor::Escalation,
                    p::EntryKind::PersonTask => w::WaitedFor::PersonTask,
                    p::EntryKind::Reply { .. } | p::EntryKind::ChatReply { .. } => {
                        return Err(super::problem(skein_codec::Reason::Tag));
                    }
                },
            },
        )?),
        WaitsOn::Dependencies { pending, count } => w::WaitsOn::Dependencies(w::WaitsOnDependencies::new(
            limits,
            w::WaitsOnDependenciesParts { pending, count },
        )?),
        WaitsOn::Delegates => w::WaitsOn::Delegates,
        WaitsOn::News => w::WaitsOn::News,
        WaitsOn::Nothing => w::WaitsOn::Nothing,
    })
}
pub(super) fn line(value: &d::TaskLine, limits: &w::Limits) -> Result<w::TaskLine, w::Problem> {
    w::TaskLine::new(
        limits,
        w::TaskLineParts {
            chip: chips::chip(&value.chip, limits)?,
            project: value.project,
            requester: owner::requester(value.requester, limits)?,
            tracked: value.tracked,
            label: value.label,
            flags: me::flags(value.flags, limits)?,
            changed: value.changed.as_nanos(),
            last_activity: value.last_activity.as_nanos(),
            ended_at: match value.ended_at {
                None => None,
                Some(at) => Some(at.as_nanos()),
            },
            budget: owner::budget(&value.budget, limits)?,
            waits_on: waits(value.waits_on.clone(), limits)?,
            phases: live::phases(value.phases, limits)?,
            held_below: value.held_below,
            conversation: match value.conversation {
                None => None,
                Some(v) => Some(w::ConversationRead::new(
                    limits,
                    w::ConversationReadParts { seen: v.seen, unread: live::count(v.unread, limits)? },
                )?),
            },
        },
    )
}
pub(crate) fn lines(value: &d::Lines, limits: &w::Limits) -> Result<w::Tasks, w::Problem> {
    let mut rows = List::with_capacity(value.lines.len());
    for row in &value.lines {
        let row = line(row, limits)?;
        let mut writer = Writer::new(usize::try_from(row.measure()).expect("u32 measure fits"));
        row.encode(&mut writer).expect("measured task line");
        rows.push(writer.finish()).expect("same line count");
    }
    let rows = super::pieces(&rows, limits.tasks_rows, w::Path::TasksRows)?;
    w::Tasks::new(limits, w::TasksParts { rows, more: value.next.is_some() })
}
