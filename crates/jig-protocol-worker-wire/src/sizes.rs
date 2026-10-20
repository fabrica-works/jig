//! Channel bounds from the owners' numbers (protocol/channel.md, 9 and 11).
use crate::{Kind, v3};

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
pub fn limits(sizes: &Sizes) -> Option<v3::Limits> {
    let limits = v3::Limits {
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
        invalid_start_why: sizes.detail,
        // A provider's code and reason are smith's text, within the detail's bound.
        account_refused_code: sizes.detail.min(v3::CEILINGS.account_refused_code),
        account_refused_reason: sizes.detail.min(v3::CEILINGS.account_refused_reason),
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
    v3::check_limits(&limits).ok()?;
    Some(limits)
}

#[must_use]
pub fn largest_body(sizes: &Sizes, kind: Kind) -> Option<u32> {
    body(&limits(sizes)?, kind)
}

fn body(limits: &v3::Limits, kind: Kind) -> Option<u32> {
    let bytes = match kind {
        Kind::Hello => v3::Hello::worst_case_bytes(limits),
        Kind::Turn => v3::Turn::worst_case_bytes(limits),
        Kind::Call => v3::Call::worst_case_bytes(limits),
        Kind::Answer => v3::Answer::worst_case_bytes(limits),
        Kind::Fact => v3::Fact::worst_case_bytes(limits),
        Kind::Unauthorized => v3::Unauthorized::worst_case_bytes(limits),
        Kind::Limited => v3::Limited::worst_case_bytes(limits),
        Kind::AccountRefused => v3::AccountRefused::worst_case_bytes(limits),
        Kind::Assignment => v3::Assignment::worst_case_bytes(limits),
        Kind::Message => v3::Inbound::worst_case_bytes(limits),
        Kind::Relayed => v3::Relayed::worst_case_bytes(limits),
        Kind::TurnKept => v3::TurnKept::worst_case_bytes(limits),
        Kind::TurnBusy => v3::TurnBusy::worst_case_bytes(limits),
        Kind::Acknowledge => v3::Acknowledge::worst_case_bytes(limits),
        Kind::Cancel => v3::Cancel::worst_case_bytes(limits),
        Kind::Grant => v3::Granted::worst_case_bytes(limits),
    }?;
    u32::try_from(bytes).ok()
}

fn frame(limits: &v3::Limits, kind: Kind) -> Option<u32> {
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
    let notices = frame(&limits, Kind::Unauthorized)?
        .checked_add(frame(&limits, Kind::Limited)?)?
        .checked_add(frame(&limits, Kind::AccountRefused)?)?;
    let mut bytes = frame(&limits, Kind::Hello)?.checked_add(sizes.slots.checked_mul(run)?)?;
    bytes = bytes.checked_add(sizes.accounts.checked_mul(notices)?)?;
    // LongDone is the shortest fact: task + attempt + activity tag + header.
    let fact_frames = sizes.facts / 25;
    let run_frames = sizes.turns.checked_add(1)?.checked_add(sizes.calls)?;
    let mut frames = 1_u32.checked_add(sizes.slots.checked_mul(run_frames)?)?;
    frames = frames.checked_add(sizes.accounts.checked_mul(3)?)?.checked_add(fact_frames)?;
    Some(WorkerCap { cap: Cap { bytes: bytes.checked_add(sizes.facts)?, frames }, facts: sizes.facts })
}
