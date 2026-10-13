//! Whole pages and single documents (protocol/client.md, 3 and 6).
use super::{
    Box, Limits, List, Wall, c, hold_fields, owner_authority_document_fields, proposed_task_piece,
    specification_document_fields, text, too_large, w,
};

pub(crate) fn history_change_fields(value: &w::HistoryChange, limits: &Limits) -> Result<c::HistoryChange, w::Problem> {
    Ok(match value {
        w::HistoryChange::Made(row) => c::HistoryChange::Made { batch: row.batch() },
        w::HistoryChange::Amended(row) => {
            c::HistoryChange::Amended { before: text(row.before(), limits)?, after: text(row.after(), limits)? }
        }
        w::HistoryChange::Held(row) => c::HistoryChange::Held(hold_fields(row.value(), limits)?),
        w::HistoryChange::Released => c::HistoryChange::Released,
        w::HistoryChange::Cancelled(row) => c::HistoryChange::Cancelled { reason: text(row.reason(), limits)? },
        w::HistoryChange::Moved(row) => c::HistoryChange::Moved {
            from: crate::translate::answers::actor(row.from(), limits),
            to: crate::translate::answers::actor(row.to(), limits),
            reason: text(row.reason(), limits)?,
        },
    })
}

pub(crate) fn history_line_fields(value: &w::HistoryLine, limits: &Limits) -> Result<c::HistoryLine, w::Problem> {
    Ok(c::HistoryLine {
        position: value.position(),
        task: value.task(),
        by: crate::translate::answers::actor(value.by(), limits),
        at: Wall::from_nanos(value.at()),
        change: history_change_fields(value.change(), limits)?,
    })
}

pub(crate) fn owner_history_change_fields(value: &w::OwnerHistoryChange, _limits: &Limits) -> c::OwnerHistoryChange {
    match value {
        w::OwnerHistoryChange::Cancelled => c::OwnerHistoryChange::Cancelled,
        w::OwnerHistoryChange::Released => c::OwnerHistoryChange::Released,
        w::OwnerHistoryChange::Amended => c::OwnerHistoryChange::Amended,
        w::OwnerHistoryChange::Prioritised(row) => c::OwnerHistoryChange::Prioritised { priority: row.priority() },
        w::OwnerHistoryChange::Moved => c::OwnerHistoryChange::Moved,
        w::OwnerHistoryChange::Proposed => c::OwnerHistoryChange::Proposed,
        w::OwnerHistoryChange::ProposalPassed => c::OwnerHistoryChange::ProposalPassed,
        w::OwnerHistoryChange::ProposalAccepted => c::OwnerHistoryChange::ProposalAccepted,
        w::OwnerHistoryChange::ProposalRejected => c::OwnerHistoryChange::ProposalRejected,
        w::OwnerHistoryChange::ProposalWithdrawn => c::OwnerHistoryChange::ProposalWithdrawn,
    }
}

pub(crate) fn address_change_fields(value: &w::AddressChange, limits: &Limits) -> c::AddressChange {
    c::AddressChange {
        revision: crate::translate::answers::revision(value.revision(), limits),
        address: crate::translate::facts::person_address(value.address(), limits),
    }
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn owner_retained_amendment_fields(
    value: &w::OwnerRetainedAmendment,
    limits: &Limits,
) -> Result<c::OwnerRetainedAmendment, w::Problem> {
    let opt0 = match value.addressed() {
        Some(item) => Some(address_change_fields(item, limits)),
        None => None,
    };
    let opt1 = match value.instructions() {
        Some(item) => Some(crate::translate::facts::document_text(item, limits)?),
        None => None,
    };
    let opt2 = match value.release() {
        Some(item) => Some(*item),
        None => None,
    };
    let opt3 = match value.spec() {
        Some(item) => Some(specification_document_fields(item, limits)?),
        None => None,
    };
    let opt4 = match value.authority() {
        Some(item) => Some(owner_authority_document_fields(item, limits)?),
        None => None,
    };
    let opt5 = match value.wake() {
        Some(item) => Some(crate::translate::facts::owner_wake_policy(item, limits)),
        None => None,
    };
    let opt6 = match value.dependencies() {
        Some(item) => {
            if (item).len() > limits.client.rows {
                return Err(too_large());
            }
            let mut list0 = List::with_capacity((item).len());
            for item in item {
                list0.push(*item).expect("reserved supplied list");
            }
            Some(list0)
        }
        None => None,
    };
    Ok(c::OwnerRetainedAmendment {
        addressed: opt0,
        instructions: opt1,
        release: opt2,
        spec: opt3,
        authority: opt4,
        wake: opt5,
        dependencies: opt6,
        reason: crate::translate::facts::document_text(value.reason(), limits)?,
        cut: value.cut(),
    })
}

pub(crate) fn owner_retained_action_fields(
    value: &w::OwnerRetainedAction,
    limits: &Limits,
) -> Result<c::OwnerRetainedAction, w::Problem> {
    Ok(match value {
        w::OwnerRetainedAction::Batch(row) => {
            c::OwnerRetainedAction::Batch(proposed_task_piece(row.value(), limits, limits.client.rows)?)
        }
        w::OwnerRetainedAction::Effect(row) => c::OwnerRetainedAction::Effect {
            part: crate::translate::facts::owner_part(row.part(), limits),
            authority: owner_authority_document_fields(row.authority(), limits)?,
            attempt: row.attempt(),
            completion: row.completion(),
            position: row.position(),
            purpose: row.purpose(),
        },
        w::OwnerRetainedAction::ProcedureEffect(row) => c::OwnerRetainedAction::ProcedureEffect {
            part: crate::translate::facts::owner_part(row.part(), limits),
            authority: owner_authority_document_fields(row.authority(), limits)?,
            step: row.step(),
            purpose: row.purpose(),
        },
        w::OwnerRetainedAction::Amend(row) => c::OwnerRetainedAction::Amend {
            task: row.task(),
            change: Box::new(owner_retained_amendment_fields(row.change(), limits)?),
        },
        w::OwnerRetainedAction::Widen(row) => c::OwnerRetainedAction::Widen {
            task: row.task(),
            authority: owner_authority_document_fields(row.authority(), limits)?,
        },
        w::OwnerRetainedAction::Release(row) => c::OwnerRetainedAction::Release { task: row.task() },
    })
}
