//! Channel bounds from the owners' numbers (protocol/channel.md, 9 and 11).
use crate::{Kind, v1};

/// Bounds supplied by the domains and the application's protocol layer.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Sizes {
    pub charter: u32,
    /// The whole resumed window, including each turn's four-byte length.
    pub transcript: u32,
    pub turn: u32,
    pub message: u32,
    pub sender: u32,
    pub call_name: u32,
    pub tool_name: u32,
    pub call_input: u32,
    pub answer: u32,
    pub result: u32,
    pub detail: u32,
    pub grant: u32,
    pub account_id: u32,
    pub name: u32,
    pub secret: u32,
    pub model_name: u32,
    pub items: u32,
    pub left: u32,
    pub saved: u32,
    pub evidence: u32,
    pub resource: u32,
    pub slots: u32,
    pub calls: u32,
    pub turns: u32,
    pub inbox: u32,
    pub answered: u32,
    pub models: u32,
    pub workstreams: u32,
    pub accounts: u32,
    pub credentials: u32,
    /// Bytes reserved for facts, including their frame headers.
    pub facts: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Cap {
    pub bytes: u32,
    pub frames: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct WorkerCap {
    pub cap: Cap,
    pub facts: u32,
}

/// Map field and count bounds without inspecting any payload or sum.
#[must_use]
pub fn limits(sizes: &Sizes) -> Option<v1::Limits> {
    let limits = v1::Limits {
        credential_name: sizes.name,
        credential_secret: sizes.secret,
        model_bound_name: sizes.model_name,
        bounds_models: sizes.models,
        hello_workstreams: sizes.workstreams,
        hello_hosting: sizes.slots,
        turn_refused: sizes.inbox,
        turn_body: sizes.turn,
        call_name: sizes.call_name,
        call_tool: sizes.tool_name,
        call_input: sizes.call_input,
        work_left_some: sizes.left,
        work_saved_some: sizes.saved,
        ended_outcome: sizes.result,
        resource_resource_some: sizes.resource,
        failed_detail: sizes.detail,
        answer_refused: sizes.inbox,
        message_sender: sizes.sender,
        message_words: sizes.message,
        workspace_items: sizes.items,
        host_answer_body: sizes.answer,
        delivery_answer_evidence: sizes.evidence,
        answered_call_name: sizes.call_name,
        answered_call_tool: sizes.tool_name,
        grant_value: sizes.grant,
        grant_account_id: sizes.account_id,
        assignment_charter: sizes.charter,
        assignment_transcript: sizes.transcript,
        assignment_answered: sizes.answered,
        assignment_messages: sizes.inbox,
        assignment_grants: sizes.accounts.checked_add(sizes.credentials)?,
        inbound_sender: sizes.sender,
        inbound_words: sizes.message,
        relay_text_text: sizes.answer,
        relayed_name: sizes.call_name,
    };
    if within_ceilings(&limits) { Some(limits) } else { None }
}

fn within_ceilings(limits: &v1::Limits) -> bool {
    let ceilings = &v1::CEILINGS;
    limits.credential_name <= ceilings.credential_name
        && limits.credential_secret <= ceilings.credential_secret
        && limits.model_bound_name <= ceilings.model_bound_name
        && limits.bounds_models <= ceilings.bounds_models
        && limits.hello_workstreams <= ceilings.hello_workstreams
        && limits.hello_hosting <= ceilings.hello_hosting
        && limits.turn_refused <= ceilings.turn_refused
        && limits.turn_body <= ceilings.turn_body
        && limits.call_name <= ceilings.call_name
        && limits.call_tool <= ceilings.call_tool
        && limits.call_input <= ceilings.call_input
        && limits.work_left_some <= ceilings.work_left_some
        && limits.work_saved_some <= ceilings.work_saved_some
        && limits.ended_outcome <= ceilings.ended_outcome
        && limits.resource_resource_some <= ceilings.resource_resource_some
        && limits.failed_detail <= ceilings.failed_detail
        && limits.answer_refused <= ceilings.answer_refused
        && limits.message_sender <= ceilings.message_sender
        && limits.message_words <= ceilings.message_words
        && limits.workspace_items <= ceilings.workspace_items
        && limits.host_answer_body <= ceilings.host_answer_body
        && limits.delivery_answer_evidence <= ceilings.delivery_answer_evidence
        && limits.answered_call_name <= ceilings.answered_call_name
        && limits.answered_call_tool <= ceilings.answered_call_tool
        && limits.grant_value <= ceilings.grant_value
        && limits.grant_account_id <= ceilings.grant_account_id
        && limits.assignment_charter <= ceilings.assignment_charter
        && limits.assignment_transcript <= ceilings.assignment_transcript
        && limits.assignment_answered <= ceilings.assignment_answered
        && limits.assignment_messages <= ceilings.assignment_messages
        && limits.assignment_grants <= ceilings.assignment_grants
        && limits.inbound_sender <= ceilings.inbound_sender
        && limits.inbound_words <= ceilings.inbound_words
        && limits.relay_text_text <= ceilings.relay_text_text
        && limits.relayed_name <= ceilings.relayed_name
}

#[must_use]
pub fn largest_body(sizes: &Sizes, kind: Kind) -> Option<u32> {
    body(&limits(sizes)?, kind)
}

fn body(limits: &v1::Limits, kind: Kind) -> Option<u32> {
    let bytes = match kind {
        Kind::Hello => v1::Hello::worst_case_bytes(limits),
        Kind::Turn => v1::Turn::worst_case_bytes(limits),
        Kind::Call => v1::Call::worst_case_bytes(limits),
        Kind::Answer => v1::Answer::worst_case_bytes(limits),
        Kind::Fact => v1::Fact::worst_case_bytes(limits),
        Kind::Rejected => v1::Rejected::worst_case_bytes(limits),
        Kind::Exhausted => v1::Exhausted::worst_case_bytes(limits),
        Kind::Assignment => v1::Assignment::worst_case_bytes(limits),
        Kind::Message => v1::Inbound::worst_case_bytes(limits),
        Kind::Relayed => v1::Relayed::worst_case_bytes(limits),
        Kind::TurnKept => v1::TurnKept::worst_case_bytes(limits),
        Kind::TurnBusy => v1::TurnBusy::worst_case_bytes(limits),
        Kind::Acknowledge => v1::Acknowledge::worst_case_bytes(limits),
        Kind::Cancel => v1::Cancel::worst_case_bytes(limits),
        Kind::Grant => v1::Granted::worst_case_bytes(limits),
    }?;
    u32::try_from(bytes).ok()
}

fn frame(limits: &v1::Limits, kind: Kind) -> Option<u32> {
    body(limits, kind)?.checked_add(8)
}

#[must_use]
pub fn engine_cap(sizes: &Sizes) -> Option<Cap> {
    let limits = limits(sizes)?;
    let grants = sizes.accounts.checked_add(sizes.credentials)?;
    let mut bytes = frame(&limits, Kind::Assignment)?;
    bytes = bytes.checked_add(sizes.inbox.checked_mul(frame(&limits, Kind::Message)?)?)?;
    bytes = bytes.checked_add(sizes.calls.checked_mul(frame(&limits, Kind::Relayed)?)?)?;
    bytes = bytes.checked_add(sizes.turns.checked_mul(frame(&limits, Kind::TurnKept)?)?)?;
    bytes = bytes.checked_add(frame(&limits, Kind::Acknowledge)?)?;
    bytes = bytes.checked_add(frame(&limits, Kind::Cancel)?)?;
    bytes = bytes.checked_add(grants.checked_mul(frame(&limits, Kind::Grant)?)?)?;
    let frames = 3_u32.checked_add(sizes.inbox)?.checked_add(sizes.calls)?.checked_add(sizes.turns)?;
    let frames = frames.checked_add(grants)?.checked_mul(sizes.slots)?;
    Some(Cap { bytes: bytes.checked_mul(sizes.slots)?, frames })
}

#[must_use]
pub fn worker_cap(sizes: &Sizes) -> Option<WorkerCap> {
    let limits = limits(sizes)?;
    let mut run = sizes.turns.checked_mul(frame(&limits, Kind::Turn)?)?;
    run = run.checked_add(frame(&limits, Kind::Answer)?)?;
    run = run.checked_add(sizes.calls.checked_mul(frame(&limits, Kind::Call)?)?)?;
    let notices = frame(&limits, Kind::Rejected)?.checked_add(frame(&limits, Kind::Exhausted)?)?;
    let mut bytes = frame(&limits, Kind::Hello)?.checked_add(sizes.slots.checked_mul(run)?)?;
    bytes = bytes.checked_add(sizes.accounts.checked_mul(notices)?)?;
    // LongDone is the shortest fact: task + attempt + activity tag + header.
    let fact_frames = sizes.facts / 25;
    let run_frames = sizes.turns.checked_add(1)?.checked_add(sizes.calls)?;
    let mut frames = 1_u32.checked_add(sizes.slots.checked_mul(run_frames)?)?;
    frames = frames.checked_add(sizes.accounts.checked_mul(2)?)?.checked_add(fact_frames)?;
    Some(WorkerCap { cap: Cap { bytes: bytes.checked_add(sizes.facts)?, frames }, facts: sizes.facts })
}
