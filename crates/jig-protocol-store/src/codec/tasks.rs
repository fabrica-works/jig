//! Tasks rows translated to their generated family (protocol/store.md, 5.1).
pub mod phase;
pub mod proposals;
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
        || domain.dependencies > wire::CEILINGS.member_dependencies
        || domain.holdings > wire::CEILINGS.member_holdings
        || domain.batch > wire::CEILINGS.recurring_template_batch
        || domain.result_bytes > wire::CEILINGS.result_field_name
        || domain.result_bytes > wire::CEILINGS.result_field_value
        || domain.result_bytes > wire::CEILINGS.approved_state
        || domain.result_bytes > wire::CEILINGS.approved_words_some
        || domain.result_bytes > wire::CEILINGS.report_result_words
        || domain.result_bytes > wire::CEILINGS.verdict_result_fields
        || domain.result_bytes > wire::CEILINGS.verdict_result_words
        || domain.result_bytes > wire::CEILINGS.change_result_words
        || domain.result_bytes > wire::CEILINGS.failure_result_reason
        || domain.result_bytes > wire::CEILINGS.cancelled_reason
        || domain.message_bytes > wire::CEILINGS.rejected_proposal_reason
        || domain.message_bytes > wire::CEILINGS.amendment_instructions_some
        || domain.dependencies > wire::CEILINGS.amendment_dependencies_some
        || domain.message_bytes > wire::CEILINGS.amendment_reason
        || domain.batch > wire::CEILINGS.proposed_batch_members
        || domain.message_bytes > wire::CEILINGS.proposal_reason
        || domain.message_bytes > wire::CEILINGS.settings_instructions_some
        || domain.dependencies > wire::CEILINGS.settings_dependencies
        || domain.result_bytes.max(domain.message_bytes) > wire::CEILINGS.history_reason
        || domain.message_bytes > wire::CEILINGS.rejected_person_proposal_reason
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
        member_dependencies: domain.dependencies,
        member_holdings: domain.holdings,
        recurring_template_batch: domain.batch,
        result_field_name: domain.result_bytes,
        result_field_value: domain.result_bytes,
        approved_state: domain.result_bytes,
        approved_words_some: domain.result_bytes,
        report_result_words: domain.result_bytes,
        verdict_result_fields: domain.result_bytes,
        verdict_result_words: domain.result_bytes,
        change_result_words: domain.result_bytes,
        failure_result_reason: domain.result_bytes,
        cancelled_reason: domain.result_bytes,
        rejected_proposal_reason: domain.message_bytes,
        amendment_instructions_some: domain.message_bytes,
        amendment_dependencies_some: domain.dependencies,
        amendment_reason: domain.message_bytes,
        proposed_batch_members: domain.batch,
        proposal_reason: domain.message_bytes,
        settings_instructions_some: domain.message_bytes,
        settings_dependencies: domain.dependencies,
        history_reason: domain.result_bytes.max(domain.message_bytes),
        rejected_person_proposal_reason: domain.message_bytes,
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
        domain::Stored::History(row) => wire::Value::History(Box::new(proposals::history(row, limits)?)),
        domain::Stored::Milestone(row) => wire::Value::Milestone(proposals::milestone(row, limits)?),
        domain::Stored::PersonProposal(row) => wire::Value::PersonProposal(proposals::person_proposal(row, limits)?),
        domain::Stored::DecidedProposal(row) => wire::Value::DecidedProposal(proposals::person_proposal(row, limits)?),
        domain::Stored::Update(_)
        | domain::Stored::Goal(_)
        | domain::Stored::Result(_)
        | domain::Stored::ConversationNewest(_)
        | domain::Stored::ConversationOldest(_)
        | domain::Stored::Activity(_)
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
        wire::Value::History(value) => domain::Stored::History(proposals::read_history(*value, limits)?),
        wire::Value::Milestone(value) => domain::Stored::Milestone(proposals::read_milestone(value, limits)?),
        wire::Value::PersonProposal(value) => {
            domain::Stored::PersonProposal(Box::new(proposals::read_person_proposal(value, limits)?))
        }
        wire::Value::DecidedProposal(value) => {
            domain::Stored::DecidedProposal(Box::new(proposals::read_person_proposal(value, limits)?))
        }
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
