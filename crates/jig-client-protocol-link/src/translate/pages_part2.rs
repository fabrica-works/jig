//! Whole pages and single documents (protocol/client.md, 3 and 6).
use super::{
    Box, Limits, List, Reader, Wall, c, cursor_fields, history_line_fields, inbox_entry_piece,
    owner_authority_document_fields, owner_chip_ending_fields, owner_history_change_fields,
    owner_message_proposal_kind_fields, owner_retained_action_fields, piece_problem, specification_document_fields,
    too_large, w,
};
pub(crate) fn owner_history_holder_fields(value: &w::OwnerHistoryHolder, limits: &Limits) -> c::OwnerHistoryHolder {
    match value {
        w::OwnerHistoryHolder::Task(row) => c::OwnerHistoryHolder::Task(row.value()),
        w::OwnerHistoryHolder::Person(row) => c::OwnerHistoryHolder::Person(row.value()),
        w::OwnerHistoryHolder::Policy(row) => c::OwnerHistoryHolder::Policy {
            project: row.project(),
            kind: owner_message_proposal_kind_fields(row.kind(), limits),
        },
    }
}

pub(crate) fn owner_proposal_history_state_fields(
    value: &w::OwnerProposalHistoryState,
    limits: &Limits,
) -> Result<c::OwnerProposalHistoryState, w::Problem> {
    Ok(match value {
        w::OwnerProposalHistoryState::Pending(row) => c::OwnerProposalHistoryState::Pending {
            holder: owner_history_holder_fields(row.holder(), limits),
            since: Wall::from_nanos(row.since()),
        },
        w::OwnerProposalHistoryState::Accepted(row) => {
            c::OwnerProposalHistoryState::Accepted { by: crate::translate::facts::owner_requester(row.by(), limits) }
        }
        w::OwnerProposalHistoryState::Rejected(row) => c::OwnerProposalHistoryState::Rejected {
            by: crate::translate::facts::owner_requester(row.by(), limits),
            reason: crate::translate::facts::document_text(row.reason(), limits)?,
        },
        w::OwnerProposalHistoryState::Withdrawn => c::OwnerProposalHistoryState::Withdrawn,
    })
}

pub(crate) fn owner_proposal_history_fields(
    value: &w::OwnerProposalHistory,
    limits: &Limits,
) -> Result<c::OwnerProposalHistory, w::Problem> {
    Ok(c::OwnerProposalHistory {
        action: owner_retained_action_fields(value.action(), limits)?,
        number: value.number(),
        proposer: value.proposer(),
        as_holder: value.as_holder(),
        state: owner_proposal_history_state_fields(value.state(), limits)?,
    })
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn owner_amendment_state_fields(
    value: &w::OwnerAmendmentState,
    limits: &Limits,
) -> Result<c::OwnerAmendmentState, w::Problem> {
    let opt0 = match value.addressed() {
        Some(item) => Some(crate::translate::facts::person_address(item, limits)),
        None => None,
    };
    let opt1 = match value.taken_by() {
        Some(item) => Some(*item),
        None => None,
    };
    let opt2 = match value.instructions() {
        Some(item) => Some(crate::translate::facts::document_text(item, limits)?),
        None => None,
    };
    if value.dependencies().len() > limits.client.rows {
        return Err(too_large());
    }
    let mut list3 = List::with_capacity(value.dependencies().len());
    for item in value.dependencies() {
        list3.push(*item).expect("reserved supplied list");
    }
    Ok(c::OwnerAmendmentState {
        addressed: opt0,
        taken_by: opt1,
        instructions: opt2,
        spec: specification_document_fields(value.spec(), limits)?,
        authority: owner_authority_document_fields(value.authority(), limits)?,
        wake: crate::translate::facts::owner_wake_policy(value.wake(), limits),
        dependencies: list3,
        cut: value.cut(),
    })
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn owner_history_line_fields(
    value: &w::OwnerHistoryLine,
    limits: &Limits,
) -> Result<c::OwnerHistoryLine, w::Problem> {
    let opt0 = match value.proposal() {
        Some(item) => Some(owner_proposal_history_fields(item, limits)?),
        None => None,
    };
    let opt1 = match value.amendment() {
        Some(item) => Some((
            Box::new(owner_amendment_state_fields(item.before(), limits)?),
            Box::new(owner_amendment_state_fields(item.after(), limits)?),
        )),
        None => None,
    };
    let opt2 = match value.at() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    Ok(c::OwnerHistoryLine {
        task: value.task(),
        revision: value.revision(),
        by: crate::translate::facts::owner_requester(value.by(), limits),
        change: owner_history_change_fields(value.change(), limits),
        reason: crate::translate::facts::document_text(value.reason(), limits)?,
        proposal: opt0,
        amendment: opt1,
        at: opt2,
    })
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn owner_milestone_phase_fields(
    value: &w::OwnerMilestonePhase,
    limits: &Limits,
) -> Result<c::OwnerMilestonePhase, w::Problem> {
    Ok(match value {
        w::OwnerMilestonePhase::Waiting => c::OwnerMilestonePhase::Waiting,
        w::OwnerMilestonePhase::Active(row) => {
            c::OwnerMilestonePhase::Active(crate::translate::facts::run_stage(row.value(), limits))
        }
        w::OwnerMilestonePhase::Closing(row) => c::OwnerMilestonePhase::Closing {
            stage: crate::translate::facts::closing_stage(row.stage(), limits),
            ending: owner_chip_ending_fields(row.ending(), limits)?,
        },
        w::OwnerMilestonePhase::Held(row) => {
            let opt0 = match row.source() {
                Some(item) => Some(crate::translate::facts::owner_part(item, limits)),
                None => None,
            };
            let opt1 = match row.ending() {
                Some(item) => Some(owner_chip_ending_fields(item, limits)?),
                None => None,
            };
            c::OwnerMilestonePhase::Held {
                before: crate::translate::facts::owner_prior_phase(row.before(), limits),
                why: crate::translate::answers::hold_category(row.why(), limits),
                source: opt0,
                ending: opt1,
            }
        }
        w::OwnerMilestonePhase::Ended(row) => {
            c::OwnerMilestonePhase::Ended(owner_chip_ending_fields(row.value(), limits)?)
        }
    })
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn owner_milestone_line_fields(
    value: &w::OwnerMilestoneLine,
    limits: &Limits,
) -> Result<c::OwnerMilestoneLine, w::Problem> {
    let opt0 = match value.at() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    Ok(c::OwnerMilestoneLine {
        task: value.task(),
        position: value.position(),
        phase: owner_milestone_phase_fields(value.phase(), limits)?,
        at: opt0,
    })
}

pub(crate) fn history_row_fields(value: &w::HistoryRow, limits: &Limits) -> Result<c::HistoryRow, w::Problem> {
    Ok(match value {
        w::HistoryRow::Line(row) => c::HistoryRow::Line(Box::new(history_line_fields(row.value(), limits)?)),
        w::HistoryRow::Change(row) => c::HistoryRow::Change(Box::new(owner_history_line_fields(row.value(), limits)?)),
        w::HistoryRow::Milestone(row) => {
            c::HistoryRow::Milestone(Box::new(owner_milestone_line_fields(row.value(), limits)?))
        }
    })
}

pub(crate) fn history_row_piece(bytes: &[u8], limits: &Limits, bound: u32) -> Result<List<c::HistoryRow>, w::Problem> {
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
        let row = w::HistoryRow::decode(&limits.wire, &mut Reader::new(&item))?;
        let row = history_row_fields(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn history_page_fields(value: &w::HistoryPage, limits: &Limits) -> Result<c::HistoryPage, w::Problem> {
    let opt0 = match value.before() {
        Some(item) => Some(cursor_fields(item, limits)?),
        None => None,
    };
    let opt1 = match value.after() {
        Some(item) => Some(cursor_fields(item, limits)?),
        None => None,
    };
    Ok(c::HistoryPage {
        cut: value.cut(),
        rows: history_row_piece(value.rows(), limits, limits.client.rows)?,
        before: opt0,
        after: opt1,
    })
}

pub(crate) fn inbox_page_fields(value: &w::InboxPage, limits: &Limits) -> Result<c::InboxPage, w::Problem> {
    let opt0 = match value.before() {
        Some(item) => Some(cursor_fields(item, limits)?),
        None => None,
    };
    let opt1 = match value.after() {
        Some(item) => Some(cursor_fields(item, limits)?),
        None => None,
    };
    Ok(c::InboxPage {
        cut: value.cut(),
        rows: inbox_entry_piece(value.rows(), limits, limits.client.rows)?,
        before: opt0,
        after: opt1,
    })
}
