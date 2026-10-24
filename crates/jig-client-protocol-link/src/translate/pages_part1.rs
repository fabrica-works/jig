//! Whole pages and single documents (protocol/client.md, 3 and 6).
use super::{
    Box, Limits, List, Reader, Wall, archived_turn_fields, c, cursor_fields, hold_fields, object_key_fields,
    owner_activity_line_fields, owner_authority_document_fields, piece_problem, proposed_task_piece, result_fields,
    specification_document_fields, text, too_large, turn_fields, turn_name_fields, w,
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
        w::OwnerRetainedAction::Settings(row) => c::OwnerRetainedAction::Settings(
            crate::translate::settings_proposals::restore_owner_retained_action_settings(row, limits)?,
        ),
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

pub(crate) fn activity_page_fields(value: &w::ActivityPage, limits: &Limits) -> Result<c::ActivityPage, w::Problem> {
    let opt0 = match value.before() {
        Some(item) => Some(cursor_fields(item, limits)?),
        None => None,
    };
    let opt1 = match value.after() {
        Some(item) => Some(cursor_fields(item, limits)?),
        None => None,
    };
    Ok(c::ActivityPage {
        cut: value.cut(),
        rows: activity_line_piece(value.rows(), limits, limits.client.rows)?,
        before: opt0,
        after: opt1,
    })
}

pub(crate) fn activity_line_piece(
    bytes: &[u8],
    limits: &Limits,
    bound: u32,
) -> Result<List<c::ActivityLine>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let item = match skein_codec::piece::split(&mut reader) {
            Ok(value) => value,
            Err(reason) => return Err(piece_problem(reason)),
        };
        let Some(item) = item else {
            return Ok(rows);
        };
        sum = sum
            .checked_add(u64::try_from(item.len()).expect("piece fits u64"))
            .ok_or(too_large())?
            .checked_add(4)
            .ok_or(too_large())?;
        if sum > limits.client.document_bytes {
            return Err(too_large());
        }
        let row = w::ActivityLine::decode(&limits.wire, &mut Reader::new(&item))?;
        let row = activity_line_fields(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn activity_line_fields(value: &w::ActivityLine, limits: &Limits) -> Result<c::ActivityLine, w::Problem> {
    Ok(c::ActivityLine {
        position: value.position(),
        at: Wall::from_nanos(value.at()),
        event: activity_fields(value.event(), limits)?,
    })
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn activity_fields(value: &w::Activity, limits: &Limits) -> Result<c::Activity, w::Problem> {
    Ok(match value {
        w::Activity::Owner(row) => {
            if row.decoded().is_none()
                && let w::OwnerActivity::Turn(turn) = row.source().line()
            {
                return Ok(c::Activity::Turn {
                    name: c::TurnName { attempt: turn.attempt(), number: turn.turn() },
                    turn: c::TurnRead::Undecodable { why: c::TurnProblem::Malformed },
                });
            }
            let opt0 = match row.decoded() {
                Some(item) => Some(archived_turn_fields(item, limits)),
                None => None,
            };
            c::Activity::Owner { source: Box::new(owner_activity_line_fields(row.source(), limits)?), decoded: opt0 }
        }
        w::Activity::Words(row) => c::Activity::Words {
            by: crate::translate::answers::party(row.by(), limits)?,
            message: row.message(),
            words: text(row.words(), limits)?,
        },
        w::Activity::RunBegan(row) => c::Activity::RunBegan {
            attempt: row.attempt(),
            how: run_beginning_fields(row.how(), limits),
            why: text(row.why(), limits)?,
        },
        w::Activity::RunEnded(row) => c::Activity::RunEnded {
            attempt: row.attempt(),
            how: run_ending_fields(row.how(), limits)?,
            spent: row.spent(),
        },
        w::Activity::Turn(row) => {
            c::Activity::Turn { name: turn_name_fields(row.name(), limits), turn: turn_fields(row.turn(), limits) }
        }
        w::Activity::Result(row) => c::Activity::Result { result: result_fields(row.result(), limits)? },
        w::Activity::News(row) => {
            let opt0 = match row.part() {
                Some(item) => Some(crate::translate::facts::part(item, limits)),
                None => None,
            };
            c::Activity::News { topic: row.topic(), summary: text(row.summary(), limits)?, part: opt0 }
        }
        w::Activity::Decision(row) => c::Activity::Decision {
            object: object_key_fields(row.object(), limits),
            by: crate::translate::answers::actor(row.by(), limits),
            how: crate::translate::answers::decision(row.how(), limits)?,
        },
    })
}

pub(crate) fn run_ending_fields(value: &w::RunEnding, limits: &Limits) -> Result<c::RunEnding, w::Problem> {
    Ok(match value {
        w::RunEnding::Parked => c::RunEnding::Parked,
        w::RunEnding::Finished => c::RunEnding::Finished,
        w::RunEnding::Cancelled => c::RunEnding::Cancelled,
        w::RunEnding::Failed(row) => c::RunEnding::Failed {
            class: crate::translate::facts::failure_class(row.class(), limits),
            reason: text(row.reason(), limits)?,
        },
    })
}

pub(crate) fn run_beginning_fields(value: &w::RunBeginning, _limits: &Limits) -> c::RunBeginning {
    match value {
        w::RunBeginning::Resumed => c::RunBeginning::Resumed,
        w::RunBeginning::Fresh(row) => c::RunBeginning::Fresh { tail: row.tail_count() },
    }
}
