//! Whole pages and single documents (protocol/client.md, 3 and 6).
use super::{
    Limits, List, Reader, Wall, authority_fields, budget_fields, c, offered_piece, opaque, owner_authority_fields,
    owner_message_proposal_kind_fields, part_piece, piece_problem, pool_fields, recurring_piece, role_policy_fields,
    specification_fields, task_contract_document_fields, task_spend_piece, text, too_large, w,
};
pub(crate) fn role_policy_piece(bytes: &[u8], limits: &Limits, bound: u32) -> Result<List<c::RolePolicy>, w::Problem> {
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
        let row = w::RolePolicy::decode(&limits.wire, &mut Reader::new(&item))?;
        let row = role_policy_fields(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn decider_fields(value: &w::Decider, _limits: &Limits) -> c::Decider {
    c::Decider { kind: value.kind(), role: value.role() }
}

pub(crate) fn policy_fields(value: &w::Policy, limits: &Limits) -> Result<c::Policy, w::Problem> {
    if value.deciders().len() > limits.client.rows {
        return Err(too_large());
    }
    let mut list0 = List::with_capacity(value.deciders().len());
    for item in value.deciders() {
        list0.push(decider_fields(item, limits)).expect("reserved supplied list");
    }
    Ok(c::Policy {
        ceiling: authority_fields(value.ceiling(), limits)?,
        period: value.period(),
        spend: value.spend(),
        roles: role_policy_piece(value.roles(), limits, limits.client.rows)?,
        requirements: part_piece(value.requirements(), limits, limits.client.rows)?,
        deciders: list0,
    })
}

pub(crate) fn holding_fields(value: &w::Holding, limits: &Limits) -> c::Holding {
    c::Holding { party: value.party(), role: crate::translate::answers::held_role(value.role(), limits) }
}

pub(crate) fn project_fields(value: &w::Project, limits: &Limits) -> Result<c::Project, w::Problem> {
    if value.roles().len() > limits.client.rows {
        return Err(too_large());
    }
    let mut list0 = List::with_capacity(value.roles().len());
    for item in value.roles() {
        list0.push(holding_fields(item, limits)).expect("reserved supplied list");
    }
    if value.pools().len() > limits.client.rows {
        return Err(too_large());
    }
    let mut list1 = List::with_capacity(value.pools().len());
    for item in value.pools() {
        list1.push(pool_fields(item, limits)).expect("reserved supplied list");
    }
    if value.services().len() > limits.client.rows {
        return Err(too_large());
    }
    let mut list2 = List::with_capacity(value.services().len());
    for item in value.services() {
        list2.push(crate::translate::answers::party(item, limits)?).expect("reserved supplied list");
    }
    Ok(c::Project {
        period: value.period(),
        spend_lines: task_spend_piece(value.spend_lines(), limits, limits.client.rows)?,
        number: value.number(),
        name: text(value.name(), limits)?,
        policy: policy_fields(value.policy(), limits)?,
        roles: list0,
        spend: budget_fields(value.spend(), limits),
        pools: list1,
        resources: part_piece(value.resources(), limits, limits.client.rows)?,
        services: list2,
        recurring: recurring_piece(value.recurring(), limits, limits.client.rows)?,
        parts: part_piece(value.parts(), limits, limits.client.rows)?,
        acts: offered_piece(value.acts(), limits, limits.client.rows)?,
    })
}

pub(crate) fn owner_goal_holding_fields(
    value: &w::OwnerGoalHolding,
    limits: &Limits,
) -> Result<c::OwnerGoalHolding, w::Problem> {
    Ok(match value {
        w::OwnerGoalHolding::Write(row) => {
            if row.path().len() > limits.client.segments {
                return Err(too_large());
            }
            let mut list0 = List::with_capacity(row.path().len());
            for item in row.path() {
                list0.push(opaque(item, limits)?).expect("reserved supplied list");
            }
            c::OwnerGoalHolding::Write { connector: row.connector(), path: list0, kind: row.kind() }
        }
        w::OwnerGoalHolding::Slot(row) => {
            if row.path().len() > limits.client.segments {
                return Err(too_large());
            }
            let mut list0 = List::with_capacity(row.path().len());
            for item in row.path() {
                list0.push(opaque(item, limits)?).expect("reserved supplied list");
            }
            c::OwnerGoalHolding::Slot { connector: row.connector(), path: list0, kind: row.kind() }
        }
    })
}

pub(crate) fn owner_goal_holding_piece(
    bytes: &[u8],
    limits: &Limits,
    bound: u32,
) -> Result<List<c::OwnerGoalHolding>, w::Problem> {
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
        let row = w::OwnerGoalHolding::decode(&limits.wire, &mut Reader::new(&item))?;
        let row = owner_goal_holding_fields(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn owner_person_proposal_goal_fields(
    value: &w::OwnerPersonProposalGoal,
    limits: &Limits,
) -> Result<c::OwnerPersonProposalGoal, w::Problem> {
    if value.dependencies().len() > limits.client.rows {
        return Err(too_large());
    }
    let mut list0 = List::with_capacity(value.dependencies().len());
    for item in value.dependencies() {
        list0.push(*item).expect("reserved supplied list");
    }
    let opt1 = match value.tracked() {
        Some(item) => Some(*item),
        None => None,
    };
    Ok(c::OwnerPersonProposalGoal {
        number: value.number(),
        project: value.project(),
        executor: crate::translate::facts::owner_executor(value.executor(), limits),
        spec: specification_fields(value.spec(), limits)?,
        contract: task_contract_document_fields(value.contract(), limits)?,
        authority: owner_authority_fields(value.authority(), limits)?,
        numbers: crate::translate::facts::owner_numbers(value.numbers(), limits),
        funder: crate::translate::facts::funder(value.funder(), limits),
        dependencies: list0,
        holdings: owner_goal_holding_piece(value.holdings(), limits, limits.client.rows)?,
        wake: crate::translate::facts::owner_wake_policy(value.wake(), limits),
        tracked: opt1,
    })
}

pub(crate) fn owner_person_proposal_state_fields(
    value: &w::OwnerPersonProposalState,
    limits: &Limits,
) -> Result<c::OwnerPersonProposalState, w::Problem> {
    Ok(match value {
        w::OwnerPersonProposalState::Pending(row) => {
            c::OwnerPersonProposalState::Pending { since: Wall::from_nanos(row.since()) }
        }
        w::OwnerPersonProposalState::Accepted(row) => {
            c::OwnerPersonProposalState::Accepted { by: crate::translate::facts::owner_requester(row.by(), limits) }
        }
        w::OwnerPersonProposalState::Rejected(row) => c::OwnerPersonProposalState::Rejected {
            by: crate::translate::facts::owner_requester(row.by(), limits),
            reason: text(row.reason(), limits)?,
            message: row.message(),
            at: Wall::from_nanos(row.at()),
        },
    })
}

pub(crate) fn owner_person_proposal_fields(
    value: &w::OwnerPersonProposal,
    limits: &Limits,
) -> Result<c::OwnerPersonProposal, w::Problem> {
    Ok(c::OwnerPersonProposal {
        number: value.number(),
        proposer: value.proposer(),
        project: value.project(),
        goal: owner_person_proposal_goal_fields(value.goal(), limits)?,
        state: owner_person_proposal_state_fields(value.state(), limits)?,
    })
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn owner_proposal_decision_fields(
    value: &w::OwnerProposalDecision,
    limits: &Limits,
) -> c::OwnerProposalDecision {
    let opt0 = match value.created() {
        Some(item) => Some((item.first(), item.last())),
        None => None,
    };
    c::OwnerProposalDecision {
        project: value.project(),
        proposer: crate::translate::answers::proposer(value.proposer(), limits),
        proposal: value.proposal(),
        kind: owner_message_proposal_kind_fields(value.kind(), limits),
        by: value.by(),
        at: Wall::from_nanos(value.at()),
        choice: crate::translate::answers::owner_proposal_choice(value.choice(), limits),
        created: opt0,
    }
}

pub(crate) fn owner_escalation_decision_fields(
    value: &w::OwnerEscalationDecision,
    limits: &Limits,
) -> Result<c::OwnerEscalationDecision, w::Problem> {
    Ok(match value {
        w::OwnerEscalationDecision::Release => c::OwnerEscalationDecision::Release,
        w::OwnerEscalationDecision::Reject(row) => {
            c::OwnerEscalationDecision::Reject { reason: text(row.reason(), limits)? }
        }
        w::OwnerEscalationDecision::Pass => c::OwnerEscalationDecision::Pass,
    })
}
