//! Party batch values (protocol/store.md, 5.1).
pub(crate) mod parts;
use super::pieces::{decode_spec_parameters, encode_spec_parameters, shared_sum, split};
use super::policy::{decode_authority, decode_wake_policy, encode_authority, encode_wake_policy};
use super::{Box, List, Reader, Writer, count, domain, problem, wire};
pub(super) fn encode(batch: &[domain::Member], limits: &wire::Limits) -> Result<List<wire::Member>, wire::Problem> {
    let mut members = List::with_capacity(count(batch.len())?);
    for member in batch {
        members
            .push(wire::Member::new(
                limits,
                wire::MemberParts {
                    llm: super::llm::encode_llm(member.llm.as_ref(), limits)?,
                    executor: parts::encode_executor(member.executor, limits)?,
                    words: member.words.clone(),
                    parameters: encode_spec_parameters(
                        &member.parameters,
                        limits,
                        shared_sum(limits),
                        limits.member_parameters,
                    )?,
                    inputs: super::pieces::encode_numbers(
                        &member.inputs,
                        limits,
                        shared_sum(limits),
                        limits.member_inputs,
                    )?,
                    deadline: match member.deadline {
                        domain::Deadline::Named => wire::Deadline::Named,
                        domain::Deadline::Span(value) => {
                            wire::Deadline::Span(wire::DeadlineSpan::new(limits, wire::DeadlineSpanParts { value })?)
                        }
                    },
                    budget: match member.budget {
                        domain::Budget::Named => wire::Budget::Named,
                        domain::Budget::AsAllowed { cap } => wire::Budget::AsAllowed(wire::BudgetAsAllowed::new(
                            limits,
                            wire::BudgetAsAllowedParts { cap },
                        )?),
                    },
                    named: super::pieces::encode_grant_piece(
                        &member.named,
                        limits,
                        shared_sum(limits),
                        limits.member_named,
                    )?,
                    contract: encode_contract(&member.contract, limits)?,
                    authority: encode_authority(&member.authority, limits)?,
                    wake: encode_wake_policy(&member.wake, limits)?,
                    dependencies: parts::encode_dependencies(&member.dependencies, limits)?,
                    tracked: member.tracked,
                    label: member.label,
                    flags: wire::Flags::new(
                        limits,
                        wire::FlagsParts {
                            conversation: member.flags.conversation,
                            listed: member.flags.listed,
                            retention: match member.flags.retention {
                                domain::Retention::Kept => wire::Retention::Kept,
                                domain::Retention::ForgottenOnceRead => wire::Retention::ForgottenOnceRead,
                            },
                        },
                    )?,
                },
            )?)
            .expect("member count");
    }
    Ok(members)
}
pub(super) fn decode(batch: List<wire::Member>, limits: &wire::Limits) -> Result<Box<[domain::Member]>, wire::Problem> {
    let mut members = List::with_capacity(batch.len());
    for member in batch.into_boxed() {
        let wire::MemberParts {
            executor,
            words,
            parameters,
            contract,
            authority,
            budget,
            deadline,
            inputs,
            named,
            wake,
            dependencies,
            tracked,
            label,
            flags,
            llm,
        } = member.into_parts();
        let wire::FlagsParts { conversation, listed, retention } = flags.into_parts();
        members
            .push(domain::Member {
                llm: super::llm::decode_llm(llm),
                deadline: match deadline {
                    wire::Deadline::Named => domain::Deadline::Named,
                    wire::Deadline::Span(value) => domain::Deadline::Span(value.value()),
                },
                executor: parts::decode_executor(executor),
                words,
                parameters: decode_spec_parameters(parameters, limits, shared_sum(limits))?,
                contract: decode_contract(contract, limits)?,
                authority: decode_authority(authority, limits)?,
                wake: decode_wake_policy(wake),
                dependencies: parts::decode_dependencies(dependencies, limits)?,
                inputs: super::pieces::decode_numbers(inputs, limits, shared_sum(limits))?,
                budget: match budget {
                    wire::Budget::Named => domain::Budget::Named,
                    wire::Budget::AsAllowed(value) => domain::Budget::AsAllowed { cap: value.into_parts().cap },
                },
                named: super::pieces::decode_grant_piece(named, limits, shared_sum(limits))?,
                tracked,
                label,
                flags: domain::Flags {
                    conversation,
                    listed,
                    retention: match retention {
                        wire::Retention::Kept => domain::Retention::Kept,
                        wire::Retention::ForgottenOnceRead => domain::Retention::ForgottenOnceRead,
                    },
                },
            })
            .expect("member count");
    }
    Ok(members.into_boxed())
}
fn encode_contract(value: &domain::Contract, limits: &wire::Limits) -> Result<wire::Contract, wire::Problem> {
    Ok(match value {
        domain::Contract::Change { connector, kind, words } => wire::Contract::Change(wire::ContractChange::new(
            limits,
            wire::ContractChangeParts { connector: *connector, kind: *kind, words: *words },
        )?),
        domain::Contract::Report { words } => {
            wire::Contract::Report(wire::ContractReport::new(limits, wire::ContractReportParts { words: *words })?)
        }
        domain::Contract::Verdict { choices } => {
            let mut items = List::with_capacity(count(choices.len())?);
            for choice in choices {
                let value = wire::Verdict::new(
                    limits,
                    wire::VerdictParts {
                        code: choice.code,
                        words: choice.words,
                        followups: choice.followups,
                        fields: encode_fields(&choice.fields, limits)?,
                    },
                )?;
                let mut writer = Writer::new(usize::try_from(value.measure()).expect("measured size"));
                value.encode(&mut writer).expect("measured verdict");
                items.push(writer.finish()).expect("choice count");
            }
            wire::Contract::Verdict(wire::ContractVerdict::new(
                limits,
                wire::ContractVerdictParts { choices: joined(items, limits.contract_verdict_choices)? },
            )?)
        }
    })
}
fn decode_contract(value: wire::Contract, limits: &wire::Limits) -> Result<domain::Contract, wire::Problem> {
    Ok(match value {
        wire::Contract::Change(value) => {
            let wire::ContractChangeParts { connector, kind, words } = value.into_parts();
            domain::Contract::Change { connector, kind, words }
        }
        wire::Contract::Report(value) => domain::Contract::Report { words: value.into_parts().words },
        wire::Contract::Verdict(value) => {
            let bytes = value.into_parts().choices;
            let mut reader = Reader::new(&bytes);
            let mut choices = List::with_capacity(count(bytes.len())?);
            for _ in 0..bytes.len() {
                let Some(bytes) = split(&mut reader)? else { break };
                let wire::VerdictParts { code, words, followups, fields } =
                    wire::Verdict::decode(limits, &mut Reader::new(&bytes))?.into_parts();
                choices
                    .push(domain::Verdict { code, words, followups, fields: decode_fields(fields, limits)? })
                    .expect("framed choice count");
            }
            domain::Contract::Verdict { choices: choices.into_boxed() }
        }
    })
}
fn encode_fields(fields: &[domain::FieldRule], limits: &wire::Limits) -> Result<Box<[u8]>, wire::Problem> {
    let mut items = List::with_capacity(count(fields.len())?);
    for field in fields {
        let value = wire::FieldRule::new(limits, wire::FieldRuleParts { name: field.name.clone(), max: field.max })?;
        let mut writer = Writer::new(usize::try_from(value.measure()).expect("measured size"));
        value.encode(&mut writer).expect("measured field");
        items.push(writer.finish()).expect("field count");
    }
    joined(items, limits.verdict_fields)
}
fn decode_fields(bytes: Box<[u8]>, limits: &wire::Limits) -> Result<Box<[domain::FieldRule]>, wire::Problem> {
    let mut reader = Reader::new(&bytes);
    let mut fields = List::with_capacity(count(bytes.len())?);
    for _ in 0..bytes.len() {
        let Some(bytes) = split(&mut reader)? else { break };
        let wire::FieldRuleParts { name, max } =
            wire::FieldRule::decode(limits, &mut Reader::new(&bytes))?.into_parts();
        fields.push(domain::FieldRule { name, max }).expect("framed field count");
    }
    Ok(fields.into_boxed())
}

fn joined(items: List<Box<[u8]>>, bound: u32) -> Result<Box<[u8]>, wire::Problem> {
    match crate::codec::pieces::join(items, bound) {
        Ok(bytes) => Ok(bytes),
        Err(_) => Err(problem()),
    }
}
