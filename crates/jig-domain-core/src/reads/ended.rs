//! The single index serving a task query's ended group (store.md, 7.3).
use super::{PageRow, Row, vocabulary::row_owner};
use crate::{
    Core, Key, Limits,
    documents::{self, Missing, PhaseClass, TaskQuery},
};
use jig_domain_tasks as tasks;

/// The index selected by the query's keys, in priority order.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EndedPart {
    ByParty { party: u64, label: Option<u32> },
    Delegates { requester: u64 },
    ByProject { project: u32, label: Option<u32> },
    None,
}
/// Choose one ended index; a live-only query has no ended part.
#[must_use]
pub fn ended_part(query: &TaskQuery) -> EndedPart {
    if let Some(phase) = query.phase
        && phase != PhaseClass::Ended
    {
        return EndedPart::None;
    }
    match (query.requester, query.parent) {
        (Some(tasks::Party::Person(party)), _) => EndedPart::ByParty { party, label: query.label },
        (Some(tasks::Party::Task(task)), _) | (None, Some(task)) => EndedPart::Delegates { requester: task },
        (Some(tasks::Party::Deployment { .. }), _) => EndedPart::None,
        (None, None) => match query.project {
            Some(project) => EndedPart::ByProject { project, label: query.label },
            None => EndedPart::None,
        },
    }
}
/// Validate an examined position against the chosen prefix.
#[must_use]
pub fn position(query: &TaskQuery, key: &Key) -> bool {
    match (ended_part(query), key) {
        (
            EndedPart::ByParty { party, label },
            Key::Tasks(tasks::Key::EndedByParty { party: owner, label: value, .. }),
        ) => party == *owner && (label.is_none() || label == Some(*value)),
        (EndedPart::Delegates { requester }, Key::Tasks(tasks::Key::EndedDelegate { requester: owner, .. })) => {
            requester == *owner
        }
        (
            EndedPart::ByProject { project, label },
            Key::Tasks(tasks::Key::EndedByProject { project: owner, label: value, .. }),
        ) => project == *owner && (label.is_none() || label == Some(*value)),
        _ => false,
    }
}
/// Apply keys beyond the prefix to the joined ended owner.
pub(super) fn line(
    core: &Core,
    limits: &Limits,
    party: u64,
    query: &TaskQuery,
    row: &Row,
) -> Result<Option<PageRow>, Missing> {
    let Some(owner) = row_owner(row) else { return Ok(None) };
    let tasks::Phase::Ended(_) = owner.phase else { return Ok(None) };
    if !documents::task_query::may_touch(query, owner) {
        return Ok(None);
    }
    if query.requester.is_some() && query.requester != Some(owner.requester)
        || query.tracked.is_some() && query.tracked != Some(owner.tracked.is_some())
        || query.root.is_some() && query.root != Some(row.ancestors.last().copied().unwrap_or(owner.number))
    {
        return Ok(None);
    }
    if let Some(parent) = query.parent
        && owner.requester != tasks::Party::Task(parent)
    {
        return Ok(None);
    }
    Ok(Some(PageRow::Line(documents::task_line(core, limits, party, owner, row.ended_read)?)))
}
