//! Party batch values (protocol/store.md, 5.1).
use super::pieces::{decode_spec_parameters, encode_spec_parameters, shared_sum, split};
use super::policy::{decode_authority, decode_wake_policy, encode_authority, encode_wake_policy};
use super::{Box, List, Reader, Writer, count, domain, problem, wire};
pub(super) fn encode(batch: &[domain::Member], limits: &wire::Limits) -> Result<List<wire::Member>, wire::Problem> {
    let mut members = List::with_capacity(count(batch.len())?);
    for member in batch {
        let domain::TaskExecutor::Agent { charter } = member.executor;
        let mut numbers = List::with_capacity(count(member.dependencies.len())?);
        for &at in &member.dependencies {
            numbers.push(u64::from(at)).expect("dependency count");
        }
        members
            .push(wire::Member::new(
                limits,
                wire::MemberParts {
                    executor: wire::TaskExecutor::Agent(wire::TaskExecutorAgent::new(
                        limits,
                        wire::TaskExecutorAgentParts { charter },
                    )?),
                    words: member.words.clone(),
                    parameters: encode_spec_parameters(
                        &member.parameters,
                        limits,
                        shared_sum(limits),
                        limits.member_parameters,
                    )?,
                    contract: encode_contract(&member.contract, limits)?,
                    authority: encode_authority(&member.authority, limits)?,
                    wake: encode_wake_policy(&member.wake, limits)?,
                    dependencies: super::pieces::encode_numbers(
                        numbers.as_slice(),
                        limits,
                        shared_sum(limits).checked_mul(2).ok_or(problem())?,
                        limits.member_dependencies,
                    )?,
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
            wake,
            dependencies,
            tracked,
            label,
            flags,
        } = member.into_parts();
        let mut numbers = List::with_capacity(count(dependencies.len())?);
        for at in
            super::pieces::decode_numbers(dependencies, limits, shared_sum(limits).checked_mul(2).ok_or(problem())?)?
        {
            numbers
                .push(match u32::try_from(at) {
                    Ok(at) => at,
                    Err(_) => return Err(problem()),
                })
                .expect("dependency count");
        }
        let wire::FlagsParts { conversation, listed, retention } = flags.into_parts();
        members
            .push(domain::Member {
                executor: match executor {
                    wire::TaskExecutor::Agent(value) => {
                        domain::TaskExecutor::Agent { charter: value.into_parts().charter }
                    }
                },
                words,
                parameters: decode_spec_parameters(parameters, limits, shared_sum(limits))?,
                contract: decode_contract(contract, limits)?,
                authority: decode_authority(authority, limits)?,
                wake: decode_wake_policy(wake),
                dependencies: numbers.into_boxed(),
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
