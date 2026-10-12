//! Tasks rows translated to their generated family (protocol/store.md, 5.1).
pub mod value;
use crate::tasks_v1 as wire;
use alloc::boxed::Box;
use jig_domain_tasks as domain;
use skein_lib::{Reader, Writer};

/// Translate the tasks child's bounds, refusing any schema ceiling.
#[must_use]
pub fn limits(domain: &domain::Limits) -> Option<wire::Limits> {
    if domain.hold_bytes > wire::CEILINGS.segment_bytes
        || domain.hold_segments > wire::CEILINGS.name_path
        || domain.spec_bytes > wire::CEILINGS.bytes_parameter_value
        || domain.spec_bytes > wire::CEILINGS.spec_words
        || domain.parameters > wire::CEILINGS.spec_parameters
        || domain.inputs > wire::CEILINGS.spec_inputs
        || domain.hold_bytes > wire::CEILINGS.part_segment_bytes
        || domain.hold_segments > wire::CEILINGS.part_path_segments
        || domain.result_bytes > wire::CEILINGS.field_rule_name
        || domain.result_bytes > wire::CEILINGS.verdict_fields
        || domain.result_bytes > wire::CEILINGS.approval_state
        || domain.contract_choices > wire::CEILINGS.choices_choices
        || domain.authority_bytes > wire::CEILINGS.authority_segment_bytes
        || domain.authority_segments > wire::CEILINGS.pattern_segments
        || domain.authority_grants > wire::CEILINGS.authority_grants
        || domain.authority_grants > wire::CEILINGS.authority_note_resources
        || domain.executor_kinds > wire::CEILINGS.delegation_kinds
    {
        return None;
    }
    Some(wire::Limits {
        segment_bytes: domain.hold_bytes,
        name_path: domain.hold_segments,
        bytes_parameter_value: domain.spec_bytes,
        spec_words: domain.spec_bytes,
        spec_parameters: domain.parameters,
        spec_inputs: domain.inputs,
        part_segment_bytes: domain.hold_bytes,
        part_path_segments: domain.hold_segments,
        field_rule_name: domain.result_bytes,
        verdict_fields: domain.result_bytes,
        approval_state: domain.result_bytes,
        choices_choices: domain.contract_choices,
        authority_segment_bytes: domain.authority_bytes,
        pattern_segments: domain.authority_segments,
        authority_grants: domain.authority_grants,
        authority_note_resources: domain.authority_grants,
        delegation_kinds: domain.executor_kinds,
    })
}

/// Encode one implemented tasks row within the configured bounds.
pub fn encode(row: &domain::Stored, limits: &wire::Limits) -> Result<Box<[u8]>, wire::Problem> {
    let value = match row {
        domain::Stored::Ledger(row) => wire::Value::Ledger(value::funding(row, limits)?),
        domain::Stored::ClosedLedger(row) => wire::Value::ClosedLedger(value::funding(row, limits)?),
        domain::Stored::Writer(domain::WriterSlot { number, resource, writer, lost }) => {
            let writer = match writer {
                domain::Writer::Run { task, attempt } => {
                    wire::Writer::Run(wire::Run::new(limits, wire::RunParts { task: *task, attempt: *attempt })?)
                }
                domain::Writer::Effect { entry } => {
                    wire::Writer::Effect(wire::Effect::new(limits, wire::EffectParts { entry: *entry })?)
                }
            };
            wire::Value::Writer(wire::WriterSlot::new(
                limits,
                wire::WriterSlotParts {
                    number: *number,
                    resource: value::name(resource, limits)?,
                    writer,
                    lost: *lost,
                },
            )?)
        }
        domain::Stored::Pool(domain::PoolSlots { number, pool, slots }) => wire::Value::Pool(wire::PoolSlots::new(
            limits,
            wire::PoolSlotsParts { number: *number, pool: value::name(pool, limits)?, slots: *slots },
        )?),
        domain::Stored::Update(_)
        | domain::Stored::Result(_)
        | domain::Stored::ConversationNewest(_)
        | domain::Stored::ConversationOldest(_)
        | domain::Stored::Activity(_)
        | domain::Stored::Milestone(_)
        | domain::Stored::PersonProposal(_)
        | domain::Stored::DecidedProposal(_)
        | domain::Stored::History(_)
        | domain::Stored::Live(_)
        | domain::Stored::Ended(_)
        | domain::Stored::Stub(_) => {
            return Err(wire::Problem { path: wire::Path::RowValue, reason: skein_codec::Reason::Tag });
        }
    };
    let row = wire::Row::new(limits, wire::RowParts { value })?;
    let mut writer = Writer::new(usize::try_from(row.measure()).expect("encoded row fits usize"));
    row.encode(&mut writer).expect("measured room");
    Ok(writer.finish())
}

/// Decode one tasks row, refusing another version or trailing bytes.
pub fn decode(bytes: &[u8], limits: &wire::Limits) -> Result<domain::Stored, wire::Problem> {
    let wire::RowParts { value } = wire::Row::decode(limits, &mut Reader::new(bytes))?.into_parts();
    Ok(match value {
        wire::Value::Ledger(value) => domain::Stored::Ledger(value::read_funding(value)),
        wire::Value::ClosedLedger(value) => domain::Stored::ClosedLedger(value::read_funding(value)),
        wire::Value::Writer(value) => {
            let wire::WriterSlotParts { number, resource, writer, lost } = value.into_parts();
            let writer = match writer {
                wire::Writer::Run(value) => {
                    let wire::RunParts { task, attempt } = value.into_parts();
                    domain::Writer::Run { task, attempt }
                }
                wire::Writer::Effect(value) => domain::Writer::Effect { entry: value.into_parts().entry },
            };
            domain::Stored::Writer(domain::WriterSlot {
                number,
                resource: value::read_name(resource, limits)?,
                writer,
                lost,
            })
        }
        wire::Value::Pool(value) => {
            let wire::PoolSlotsParts { number, pool, slots } = value.into_parts();
            domain::Stored::Pool(domain::PoolSlots { number, pool: value::read_name(pool, limits)?, slots })
        }
    })
}
