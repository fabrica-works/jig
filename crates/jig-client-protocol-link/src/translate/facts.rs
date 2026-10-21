//! Field-for-field client contract translation (protocol/client.md, 3 and 5).
use crate::Limits;
use crate::translate::{piece_problem, too_large};
use alloc::boxed::Box;
use jig_client_domain::contract as c;
use jig_client_protocol_wire::v11 as w;
use skein_lib::Map;
use skein_lib::Reader;
#[path = "facts_part0.rs"]
mod part0;
use part0::chip_fields;
pub(crate) use part0::{
    active_run, call_identity, closing_stage, document_text, failure_class, hold_category, owner_chip_phase, part,
    person_address, phase, proposal_key, recipient, retry_notice, revision, run_stage, taken_by, wake_policy,
};
#[path = "facts_part1.rs"]
mod part1;
use part1::{chip_phase_fields, closing_fields, ending_fields, held_runtime_fields, hold_reason_fields};
pub(crate) use part1::{
    owner_chip_hold, owner_executor, owner_numbers, owner_part, owner_person_wait, owner_wake_policy, party,
};
#[path = "facts_part2.rs"]
mod part2;
pub(crate) use part2::dependencies;
pub(crate) use part2::{funder, owner_budget, owner_requester, owner_resource_line};
use part2::{owner_chip_details_fields, owner_chip_ending_fields, result_value_fields};

#[derive(Debug)]
struct Child {
    parent: Option<u32>,
    value: Node,
}
#[derive(Debug)]
pub(crate) enum Node {
    Chip(Box<c::Chip>),
    ChipPhase(Box<c::ChipPhase>),
    Closing(Box<c::Closing>),
    Ending(Box<c::Ending>),
    HeldRuntime(Box<c::HeldRuntime>),
    HoldReason(Box<c::HoldReason>),
    OwnerChipDetails(Box<c::OwnerChipDetails>),
    OwnerChipEnding(Box<c::OwnerChipEnding>),
    ResultValue(Box<c::ResultValue>),
}
fn take_chip(index: u32, parent: u32, children: &mut Map<u32, Child>) -> Result<c::Chip, w::Problem> {
    let child = children.remove(&index).ok_or(piece_problem(skein_codec::Reason::Malformed))?;
    if child.parent != Some(parent) {
        return Err(piece_problem(skein_codec::Reason::Malformed));
    }
    match child.value {
        Node::Chip(value) => Ok(*value),
        Node::ChipPhase(_)
        | Node::Closing(_)
        | Node::Ending(_)
        | Node::HeldRuntime(_)
        | Node::HoldReason(_)
        | Node::OwnerChipDetails(_)
        | Node::OwnerChipEnding(_)
        | Node::ResultValue(_) => Err(piece_problem(skein_codec::Reason::Malformed)),
    }
}

fn take_chip_phase(index: u32, parent: u32, children: &mut Map<u32, Child>) -> Result<c::ChipPhase, w::Problem> {
    let child = children.remove(&index).ok_or(piece_problem(skein_codec::Reason::Malformed))?;
    if child.parent != Some(parent) {
        return Err(piece_problem(skein_codec::Reason::Malformed));
    }
    match child.value {
        Node::ChipPhase(value) => Ok(*value),
        Node::Chip(_)
        | Node::Closing(_)
        | Node::Ending(_)
        | Node::HeldRuntime(_)
        | Node::HoldReason(_)
        | Node::OwnerChipDetails(_)
        | Node::OwnerChipEnding(_)
        | Node::ResultValue(_) => Err(piece_problem(skein_codec::Reason::Malformed)),
    }
}

fn take_closing(index: u32, parent: u32, children: &mut Map<u32, Child>) -> Result<c::Closing, w::Problem> {
    let child = children.remove(&index).ok_or(piece_problem(skein_codec::Reason::Malformed))?;
    if child.parent != Some(parent) {
        return Err(piece_problem(skein_codec::Reason::Malformed));
    }
    match child.value {
        Node::Closing(value) => Ok(*value),
        Node::Chip(_)
        | Node::ChipPhase(_)
        | Node::Ending(_)
        | Node::HeldRuntime(_)
        | Node::HoldReason(_)
        | Node::OwnerChipDetails(_)
        | Node::OwnerChipEnding(_)
        | Node::ResultValue(_) => Err(piece_problem(skein_codec::Reason::Malformed)),
    }
}

fn take_ending(index: u32, parent: u32, children: &mut Map<u32, Child>) -> Result<c::Ending, w::Problem> {
    let child = children.remove(&index).ok_or(piece_problem(skein_codec::Reason::Malformed))?;
    if child.parent != Some(parent) {
        return Err(piece_problem(skein_codec::Reason::Malformed));
    }
    match child.value {
        Node::Ending(value) => Ok(*value),
        Node::Chip(_)
        | Node::ChipPhase(_)
        | Node::Closing(_)
        | Node::HeldRuntime(_)
        | Node::HoldReason(_)
        | Node::OwnerChipDetails(_)
        | Node::OwnerChipEnding(_)
        | Node::ResultValue(_) => Err(piece_problem(skein_codec::Reason::Malformed)),
    }
}
pub(crate) fn ending(value: &w::Ending, limits: &Limits) -> Result<c::Ending, w::Problem> {
    let root = decode(value.rows(), limits)?;
    match root {
        Node::Ending(value) => Ok(*value),
        Node::Chip(_)
        | Node::ChipPhase(_)
        | Node::Closing(_)
        | Node::HeldRuntime(_)
        | Node::HoldReason(_)
        | Node::OwnerChipDetails(_)
        | Node::OwnerChipEnding(_)
        | Node::ResultValue(_) => Err(piece_problem(skein_codec::Reason::Malformed)),
    }
}
fn take_held_runtime(index: u32, parent: u32, children: &mut Map<u32, Child>) -> Result<c::HeldRuntime, w::Problem> {
    let child = children.remove(&index).ok_or(piece_problem(skein_codec::Reason::Malformed))?;
    if child.parent != Some(parent) {
        return Err(piece_problem(skein_codec::Reason::Malformed));
    }
    match child.value {
        Node::HeldRuntime(value) => Ok(*value),
        Node::Chip(_)
        | Node::ChipPhase(_)
        | Node::Closing(_)
        | Node::Ending(_)
        | Node::HoldReason(_)
        | Node::OwnerChipDetails(_)
        | Node::OwnerChipEnding(_)
        | Node::ResultValue(_) => Err(piece_problem(skein_codec::Reason::Malformed)),
    }
}

fn take_hold_reason(index: u32, parent: u32, children: &mut Map<u32, Child>) -> Result<c::HoldReason, w::Problem> {
    let child = children.remove(&index).ok_or(piece_problem(skein_codec::Reason::Malformed))?;
    if child.parent != Some(parent) {
        return Err(piece_problem(skein_codec::Reason::Malformed));
    }
    match child.value {
        Node::HoldReason(value) => Ok(*value),
        Node::Chip(_)
        | Node::ChipPhase(_)
        | Node::Closing(_)
        | Node::Ending(_)
        | Node::HeldRuntime(_)
        | Node::OwnerChipDetails(_)
        | Node::OwnerChipEnding(_)
        | Node::ResultValue(_) => Err(piece_problem(skein_codec::Reason::Malformed)),
    }
}
pub(crate) fn hold_reason(value: &w::HoldReason, limits: &Limits) -> Result<c::HoldReason, w::Problem> {
    let root = decode(value.rows(), limits)?;
    match root {
        Node::HoldReason(value) => Ok(*value),
        Node::Chip(_)
        | Node::ChipPhase(_)
        | Node::Closing(_)
        | Node::Ending(_)
        | Node::HeldRuntime(_)
        | Node::OwnerChipDetails(_)
        | Node::OwnerChipEnding(_)
        | Node::ResultValue(_) => Err(piece_problem(skein_codec::Reason::Malformed)),
    }
}
fn take_owner_chip_details(
    index: u32,
    parent: u32,
    children: &mut Map<u32, Child>,
) -> Result<c::OwnerChipDetails, w::Problem> {
    let child = children.remove(&index).ok_or(piece_problem(skein_codec::Reason::Malformed))?;
    if child.parent != Some(parent) {
        return Err(piece_problem(skein_codec::Reason::Malformed));
    }
    match child.value {
        Node::OwnerChipDetails(value) => Ok(*value),
        Node::Chip(_)
        | Node::ChipPhase(_)
        | Node::Closing(_)
        | Node::Ending(_)
        | Node::HeldRuntime(_)
        | Node::HoldReason(_)
        | Node::OwnerChipEnding(_)
        | Node::ResultValue(_) => Err(piece_problem(skein_codec::Reason::Malformed)),
    }
}

fn take_owner_chip_ending(
    index: u32,
    parent: u32,
    children: &mut Map<u32, Child>,
) -> Result<c::OwnerChipEnding, w::Problem> {
    let child = children.remove(&index).ok_or(piece_problem(skein_codec::Reason::Malformed))?;
    if child.parent != Some(parent) {
        return Err(piece_problem(skein_codec::Reason::Malformed));
    }
    match child.value {
        Node::OwnerChipEnding(value) => Ok(*value),
        Node::Chip(_)
        | Node::ChipPhase(_)
        | Node::Closing(_)
        | Node::Ending(_)
        | Node::HeldRuntime(_)
        | Node::HoldReason(_)
        | Node::OwnerChipDetails(_)
        | Node::ResultValue(_) => Err(piece_problem(skein_codec::Reason::Malformed)),
    }
}

fn take_result_value(index: u32, parent: u32, children: &mut Map<u32, Child>) -> Result<c::ResultValue, w::Problem> {
    let child = children.remove(&index).ok_or(piece_problem(skein_codec::Reason::Malformed))?;
    if child.parent != Some(parent) {
        return Err(piece_problem(skein_codec::Reason::Malformed));
    }
    match child.value {
        Node::ResultValue(value) => Ok(*value),
        Node::Chip(_)
        | Node::ChipPhase(_)
        | Node::Closing(_)
        | Node::Ending(_)
        | Node::HeldRuntime(_)
        | Node::HoldReason(_)
        | Node::OwnerChipDetails(_)
        | Node::OwnerChipEnding(_) => Err(piece_problem(skein_codec::Reason::Malformed)),
    }
}

pub(crate) fn decode(bytes: &[u8], limits: &Limits) -> Result<Node, w::Problem> {
    let mut reader = Reader::new(bytes);
    let bound = match u32::try_from(limits.client.document_bytes.checked_div(4).ok_or(too_large())?) {
        Ok(value) => value,
        Err(problem) => return Err(size_problem(problem)),
    };
    let mut rows = Map::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let item = match skein_codec::piece::split(&mut reader) {
            Ok(value) => value,
            Err(problem) => return Err(piece_problem(problem)),
        };
        let Some(item) = item else { break };
        sum = sum.checked_add(u64::try_from(item.len()).expect("piece fits u64")).ok_or(too_large())?;
        sum = sum.checked_add(4).ok_or(too_large())?;
        if sum > limits.client.document_bytes {
            return Err(too_large());
        }
        let row = w::Fact::decode(&limits.wire, &mut Reader::new(&item))?;
        if rows.insert(rows.len(), row).is_err() {
            return Err(too_large());
        }
    }
    if !reader.is_empty() || rows.is_empty() {
        return Err(piece_problem(skein_codec::Reason::Malformed));
    }
    let mut children = Map::with_capacity(rows.len());
    let mut root = None;
    for _ in 0..rows.len() {
        let index = rows.len().checked_sub(1).expect("fact row exists");
        let row = rows.remove(&index).expect("fact row exists");
        let (parent, value) = match row {
            w::Fact::Chip(row) => {
                (*row.parent(), Node::Chip(Box::new(chip_fields(row.value(), limits, index, &mut children)?)))
            }
            w::Fact::ChipPhase(row) => (
                *row.parent(),
                Node::ChipPhase(Box::new(chip_phase_fields(row.value(), limits, index, &mut children)?)),
            ),
            w::Fact::Closing(row) => {
                (*row.parent(), Node::Closing(Box::new(closing_fields(row.value(), limits, index, &mut children)?)))
            }
            w::Fact::Ending(row) => {
                (*row.parent(), Node::Ending(Box::new(ending_fields(row.value(), limits, index, &mut children)?)))
            }
            w::Fact::HeldRuntime(row) => (
                *row.parent(),
                Node::HeldRuntime(Box::new(held_runtime_fields(row.value(), limits, index, &mut children)?)),
            ),
            w::Fact::HoldReason(row) => (
                *row.parent(),
                Node::HoldReason(Box::new(hold_reason_fields(row.value(), limits, index, &mut children)?)),
            ),
            w::Fact::OwnerChipDetails(row) => (
                *row.parent(),
                Node::OwnerChipDetails(Box::new(owner_chip_details_fields(row.value(), limits, index, &mut children)?)),
            ),
            w::Fact::OwnerChipEnding(row) => (
                *row.parent(),
                Node::OwnerChipEnding(Box::new(owner_chip_ending_fields(row.value(), limits, index, &mut children)?)),
            ),
            w::Fact::ResultValue(row) => (
                *row.parent(),
                Node::ResultValue(Box::new(result_value_fields(row.value(), limits, index, &mut children)?)),
            ),
        };
        if index == 0 {
            if parent.is_some() {
                return Err(piece_problem(skein_codec::Reason::Malformed));
            }
            root = Some(value);
        } else {
            if parent.is_none() || parent >= Some(index) {
                return Err(piece_problem(skein_codec::Reason::Malformed));
            }
            children.insert(index, Child { parent, value }).expect("fact capacity checked");
        }
    }
    if !children.is_empty() {
        return Err(piece_problem(skein_codec::Reason::Malformed));
    }
    root.ok_or(piece_problem(skein_codec::Reason::Malformed))
}

fn size_problem(_: core::num::TryFromIntError) -> w::Problem {
    too_large()
}

pub(crate) use part1::{owner_failure_tries, owner_prior_phase};
pub(crate) use part2::result_field;
