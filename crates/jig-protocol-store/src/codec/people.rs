//! People rows translated exhaustively (protocol/store.md, 5.1).
use crate::people_v1 as wire;
use alloc::boxed::Box;
use jig_domain_core as core;
use jig_domain_people as domain;
use skein_lib::{Duration, List, Reader, Wall, Writer};
mod asks;
mod policy;
pub use asks::{decode_ask, encode_ask};
use policy::{
    decode_amendment, decode_executor, decode_grant, decode_parameter, decode_pattern, decode_permission_role,
    decode_policy_change, decode_policy_role, decode_policy_value, decode_requirement, decode_resource_scope,
    encode_amendment, encode_executor, encode_grant, encode_parameter, encode_pattern, encode_permission_role,
    encode_policy_change, encode_policy_role, encode_policy_value, encode_requirement, encode_resource_scope,
};
#[must_use]
pub fn limits(domain: &core::Limits) -> Option<wire::Limits> {
    let people = &domain.people;
    let authority = &domain.authority;
    let policy_bytes = u32::try_from(domain.policy_bytes).ok()?;
    let limits = wire::Limits {
        identity_key_subject: people.identity_bytes,
        identity_login: people.identity_bytes,
        identity_name: people.identity_bytes,
        last_exact_value: people.amendment_bytes.max(authority.segment_bytes),
        last_open_value: people.amendment_bytes.max(authority.segment_bytes),
        pattern_segments: members(people.amendment_bytes, size_of::<Box<[u8]>>())?.max(authority.segments),
        delegation_kinds: members(people.amendment_bytes, size_of::<domain::Executor>())?.max(authority.executors),
        authority_grants: members(people.amendment_bytes, size_of::<domain::Grant>())?.max(authority.grants),
        authority_note_resources: members(people.amendment_bytes, size_of::<domain::ResourceScope>())?
            .max(authority.grants),
        policy_value_projections: authority.grants,
        policy_value_roles: authority.roles,
        policy_value_requirements: authority.requirements,
        policy_value_permissions: members(policy_bytes, size_of::<domain::PermissionRole>())?,
        policy_change_requirements_requirements: members(people.amendment_bytes, size_of::<domain::Requirement>())?,
        policy_change_projections_grants: members(people.amendment_bytes, size_of::<domain::Grant>())?,
        policy_change_permissions_mappings: members(people.amendment_bytes, size_of::<domain::PermissionRole>())?,
        note_change_correct_description: people.amendment_bytes,
        note_change_correct_body: people.amendment_bytes,
        note_change_correct_references: members(people.amendment_bytes, size_of::<u64>())?,
        resource_name_path: members(people.amendment_bytes, size_of::<Box<[u8]>>())?,
        adoption_options: members(people.amendment_bytes, size_of::<Box<[u8]>>())?,
        person_result_approved_state: people.words,
        person_result_approved_words_some: people.words,
        person_result_report_words: people.words,
        person_result_verdict_words: people.words,
        person_result_failure_reason: people.words,
        proposal_decision_reject_reason: people.words,
        escalation_decision_reject_reason: people.words,
        parameter_bytes_value: people.amendment_bytes,
        spec_words: people.amendment_bytes,
        spec_parameters: members(people.amendment_bytes, size_of::<domain::Parameter>())?,
        spec_inputs: members(people.amendment_bytes, size_of::<u64>())?,
        amendment_instructions_some: people.amendment_bytes,
        amendment_dependencies_some: members(people.amendment_bytes, size_of::<u64>())?,
        amendment_reason: people.words,
        ask_close_disposals: domain::disposal_capacity(people),
        ask_make_service_name: people.words,
        ask_set_goal_spec: people.words,
        ask_cancel_reason: people.words,
        ask_move_reason: people.words,
        ask_say_words: people.words,
        ask_answer_question_words: people.words,
        ask_prioritise_goals: people.goals,
        ask_set_roles_holdings: people.holdings,
        ask_start_chat_words: people.words,
        stored_roles_holdings: people.holdings,
        segment_bytes: people.amendment_bytes.max(authority.segment_bytes),
    };
    if within(&limits) { Some(limits) } else { None }
}

fn within(limits: &wire::Limits) -> bool {
    for fits in [
        limits.identity_key_subject <= wire::CEILINGS.identity_key_subject,
        limits.identity_login <= wire::CEILINGS.identity_login,
        limits.identity_name <= wire::CEILINGS.identity_name,
        limits.last_exact_value <= wire::CEILINGS.last_exact_value,
        limits.last_open_value <= wire::CEILINGS.last_open_value,
        limits.pattern_segments <= wire::CEILINGS.pattern_segments,
        limits.delegation_kinds <= wire::CEILINGS.delegation_kinds,
        limits.authority_grants <= wire::CEILINGS.authority_grants,
        limits.authority_note_resources <= wire::CEILINGS.authority_note_resources,
        limits.policy_value_projections <= wire::CEILINGS.policy_value_projections,
        limits.policy_value_roles <= wire::CEILINGS.policy_value_roles,
        limits.policy_value_requirements <= wire::CEILINGS.policy_value_requirements,
        limits.policy_value_permissions <= wire::CEILINGS.policy_value_permissions,
        limits.policy_change_requirements_requirements <= wire::CEILINGS.policy_change_requirements_requirements,
        limits.policy_change_projections_grants <= wire::CEILINGS.policy_change_projections_grants,
        limits.policy_change_permissions_mappings <= wire::CEILINGS.policy_change_permissions_mappings,
        limits.note_change_correct_description <= wire::CEILINGS.note_change_correct_description,
        limits.note_change_correct_body <= wire::CEILINGS.note_change_correct_body,
        limits.note_change_correct_references <= wire::CEILINGS.note_change_correct_references,
        limits.resource_name_path <= wire::CEILINGS.resource_name_path,
        limits.adoption_options <= wire::CEILINGS.adoption_options,
        limits.person_result_approved_state <= wire::CEILINGS.person_result_approved_state,
        limits.person_result_approved_words_some <= wire::CEILINGS.person_result_approved_words_some,
        limits.person_result_report_words <= wire::CEILINGS.person_result_report_words,
        limits.person_result_verdict_words <= wire::CEILINGS.person_result_verdict_words,
        limits.person_result_failure_reason <= wire::CEILINGS.person_result_failure_reason,
        limits.proposal_decision_reject_reason <= wire::CEILINGS.proposal_decision_reject_reason,
        limits.escalation_decision_reject_reason <= wire::CEILINGS.escalation_decision_reject_reason,
        limits.parameter_bytes_value <= wire::CEILINGS.parameter_bytes_value,
        limits.spec_words <= wire::CEILINGS.spec_words,
        limits.spec_parameters <= wire::CEILINGS.spec_parameters,
        limits.spec_inputs <= wire::CEILINGS.spec_inputs,
        limits.amendment_instructions_some <= wire::CEILINGS.amendment_instructions_some,
        limits.amendment_dependencies_some <= wire::CEILINGS.amendment_dependencies_some,
        limits.amendment_reason <= wire::CEILINGS.amendment_reason,
        limits.ask_close_disposals <= wire::CEILINGS.ask_close_disposals,
        limits.ask_make_service_name <= wire::CEILINGS.ask_make_service_name,
        limits.ask_set_goal_spec <= wire::CEILINGS.ask_set_goal_spec,
        limits.ask_cancel_reason <= wire::CEILINGS.ask_cancel_reason,
        limits.ask_move_reason <= wire::CEILINGS.ask_move_reason,
        limits.ask_say_words <= wire::CEILINGS.ask_say_words,
        limits.ask_answer_question_words <= wire::CEILINGS.ask_answer_question_words,
        limits.ask_prioritise_goals <= wire::CEILINGS.ask_prioritise_goals,
        limits.ask_set_roles_holdings <= wire::CEILINGS.ask_set_roles_holdings,
        limits.ask_start_chat_words <= wire::CEILINGS.ask_start_chat_words,
        limits.stored_roles_holdings <= wire::CEILINGS.stored_roles_holdings,
        limits.segment_bytes <= wire::CEILINGS.segment_bytes,
    ] {
        if !fits {
            return false;
        }
    }
    true
}
pub fn encode(row: &domain::Stored, limits: &wire::Limits) -> Result<Box<[u8]>, wire::Problem> {
    let value = match row {
        domain::Stored::Person { number, identity } => wire::Value::Person(wire::StoredPerson::new(
            limits,
            wire::StoredPersonParts { number: *number, identity: encode_identity(identity, limits)? },
        )?),
        domain::Stored::ReadResults { party, through } => wire::Value::ReadResults(wire::StoredReadResults::new(
            limits,
            wire::StoredReadResultsParts { party: *party, through: *through },
        )?),
        domain::Stored::ReadChat { party, task, through } => wire::Value::ReadChat(wire::StoredReadChat::new(
            limits,
            wire::StoredReadChatParts { party: *party, task: *task, through: *through },
        )?),
        domain::Stored::SignIn { number, person, expires } => wire::Value::SignIn(wire::StoredSignIn::new(
            limits,
            wire::StoredSignInParts { number: *number, person: *person, expires: expires.as_nanos() },
        )?),
        domain::Stored::Roles { project, holdings } => wire::Value::Roles(wire::StoredRoles::new(
            limits,
            wire::StoredRolesParts { project: *project, holdings: encode_holdings(holdings, limits)? },
        )?),
        domain::Stored::Policy { project, value } => wire::Value::Policy(wire::StoredPolicy::new(
            limits,
            wire::StoredPolicyParts { project: *project, value: encode_policy_value(value, limits)? },
        )?),
        domain::Stored::Answer { key: _, ask: _, outcome: _, at: _ } => return Err(problem()),
    };
    let row = wire::Row::new(limits, wire::RowParts { value })?;
    let mut writer = Writer::new(usize::try_from(row.measure()).expect("row length fits usize"));
    row.encode(&mut writer).expect("measured room");
    Ok(writer.finish())
}
pub fn decode(bytes: &[u8], limits: &wire::Limits) -> Result<domain::Stored, wire::Problem> {
    let wire::RowParts { value } = wire::Row::decode(limits, &mut Reader::new(bytes))?.into_parts();
    let row = match value {
        wire::Value::Person(value) => {
            let wire::StoredPersonParts { number, identity } = value.into_parts();
            domain::Stored::Person { number, identity: decode_identity(identity) }
        }
        wire::Value::ReadResults(value) => {
            let wire::StoredReadResultsParts { party, through } = value.into_parts();
            domain::Stored::ReadResults { party, through }
        }
        wire::Value::ReadChat(value) => {
            let wire::StoredReadChatParts { party, task, through } = value.into_parts();
            domain::Stored::ReadChat { party, task, through }
        }
        wire::Value::SignIn(value) => {
            let wire::StoredSignInParts { number, person, expires } = value.into_parts();
            domain::Stored::SignIn { number, person, expires: Wall::from_nanos(expires) }
        }
        wire::Value::Roles(value) => {
            let wire::StoredRolesParts { project, holdings } = value.into_parts();
            domain::Stored::Roles { project, holdings: decode_holdings(holdings) }
        }
        wire::Value::Policy(value) => {
            let wire::StoredPolicyParts { project, value } = value.into_parts();
            domain::Stored::Policy { project, value: decode_policy_value(value) }
        }
    };
    Ok(row)
}

fn problem() -> wire::Problem {
    wire::Problem { path: wire::Path::RowValue, reason: skein_codec::Reason::Bound }
}
fn count(length: usize) -> Result<u32, wire::Problem> {
    match u32::try_from(length) {
        Ok(length) => Ok(length),
        Err(_) => Err(problem()),
    }
}
fn members(bytes: u32, size: usize) -> Option<u32> {
    bytes.checked_div(u32::try_from(size).ok()?)
}
pub(super) fn encode_role(value: domain::Role, limits: &wire::Limits) -> Result<wire::Role, wire::Problem> {
    Ok(match value {
        domain::Role::Owner => wire::Role::Owner,
        domain::Role::Maintainer => wire::Role::Maintainer,
        domain::Role::Member => wire::Role::Member,
        domain::Role::Observer => wire::Role::Observer,
        domain::Role::Policy { role } => {
            wire::Role::Policy(wire::RolePolicy::new(limits, wire::RolePolicyParts { role })?)
        }
    })
}
pub(super) fn decode_role(value: wire::Role) -> domain::Role {
    match value {
        wire::Role::Owner => domain::Role::Owner,
        wire::Role::Maintainer => domain::Role::Maintainer,
        wire::Role::Member => domain::Role::Member,
        wire::Role::Observer => domain::Role::Observer,
        wire::Role::Policy(value) => {
            let wire::RolePolicyParts { role } = value.into_parts();
            domain::Role::Policy { role }
        }
    }
}

pub(super) fn encode_holding(value: &domain::Holding, limits: &wire::Limits) -> Result<wire::Holding, wire::Problem> {
    let domain::Holding { person, role } = value;
    wire::Holding::new(limits, wire::HoldingParts { person: *person, role: encode_role(*role, limits)? })
}
pub(super) fn decode_holding(value: wire::Holding) -> domain::Holding {
    let wire::HoldingParts { person, role } = value.into_parts();
    domain::Holding { person, role: decode_role(role) }
}

pub(super) fn encode_identity_key(
    value: &domain::IdentityKey,
    limits: &wire::Limits,
) -> Result<wire::IdentityKey, wire::Problem> {
    let domain::IdentityKey { provider, subject } = value;
    wire::IdentityKey::new(limits, wire::IdentityKeyParts { provider: *provider, subject: subject.clone() })
}
pub(super) fn decode_identity_key(value: wire::IdentityKey) -> domain::IdentityKey {
    let wire::IdentityKeyParts { provider, subject } = value.into_parts();
    domain::IdentityKey { provider, subject }
}

pub(super) fn encode_identity(
    value: &domain::Identity,
    limits: &wire::Limits,
) -> Result<wire::Identity, wire::Problem> {
    let domain::Identity { key, login, name } = value;
    wire::Identity::new(
        limits,
        wire::IdentityParts { key: encode_identity_key(key, limits)?, login: login.clone(), name: name.clone() },
    )
}
pub(super) fn decode_identity(value: wire::Identity) -> domain::Identity {
    let wire::IdentityParts { key, login, name } = value.into_parts();
    domain::Identity { key: decode_identity_key(key), login, name }
}

pub fn encode_request_key(
    value: &domain::RequestKey,
    limits: &wire::Limits,
) -> Result<wire::RequestKey, wire::Problem> {
    let domain::RequestKey { person, key } = value;
    wire::RequestKey::new(limits, wire::RequestKeyParts { person: *person, key: *key })
}
#[must_use]
pub fn decode_request_key(value: wire::RequestKey) -> domain::RequestKey {
    let wire::RequestKeyParts { person, key } = value.into_parts();
    domain::RequestKey { person, key }
}

fn encode_delegation_kinds(
    values: &[domain::Executor],
    limits: &wire::Limits,
) -> Result<List<wire::Executor>, wire::Problem> {
    let mut result = List::with_capacity(count(values.len())?);
    for value in values {
        result.push(encode_executor(*value, limits)?).expect("same number of members");
    }
    Ok(result)
}
fn decode_delegation_kinds(values: List<wire::Executor>) -> Box<[domain::Executor]> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        result.push(decode_executor(value)).expect("same number of members");
    }
    result.into_boxed()
}

fn encode_authority_note_resources(
    values: &[domain::ResourceScope],
    limits: &wire::Limits,
) -> Result<List<wire::ResourceScope>, wire::Problem> {
    let mut result = List::with_capacity(count(values.len())?);
    for value in values {
        result.push(encode_resource_scope(value, limits)?).expect("same number of members");
    }
    Ok(result)
}
fn decode_authority_note_resources(values: List<wire::ResourceScope>) -> Box<[domain::ResourceScope]> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        result.push(decode_resource_scope(value)).expect("same number of members");
    }
    result.into_boxed()
}

fn encode_policy_value_roles(
    values: &[domain::PolicyRole],
    limits: &wire::Limits,
) -> Result<List<wire::PolicyRole>, wire::Problem> {
    let mut result = List::with_capacity(count(values.len())?);
    for value in values {
        result.push(encode_policy_role(value, limits)?).expect("same number of members");
    }
    Ok(result)
}
fn decode_policy_value_roles(values: List<wire::PolicyRole>) -> Box<[domain::PolicyRole]> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        result.push(decode_policy_role(value)).expect("same number of members");
    }
    result.into_boxed()
}

fn encode_spec_parameters(
    values: &[domain::Parameter],
    limits: &wire::Limits,
) -> Result<List<wire::Parameter>, wire::Problem> {
    let mut result = List::with_capacity(count(values.len())?);
    for value in values {
        result.push(encode_parameter(value, limits)?).expect("same number of members");
    }
    Ok(result)
}
fn decode_spec_parameters(values: List<wire::Parameter>) -> Box<[domain::Parameter]> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        result.push(decode_parameter(value)).expect("same number of members");
    }
    result.into_boxed()
}

fn decode_numbers(values: List<wire::Number64>) -> Box<[u64]> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        result.push(value.into_parts().value).expect("same number of members");
    }
    result.into_boxed()
}
fn encode_numbers(values: &[u64], limits: &wire::Limits) -> Result<List<wire::Number64>, wire::Problem> {
    let mut result = List::with_capacity(count(values.len())?);
    for value in values {
        result
            .push(wire::Number64::new(limits, wire::Number64Parts { value: *value })?)
            .expect("same number of members");
    }
    Ok(result)
}
fn decode_permissions(values: List<wire::PermissionRole>) -> Box<[domain::PermissionRole]> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        result.push(decode_permission_role(value)).expect("same number of members");
    }
    result.into_boxed()
}
fn encode_permissions(
    values: &[domain::PermissionRole],
    limits: &wire::Limits,
) -> Result<List<wire::PermissionRole>, wire::Problem> {
    let mut result = List::with_capacity(count(values.len())?);
    for value in values {
        result.push(encode_permission_role(*value, limits)?).expect("same number of members");
    }
    Ok(result)
}
fn decode_grants(values: List<wire::Grant>) -> Box<[domain::Grant]> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        result.push(decode_grant(value)).expect("same number of members");
    }
    result.into_boxed()
}
fn encode_grants(values: &[domain::Grant], limits: &wire::Limits) -> Result<List<wire::Grant>, wire::Problem> {
    let mut result = List::with_capacity(count(values.len())?);
    for value in values {
        result.push(encode_grant(value, limits)?).expect("same number of members");
    }
    Ok(result)
}
fn decode_requirements(values: List<wire::Requirement>) -> Box<[domain::Requirement]> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        result.push(decode_requirement(value)).expect("same number of members");
    }
    result.into_boxed()
}
fn encode_requirements(
    values: &[domain::Requirement],
    limits: &wire::Limits,
) -> Result<List<wire::Requirement>, wire::Problem> {
    let mut result = List::with_capacity(count(values.len())?);
    for value in values {
        result.push(encode_requirement(value, limits)?).expect("same number of members");
    }
    Ok(result)
}
fn decode_segments(values: List<wire::Segment>) -> Box<[Box<[u8]>]> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        result.push(value.into_parts().bytes).expect("same number of members");
    }
    result.into_boxed()
}
fn encode_segments(values: &[Box<[u8]>], limits: &wire::Limits) -> Result<List<wire::Segment>, wire::Problem> {
    let mut result = List::with_capacity(count(values.len())?);
    for value in values {
        result
            .push(wire::Segment::new(limits, wire::SegmentParts { bytes: value.clone() })?)
            .expect("same number of members");
    }
    Ok(result)
}
fn decode_holdings(values: List<wire::Holding>) -> Box<[domain::Holding]> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        result.push(decode_holding(value)).expect("same number of members");
    }
    result.into_boxed()
}
fn encode_holdings(values: &[domain::Holding], limits: &wire::Limits) -> Result<List<wire::Holding>, wire::Problem> {
    let mut result = List::with_capacity(count(values.len())?);
    for value in values {
        result.push(encode_holding(value, limits)?).expect("same number of members");
    }
    Ok(result)
}
