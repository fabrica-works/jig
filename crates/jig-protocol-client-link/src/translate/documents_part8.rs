//! Field-for-field document values (client/domain/contract.md, 5–7).
use super::{
    Limits, List, Wall, c, object_key_fields, offered_piece, owner_project_member_fields, owner_project_policy_fields,
    owner_project_resource_piece, owner_project_spend_piece, part_piece, pool_ledger_fields, recurring_piece, text,
    too_large, w,
};
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn owner_project_fields(value: &w::OwnerProject, limits: &Limits) -> Result<c::OwnerProject, w::Problem> {
    if value.members().len() > limits.client.rows {
        return Err(too_large());
    }
    let mut list0 = List::with_capacity(value.members().len());
    for item in value.members() {
        list0.push(owner_project_member_fields(item, limits)?).expect("reserved supplied list");
    }
    let opt1 = match value.current_period() {
        Some(item) => Some(pool_ledger_fields(item, limits)),
        None => None,
    };
    if value.pools().len() > limits.client.rows {
        return Err(too_large());
    }
    let mut list2 = List::with_capacity(value.pools().len());
    for item in value.pools() {
        list2.push(pool_ledger_fields(item, limits)).expect("reserved supplied list");
    }
    if value.configured_pools().len() > limits.client.rows {
        return Err(too_large());
    }
    let mut list3 = List::with_capacity(value.configured_pools().len());
    for item in value.configured_pools() {
        list3.push(pool_ledger_fields(item, limits)).expect("reserved supplied list");
    }
    Ok(c::OwnerProject {
        acts: offered_piece(value.acts(), limits, limits.client.rows)?,
        number: value.number(),
        resources: owner_project_resource_piece(value.resources(), limits, limits.client.rows)?,
        policy: owner_project_policy_fields(value.policy(), limits)?,
        members: list0,
        current_period: opt1,
        pools: list2,
        configured_period: value.configured_period(),
        configured_pools: list3,
        configured_pools_cut: value.configured_pools_cut(),
        spend: owner_project_spend_piece(value.spend(), limits, limits.client.rows)?,
        recurring: recurring_piece(value.recurring(), limits, limits.client.rows)?,
        cut: value.cut(),
    })
}

pub(crate) fn provisional_turn_name_fields(value: &w::ProvisionalTurnName, _limits: &Limits) -> c::ProvisionalTurnName {
    c::ProvisionalTurnName { attempt: value.attempt(), number: value.number(), retry: value.retry() }
}

pub(crate) fn call_fields(value: &w::Call, limits: &Limits) -> Result<c::Call, w::Problem> {
    let opt0 = match value.outcome() {
        Some(item) => Some(text(item, limits)?),
        None => None,
    };
    if value.objects().len() > limits.client.rows {
        return Err(too_large());
    }
    let mut list1 = List::with_capacity(value.objects().len());
    for item in value.objects() {
        list1.push(object_key_fields(item, limits)).expect("reserved supplied list");
    }
    Ok(c::Call {
        number: value.number(),
        tool: text(value.tool(), limits)?,
        input: text(value.input(), limits)?,
        outcome: opt0,
        failed: value.failed(),
        objects: list1,
        parts: part_piece(value.parts(), limits, limits.client.rows)?,
    })
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn current_call_fields(value: &w::CurrentCall, limits: &Limits) -> Result<c::CurrentCall, w::Problem> {
    let opt0 = match value.began() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    Ok(c::CurrentCall {
        identity: crate::translate::facts::call_identity(value.identity(), limits)?,
        tool: text(value.tool(), limits)?,
        input: text(value.input(), limits)?,
        cut: value.cut(),
        began: opt0,
    })
}

pub(crate) fn live_call_fields(value: &w::LiveCall, limits: &Limits) -> Result<c::LiveCall, w::Problem> {
    Ok(match value {
        w::LiveCall::Decoded(row) => c::LiveCall::Decoded(call_fields(row.value(), limits)?),
        w::LiveCall::Raw(row) => c::LiveCall::Raw(current_call_fields(row.value(), limits)?),
    })
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn run_text_fields(value: &w::RunText, limits: &Limits) -> Result<c::RunText, w::Problem> {
    let opt0 = match value.began() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    let opt1 = match value.call() {
        Some(item) => Some(live_call_fields(item, limits)?),
        None => None,
    };
    Ok(c::RunText {
        attempt: value.attempt(),
        began: opt0,
        turn: provisional_turn_name_fields(value.turn(), limits),
        from: value.from(),
        through: value.through(),
        text: text(value.text(), limits)?,
        cut: value.cut(),
        call: opt1,
    })
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn run_state_fields(value: &w::RunState, limits: &Limits) -> Result<c::RunState, w::Problem> {
    let opt0 = match value.attempt() {
        Some(item) => Some(*item),
        None => None,
    };
    let opt1 = match value.writing() {
        Some(item) => Some(run_text_fields(item, limits)?),
        None => None,
    };
    let opt2 = match value.call() {
        Some(item) => Some(current_call_fields(item, limits)?),
        None => None,
    };
    Ok(c::RunState {
        task: value.task(),
        attempt: opt0,
        committed_turn: value.committed_turn(),
        writing: opt1,
        call: opt2,
    })
}
