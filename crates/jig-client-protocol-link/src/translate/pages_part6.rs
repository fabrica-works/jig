//! Whole pages and single documents (protocol/client.md, 3 and 6).
use super::{
    Box, Limits, List, Reader, Wall, authority_fields, c, chip_fields, ending_fields, field_rule_fields, offered_piece,
    owner_proposal_action_fields, owner_proposal_waiting_fields, owner_task_link_fields, piece_problem, result_fields,
    result_piece, specification_document_fields, task_contract_document_fields, task_fields, text, too_large, w,
};
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn owner_funding_offer_fields(value: &w::OwnerFundingOffer, limits: &Limits) -> c::OwnerFundingOffer {
    let opt0 = match value.left() {
        Some(item) => Some(*item),
        None => None,
    };
    let opt1 = match value.after() {
        Some(item) => Some(*item),
        None => None,
    };
    c::OwnerFundingOffer {
        source: crate::translate::facts::funder(value.source(), limits),
        needed: value.needed(),
        left: opt0,
        after: opt1,
    }
}

pub(crate) fn owner_proposal_fields(value: &w::OwnerProposal, limits: &Limits) -> Result<c::OwnerProposal, w::Problem> {
    if value.funding().len() > limits.client.rows {
        return Err(too_large());
    }
    let mut list0 = List::with_capacity(value.funding().len());
    for item in value.funding() {
        list0.push(owner_funding_offer_fields(item, limits)).expect("reserved supplied list");
    }
    Ok(c::OwnerProposal {
        key: crate::translate::answers::proposal_key(value.key(), limits),
        project: value.project(),
        reason: crate::translate::facts::document_text(value.reason(), limits)?,
        action: owner_proposal_action_fields(value.action(), limits)?,
        as_holder: value.as_holder(),
        waiting: owner_proposal_waiting_fields(value.waiting(), limits),
        funding: list0,
        acts: offered_piece(value.acts(), limits, c::MAX_OFFERED_ACTS)?,
    })
}

pub(crate) fn owner_escalation_fields(
    value: &w::OwnerEscalation,
    limits: &Limits,
) -> Result<c::OwnerEscalation, w::Problem> {
    Ok(c::OwnerEscalation {
        acts: offered_piece(value.acts(), limits, c::MAX_OFFERED_ACTS)?,
        task: Box::new(task_fields(value.task(), limits)?),
    })
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn question_fields(value: &w::Question, limits: &Limits) -> Result<c::Question, w::Problem> {
    let opt0 = match value.from() {
        Some(item) => Some(crate::translate::facts::owner_requester(item, limits)),
        None => None,
    };
    let opt1 = match value.ending() {
        Some(item) => Some(ending_fields(item, limits)?),
        None => None,
    };
    Ok(c::Question {
        task: chip_fields(value.task(), limits)?,
        from: opt0,
        message: value.message(),
        words: text(value.words(), limits)?,
        since: Wall::from_nanos(value.since()),
        acts: offered_piece(value.acts(), limits, c::MAX_OFFERED_ACTS)?,
        ending: opt1,
    })
}

pub(crate) fn option_line_fields(value: &w::OptionLine, limits: &Limits) -> Result<c::OptionLine, w::Problem> {
    if value.fields().len() > limits.client.rows {
        return Err(too_large());
    }
    let mut list0 = List::with_capacity(value.fields().len());
    for item in value.fields() {
        list0.push(field_rule_fields(item, limits)?).expect("reserved supplied list");
    }
    Ok(c::OptionLine {
        number: value.number(),
        words: text(value.words(), limits)?,
        words_max: value.words_max(),
        followups: value.followups(),
        fields: list0,
    })
}

pub(crate) fn option_line_piece(bytes: &[u8], limits: &Limits, bound: u32) -> Result<List<c::OptionLine>, w::Problem> {
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
        let row = w::OptionLine::decode(&limits.wire, &mut Reader::new(&item))?;
        let row = option_line_fields(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn person_contract_fields(
    value: &w::PersonContract,
    limits: &Limits,
) -> Result<c::PersonContract, w::Problem> {
    Ok(match value {
        w::PersonContract::Choice(row) => {
            c::PersonContract::Choice { options: option_line_piece(row.options(), limits, limits.client.rows)? }
        }
        w::PersonContract::Approval(row) => c::PersonContract::Approval {
            state: text(row.state(), limits)?,
            part: crate::translate::facts::part(row.part(), limits),
        },
        w::PersonContract::Words => c::PersonContract::Words,
    })
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn person_task_fields(value: &w::PersonTask, limits: &Limits) -> Result<c::PersonTask, w::Problem> {
    let opt0 = match value.taken() {
        Some(item) => Some(crate::translate::facts::taken_by(item, limits)?),
        None => None,
    };
    let opt1 = match value.since() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    let opt2 = match value.stall() {
        Some(item) => Some(*item),
        None => None,
    };
    let opt3 = match value.deadline() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    let opt4 = match value.ending() {
        Some(item) => Some(ending_fields(item, limits)?),
        None => None,
    };
    Ok(c::PersonTask {
        task: chip_fields(value.task(), limits)?,
        revision: crate::translate::answers::revision(value.revision(), limits),
        requester: crate::translate::answers::actor(value.requester(), limits),
        spec: specification_document_fields(value.spec(), limits)?,
        contract: person_contract_fields(value.contract(), limits)?,
        inputs: result_piece(value.inputs(), limits, limits.client.rows)?,
        addressed: crate::translate::facts::recipient(value.addressed(), limits),
        taken: opt0,
        since: opt1,
        stall: opt2,
        deadline: opt3,
        acts: offered_piece(value.acts(), limits, c::MAX_OFFERED_ACTS)?,
        ending: opt4,
    })
}

pub(crate) fn owner_shown_result_fields(
    value: &w::OwnerShownResult,
    limits: &Limits,
) -> Result<c::OwnerShownResult, w::Problem> {
    let opt0 = match value.result() {
        Some(item) => Some(Box::new(result_fields(item, limits)?)),
        None => None,
    };
    Ok(c::OwnerShownResult { task: owner_task_link_fields(value.task(), limits)?, result: opt0 })
}

pub(crate) fn owner_shown_result_piece(
    bytes: &[u8],
    limits: &Limits,
    bound: u32,
) -> Result<List<c::OwnerShownResult>, w::Problem> {
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
        let row = w::OwnerShownResult::decode(&limits.wire, &mut Reader::new(&item))?;
        let row = owner_shown_result_fields(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn owner_person_task_fields(
    value: &w::OwnerPersonTask,
    limits: &Limits,
) -> Result<c::OwnerPersonTask, w::Problem> {
    let opt0 = match value.taken_by() {
        Some(item) => Some(*item),
        None => None,
    };
    let opt1 = match value.since() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    let opt2 = match value.stall() {
        Some(item) => Some(*item),
        None => None,
    };
    let opt3 = match value.deadline() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    Ok(c::OwnerPersonTask {
        task: chip_fields(value.task(), limits)?,
        revision: crate::translate::answers::revision(value.revision(), limits),
        requester: crate::translate::facts::owner_requester(value.requester(), limits),
        spec: specification_document_fields(value.spec(), limits)?,
        address: crate::translate::facts::person_address(value.address(), limits),
        taken_by: opt0,
        since: opt1,
        stall: opt2,
        deadline: opt3,
        contract: task_contract_document_fields(value.contract(), limits)?,
        results: owner_shown_result_piece(value.results(), limits, limits.client.rows)?,
        cut: value.cut(),
        acts: offered_piece(value.acts(), limits, c::MAX_OFFERED_ACTS)?,
    })
}

pub(crate) fn role_policy_fields(value: &w::RolePolicy, limits: &Limits) -> Result<c::RolePolicy, w::Problem> {
    if value.decisions().len() > limits.client.rows {
        return Err(too_large());
    }
    let mut list0 = List::with_capacity(value.decisions().len());
    for item in value.decisions() {
        list0.push(*item).expect("reserved supplied list");
    }
    Ok(c::RolePolicy {
        number: value.number(),
        authority: authority_fields(value.authority(), limits)?,
        allotment: value.allotment(),
        decisions: list0,
    })
}
