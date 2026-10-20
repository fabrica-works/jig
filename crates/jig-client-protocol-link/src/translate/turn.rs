//! Complete single-turn admission before allocating its owned envelope (client.md, 7).
use super::Undecodable;
use crate::Limits;
use jig_client_domain::contract as c;
use jig_client_protocol_wire::{VERSION, v10 as w};
use skein_lib::{Reader, Wall};

// Schema client-v10: ReadAnswer::Value, Owner::Jig, ReadValue::Turn,
// TurnBytes::Carried. Generated-encoding boundary tests check this inspector.
const VALUE: u8 = 1;
const JIG: u8 = 0;
const TURN: u8 = 25;
const CARRIED: u8 = 0;

/// Borrows every byte, including an oversized payload, without decoding or copying it.
pub(crate) fn inspect(
    body: &[u8],
    identity: (u64, u64, u32),
    limits: &Limits,
) -> Result<Option<c::ReadNone>, Undecodable> {
    let mut reader = Reader::new(body);
    match reader.u8() {
        Some(VALUE) => {}
        Some(0 | 2) => return Ok(None),
        _ => return Err(Undecodable::Malformed),
    }
    let malformed = Undecodable::Malformed;
    if reader.u8() != Some(JIG) || reader.u32() != Some(VERSION) {
        return Err(malformed);
    }
    let length = reader.u32().ok_or(malformed)?;
    if length > w::CEILINGS.owned_record || length != reader.remaining() {
        return Err(malformed);
    }
    if reader.u8() != Some(TURN) {
        return Err(malformed);
    }
    let found = (reader.u64().ok_or(malformed)?, reader.u64().ok_or(malformed)?, reader.u32().ok_or(malformed)?);
    if found != identity {
        return Err(malformed);
    }
    reader.u64().ok_or(malformed)?; // cumulative spend
    match reader.u8() {
        Some(0) => {}
        Some(1) => {
            reader.u64().ok_or(malformed)?;
        }
        _ => return Err(malformed),
    }
    reader.u64().ok_or(malformed)?; // archive time
    let bytes = match reader.u8() {
        Some(CARRIED) => {
            let bytes = reader.u32().ok_or(malformed)?;
            if bytes > w::CEILINGS.turn_carried_bytes {
                return Err(malformed);
            }
            reader.bytes(bytes).ok_or(malformed)?;
            u64::from(bytes)
        }
        Some(1) => {
            let bytes = reader.u64().ok_or(malformed)?;
            if !reader.is_empty() {
                return Err(malformed);
            }
            return Ok(Some(c::ReadNone::TooLarge { bytes }));
        }
        _ => return Err(malformed),
    };
    if !reader.is_empty() {
        return Err(malformed);
    }
    if u64::from(length) > limits.client.document_bytes || bytes > u64::from(limits.wire.turn_carried_bytes) {
        return Ok(Some(c::ReadNone::TooLarge { bytes }));
    }
    Ok(None)
}

pub(crate) fn archive(record: w::ArchivedTurn, limits: &Limits) -> Result<c::ArchivedTurn, Undecodable> {
    let w::TurnBytes::Carried(carried) = record.turn() else { return Err(Undecodable::TooLarge) };
    let bound = limits.wire.turn_carried_bytes.min(u32::try_from(limits.client.document_bytes).expect("checked limit"));
    let c::TurnRead::Decoded(turn) = decode(carried.bytes(), limits, bound) else {
        return Err(Undecodable::Malformed);
    };
    Ok(c::ArchivedTurn {
        task: record.task(),
        name: c::TurnName { attempt: record.name().attempt(), number: record.name().number() },
        cumulative: record.cumulative(),
        read: *record.read(),
        at: Wall::from_nanos(record.at()),
        turn: c::TurnRead::Decoded(turn),
    })
}

pub(crate) fn decode(value: &[u8], limits: &Limits, bytes: u32) -> c::TurnRead {
    let turn = c::TurnLimits { messages: limits.client.rows, blocks: limits.client.rows, text: limits.client.text };
    let source_bytes = u64::try_from(value.len()).expect("payload length fits u64");
    let limits = jig_protocol_transcript::Limits::new(bytes, turn);
    let why = match jig_protocol_transcript::decode(&limits, value) {
        Ok(turn) => return c::TurnRead::Decoded(turn),
        Err(jig_protocol_transcript::Undecodable::Version { found }) => c::TurnProblem::Version { found },
        Err(jig_protocol_transcript::Undecodable::Malformed | jig_protocol_transcript::Undecodable::Unknown) => {
            c::TurnProblem::Malformed
        }
        Err(jig_protocol_transcript::Undecodable::TooLarge) => c::TurnProblem::TooLarge { bytes: source_bytes },
    };
    c::TurnRead::Undecodable { why }
}
