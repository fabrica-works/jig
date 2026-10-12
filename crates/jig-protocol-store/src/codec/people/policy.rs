//! Policy values retained by people (protocol/store.md, 5.1).
use super::{
    Duration, Wall, decode_authority_note_resources, decode_delegation_kinds, decode_grants, decode_numbers,
    decode_permissions, decode_policy_value_roles, decode_requirements, decode_segments, decode_spec_parameters,
    domain, encode_authority_note_resources, encode_delegation_kinds, encode_grants, encode_numbers,
    encode_permissions, encode_policy_value_roles, encode_requirements, encode_segments, encode_spec_parameters, wire,
};
pub(super) fn encode_last(value: &domain::Last, limits: &wire::Limits) -> Result<wire::Last, wire::Problem> {
    Ok(match value {
        domain::Last::Exact(value) => {
            wire::Last::Exact(wire::LastExact::new(limits, wire::LastExactParts { value: value.clone() })?)
        }
        domain::Last::Open(value) => {
            wire::Last::Open(wire::LastOpen::new(limits, wire::LastOpenParts { value: value.clone() })?)
        }
    })
}
pub(super) fn decode_last(value: wire::Last) -> domain::Last {
    match value {
        wire::Last::Exact(value) => {
            let wire::LastExactParts { value } = value.into_parts();
            domain::Last::Exact(value)
        }
        wire::Last::Open(value) => {
            let wire::LastOpenParts { value } = value.into_parts();
            domain::Last::Open(value)
        }
    }
}

pub(super) fn encode_pattern(value: &domain::Pattern, limits: &wire::Limits) -> Result<wire::Pattern, wire::Problem> {
    let domain::Pattern { segments, last } = value;
    wire::Pattern::new(
        limits,
        wire::PatternParts { segments: encode_segments(segments, limits)?, last: encode_last(last, limits)? },
    )
}
pub(super) fn decode_pattern(value: wire::Pattern) -> domain::Pattern {
    let wire::PatternParts { segments, last } = value.into_parts();
    domain::Pattern { segments: decode_segments(segments), last: decode_last(last) }
}

pub(super) fn encode_grant(value: &domain::Grant, limits: &wire::Limits) -> Result<wire::Grant, wire::Problem> {
    let domain::Grant { connector, kind, pattern } = value;
    wire::Grant::new(
        limits,
        wire::GrantParts { connector: *connector, kind: *kind, pattern: encode_pattern(pattern, limits)? },
    )
}
pub(super) fn decode_grant(value: wire::Grant) -> domain::Grant {
    let wire::GrantParts { connector, kind, pattern } = value.into_parts();
    domain::Grant { connector, kind, pattern: decode_pattern(pattern) }
}

pub(super) fn encode_resource_scope(
    value: &domain::ResourceScope,
    limits: &wire::Limits,
) -> Result<wire::ResourceScope, wire::Problem> {
    let domain::ResourceScope { connector, pattern } = value;
    wire::ResourceScope::new(
        limits,
        wire::ResourceScopeParts { connector: *connector, pattern: encode_pattern(pattern, limits)? },
    )
}
pub(super) fn decode_resource_scope(value: wire::ResourceScope) -> domain::ResourceScope {
    let wire::ResourceScopeParts { connector, pattern } = value.into_parts();
    domain::ResourceScope { connector, pattern: decode_pattern(pattern) }
}

pub(super) fn encode_executor(value: domain::Executor, limits: &wire::Limits) -> Result<wire::Executor, wire::Problem> {
    Ok(match value {
        domain::Executor::Charter(value) => {
            wire::Executor::Charter(wire::ExecutorCharter::new(limits, wire::ExecutorCharterParts { value })?)
        }
        domain::Executor::Procedure(value) => {
            wire::Executor::Procedure(wire::ExecutorProcedure::new(limits, wire::ExecutorProcedureParts { value })?)
        }
        domain::Executor::Role(value) => {
            wire::Executor::Role(wire::ExecutorRole::new(limits, wire::ExecutorRoleParts { value })?)
        }
    })
}
pub(super) fn decode_executor(value: wire::Executor) -> domain::Executor {
    match value {
        wire::Executor::Charter(value) => {
            let wire::ExecutorCharterParts { value } = value.into_parts();
            domain::Executor::Charter(value)
        }
        wire::Executor::Procedure(value) => {
            let wire::ExecutorProcedureParts { value } = value.into_parts();
            domain::Executor::Procedure(value)
        }
        wire::Executor::Role(value) => {
            let wire::ExecutorRoleParts { value } = value.into_parts();
            domain::Executor::Role(value)
        }
    }
}

pub(super) fn encode_delegation(
    value: &domain::Delegation,
    limits: &wire::Limits,
) -> Result<wire::Delegation, wire::Problem> {
    let domain::Delegation { kinds, tasks, depth } = value;
    wire::Delegation::new(
        limits,
        wire::DelegationParts { kinds: encode_delegation_kinds(kinds, limits)?, tasks: *tasks, depth: *depth },
    )
}
pub(super) fn decode_delegation(value: wire::Delegation) -> domain::Delegation {
    let wire::DelegationParts { kinds, tasks, depth } = value.into_parts();
    domain::Delegation { kinds: decode_delegation_kinds(kinds), tasks, depth }
}

#[expect(clippy::manual_map, reason = "step code translates optional fields through exhaustive matches")]
pub(super) fn encode_authority(
    value: &domain::Authority,
    limits: &wire::Limits,
) -> Result<wire::Authority, wire::Problem> {
    let domain::Authority { tools, grants, delegation, spend, deadline, notes, note_resources } = value;
    wire::Authority::new(
        limits,
        wire::AuthorityParts {
            tools: *tools,
            grants: encode_grants(grants, limits)?,
            delegation: encode_delegation(delegation, limits)?,
            spend: *spend,
            deadline: match deadline {
                Some(value) => Some(value.as_nanos()),
                None => None,
            },
            notes: *notes,
            note_resources: encode_authority_note_resources(note_resources, limits)?,
        },
    )
}
#[expect(clippy::manual_map, reason = "step code translates optional fields through exhaustive matches")]
pub(super) fn decode_authority(value: wire::Authority) -> domain::Authority {
    let wire::AuthorityParts { tools, grants, delegation, spend, deadline, notes, note_resources } = value.into_parts();
    domain::Authority {
        tools,
        grants: decode_grants(grants),
        delegation: decode_delegation(delegation),
        spend,
        deadline: match deadline {
            Some(value) => Some(Wall::from_nanos(value)),
            None => None,
        },
        notes,
        note_resources: decode_authority_note_resources(note_resources),
    }
}

pub(super) fn encode_judge(value: domain::Judge, limits: &wire::Limits) -> Result<wire::Judge, wire::Problem> {
    let domain::Judge { connector, requirement, parameters } = value;
    wire::Judge::new(limits, wire::JudgeParts { connector, requirement, parameters })
}
pub(super) fn decode_judge(value: wire::Judge) -> domain::Judge {
    let wire::JudgeParts { connector, requirement, parameters } = value.into_parts();
    domain::Judge { connector, requirement, parameters }
}

pub(super) fn encode_guard(value: &domain::Guard, limits: &wire::Limits) -> Result<wire::Guard, wire::Problem> {
    Ok(match value {
        domain::Guard::Guarded => wire::Guard::Guarded,
        domain::Guard::Observed { freshness } => wire::Guard::Observed(wire::GuardObserved::new(
            limits,
            wire::GuardObservedParts { freshness: freshness.as_nanos() },
        )?),
    })
}
pub(super) fn decode_guard(value: wire::Guard) -> domain::Guard {
    match value {
        wire::Guard::Guarded => domain::Guard::Guarded,
        wire::Guard::Observed(value) => {
            let wire::GuardObservedParts { freshness } = value.into_parts();
            domain::Guard::Observed { freshness: Duration::from_nanos(freshness) }
        }
    }
}

pub(super) fn encode_requirement(
    value: &domain::Requirement,
    limits: &wire::Limits,
) -> Result<wire::Requirement, wire::Problem> {
    let domain::Requirement { connector, kind, pattern, judge, guard, must_be_guarded } = value;
    wire::Requirement::new(
        limits,
        wire::RequirementParts {
            connector: *connector,
            kind: *kind,
            pattern: encode_pattern(pattern, limits)?,
            judge: encode_judge(*judge, limits)?,
            guard: encode_guard(guard, limits)?,
            must_be_guarded: *must_be_guarded,
        },
    )
}
pub(super) fn decode_requirement(value: wire::Requirement) -> domain::Requirement {
    let wire::RequirementParts { connector, kind, pattern, judge, guard, must_be_guarded } = value.into_parts();
    domain::Requirement {
        connector,
        kind,
        pattern: decode_pattern(pattern),
        judge: decode_judge(judge),
        guard: decode_guard(guard),
        must_be_guarded,
    }
}

pub(super) fn encode_permission_role(
    value: domain::PermissionRole,
    limits: &wire::Limits,
) -> Result<wire::PermissionRole, wire::Problem> {
    let domain::PermissionRole { connector, permission, role } = value;
    wire::PermissionRole::new(limits, wire::PermissionRoleParts { connector, permission, role })
}
pub(super) fn decode_permission_role(value: wire::PermissionRole) -> domain::PermissionRole {
    let wire::PermissionRoleParts { connector, permission, role } = value.into_parts();
    domain::PermissionRole { connector, permission, role }
}

pub(super) fn encode_policy_role(
    value: &domain::PolicyRole,
    limits: &wire::Limits,
) -> Result<wire::PolicyRole, wire::Problem> {
    let domain::PolicyRole { number, authority, period_spend, requests, decides } = value;
    wire::PolicyRole::new(
        limits,
        wire::PolicyRoleParts {
            number: *number,
            authority: encode_authority(authority, limits)?,
            period_spend: *period_spend,
            requests: *requests,
            decides: *decides,
        },
    )
}
pub(super) fn decode_policy_role(value: wire::PolicyRole) -> domain::PolicyRole {
    let wire::PolicyRoleParts { number, authority, period_spend, requests, decides } = value.into_parts();
    domain::PolicyRole { number, authority: decode_authority(authority), period_spend, requests, decides }
}

pub(super) fn encode_policy_value(
    value: &domain::PolicyValue,
    limits: &wire::Limits,
) -> Result<wire::PolicyValue, wire::Problem> {
    let domain::PolicyValue { period_spend, projections, roles, requirements, permissions } = value;
    wire::PolicyValue::new(
        limits,
        wire::PolicyValueParts {
            period_spend: *period_spend,
            projections: encode_grants(projections, limits)?,
            roles: encode_policy_value_roles(roles, limits)?,
            requirements: encode_requirements(requirements, limits)?,
            permissions: encode_permissions(permissions, limits)?,
        },
    )
}
pub(super) fn decode_policy_value(value: wire::PolicyValue) -> domain::PolicyValue {
    let wire::PolicyValueParts { period_spend, projections, roles, requirements, permissions } = value.into_parts();
    domain::PolicyValue {
        period_spend,
        projections: decode_grants(projections),
        roles: decode_policy_value_roles(roles),
        requirements: decode_requirements(requirements),
        permissions: decode_permissions(permissions),
    }
}

pub(super) fn encode_policy_change(
    value: &domain::PolicyChange,
    limits: &wire::Limits,
) -> Result<wire::PolicyChange, wire::Problem> {
    Ok(match value {
        domain::PolicyChange::ProjectSpend { period_spend } => {
            wire::PolicyChange::ProjectSpend(wire::PolicyChangeProjectSpend::new(
                limits,
                wire::PolicyChangeProjectSpendParts { period_spend: *period_spend },
            )?)
        }
        domain::PolicyChange::Role(value) => wire::PolicyChange::Role(wire::PolicyChangeRole::new(
            limits,
            wire::PolicyChangeRoleParts { value: encode_policy_role(value, limits)? },
        )?),
        domain::PolicyChange::Requirements { requirements } => {
            wire::PolicyChange::Requirements(wire::PolicyChangeRequirements::new(
                limits,
                wire::PolicyChangeRequirementsParts { requirements: encode_requirements(requirements, limits)? },
            )?)
        }
        domain::PolicyChange::Projections { grants } => {
            wire::PolicyChange::Projections(wire::PolicyChangeProjections::new(
                limits,
                wire::PolicyChangeProjectionsParts { grants: encode_grants(grants, limits)? },
            )?)
        }
        domain::PolicyChange::Permissions { mappings } => {
            wire::PolicyChange::Permissions(wire::PolicyChangePermissions::new(
                limits,
                wire::PolicyChangePermissionsParts { mappings: encode_permissions(mappings, limits)? },
            )?)
        }
    })
}
pub(super) fn decode_policy_change(value: wire::PolicyChange) -> domain::PolicyChange {
    match value {
        wire::PolicyChange::ProjectSpend(value) => {
            let wire::PolicyChangeProjectSpendParts { period_spend } = value.into_parts();
            domain::PolicyChange::ProjectSpend { period_spend }
        }
        wire::PolicyChange::Role(value) => {
            let wire::PolicyChangeRoleParts { value } = value.into_parts();
            domain::PolicyChange::Role(decode_policy_role(value))
        }
        wire::PolicyChange::Requirements(value) => {
            let wire::PolicyChangeRequirementsParts { requirements } = value.into_parts();
            domain::PolicyChange::Requirements { requirements: decode_requirements(requirements) }
        }
        wire::PolicyChange::Projections(value) => {
            let wire::PolicyChangeProjectionsParts { grants } = value.into_parts();
            domain::PolicyChange::Projections { grants: decode_grants(grants) }
        }
        wire::PolicyChange::Permissions(value) => {
            let wire::PolicyChangePermissionsParts { mappings } = value.into_parts();
            domain::PolicyChange::Permissions { mappings: decode_permissions(mappings) }
        }
    }
}

pub(super) fn encode_person_address(
    value: &domain::PersonAddress,
    limits: &wire::Limits,
) -> Result<wire::PersonAddress, wire::Problem> {
    Ok(match value {
        domain::PersonAddress::Person(value) => wire::PersonAddress::Person(wire::PersonAddressPerson::new(
            limits,
            wire::PersonAddressPersonParts { value: *value },
        )?),
        domain::PersonAddress::Role(value) => wire::PersonAddress::Role(wire::PersonAddressRole::new(
            limits,
            wire::PersonAddressRoleParts { value: *value },
        )?),
    })
}
pub(super) fn decode_person_address(value: wire::PersonAddress) -> domain::PersonAddress {
    match value {
        wire::PersonAddress::Person(value) => {
            let wire::PersonAddressPersonParts { value } = value.into_parts();
            domain::PersonAddress::Person(value)
        }
        wire::PersonAddress::Role(value) => {
            let wire::PersonAddressRoleParts { value } = value.into_parts();
            domain::PersonAddress::Role(value)
        }
    }
}

pub(super) fn encode_address_change(
    value: &domain::AddressChange,
    limits: &wire::Limits,
) -> Result<wire::AddressChange, wire::Problem> {
    let domain::AddressChange { revision, address } = value;
    wire::AddressChange::new(
        limits,
        wire::AddressChangeParts { revision: *revision, address: encode_person_address(address, limits)? },
    )
}
pub(super) fn decode_address_change(value: wire::AddressChange) -> domain::AddressChange {
    let wire::AddressChangeParts { revision, address } = value.into_parts();
    domain::AddressChange { revision, address: decode_person_address(address) }
}

pub(super) fn encode_parameter(
    value: &domain::Parameter,
    limits: &wire::Limits,
) -> Result<wire::Parameter, wire::Problem> {
    Ok(match value {
        domain::Parameter::Number { name, value } => wire::Parameter::Number(wire::ParameterNumber::new(
            limits,
            wire::ParameterNumberParts { name: *name, value: *value },
        )?),
        domain::Parameter::Bytes { name, value } => wire::Parameter::Bytes(wire::ParameterBytes::new(
            limits,
            wire::ParameterBytesParts { name: *name, value: value.clone() },
        )?),
        domain::Parameter::Resource { name, connector, resource } => {
            wire::Parameter::Resource(wire::ParameterResource::new(
                limits,
                wire::ParameterResourceParts { name: *name, connector: *connector, resource: *resource },
            )?)
        }
    })
}
pub(super) fn decode_parameter(value: wire::Parameter) -> domain::Parameter {
    match value {
        wire::Parameter::Number(value) => {
            let wire::ParameterNumberParts { name, value } = value.into_parts();
            domain::Parameter::Number { name, value }
        }
        wire::Parameter::Bytes(value) => {
            let wire::ParameterBytesParts { name, value } = value.into_parts();
            domain::Parameter::Bytes { name, value }
        }
        wire::Parameter::Resource(value) => {
            let wire::ParameterResourceParts { name, connector, resource } = value.into_parts();
            domain::Parameter::Resource { name, connector, resource }
        }
    }
}

pub(super) fn encode_spec(value: &domain::Spec, limits: &wire::Limits) -> Result<wire::Spec, wire::Problem> {
    let domain::Spec { words, parameters, inputs } = value;
    wire::Spec::new(
        limits,
        wire::SpecParts {
            words: words.clone(),
            parameters: encode_spec_parameters(parameters, limits)?,
            inputs: encode_numbers(inputs, limits)?,
        },
    )
}
pub(super) fn decode_spec(value: wire::Spec) -> domain::Spec {
    let wire::SpecParts { words, parameters, inputs } = value.into_parts();
    domain::Spec { words, parameters: decode_spec_parameters(parameters), inputs: decode_numbers(inputs) }
}

pub(super) fn encode_wake_rule(
    value: &domain::WakeRule,
    limits: &wire::Limits,
) -> Result<wire::WakeRule, wire::Problem> {
    Ok(match value {
        domain::WakeRule::Never => wire::WakeRule::Never,
        domain::WakeRule::Immediate => wire::WakeRule::Immediate,
        domain::WakeRule::Batch { count, age } => wire::WakeRule::Batch(wire::WakeRuleBatch::new(
            limits,
            wire::WakeRuleBatchParts { count: *count, age: age.as_nanos() },
        )?),
    })
}
pub(super) fn decode_wake_rule(value: wire::WakeRule) -> domain::WakeRule {
    match value {
        wire::WakeRule::Never => domain::WakeRule::Never,
        wire::WakeRule::Immediate => domain::WakeRule::Immediate,
        wire::WakeRule::Batch(value) => {
            let wire::WakeRuleBatchParts { count, age } = value.into_parts();
            domain::WakeRule::Batch { count, age: Duration::from_nanos(age) }
        }
    }
}

pub(super) fn encode_results_wake(value: domain::ResultsWake, _limits: &wire::Limits) -> wire::ResultsWake {
    match value {
        domain::ResultsWake::Never => wire::ResultsWake::Never,
        domain::ResultsWake::Each => wire::ResultsWake::Each,
        domain::ResultsWake::LastOrFailure => wire::ResultsWake::LastOrFailure,
    }
}
pub(super) fn decode_results_wake(value: wire::ResultsWake) -> domain::ResultsWake {
    match value {
        wire::ResultsWake::Never => domain::ResultsWake::Never,
        wire::ResultsWake::Each => domain::ResultsWake::Each,
        wire::ResultsWake::LastOrFailure => domain::ResultsWake::LastOrFailure,
    }
}

pub(super) fn encode_wake_policy(
    value: &domain::WakePolicy,
    limits: &wire::Limits,
) -> Result<wire::WakePolicy, wire::Problem> {
    let domain::WakePolicy { words, notices, news, results, questions, answers, timers } = value;
    wire::WakePolicy::new(
        limits,
        wire::WakePolicyParts {
            words: encode_wake_rule(words, limits)?,
            notices: encode_wake_rule(notices, limits)?,
            news: encode_wake_rule(news, limits)?,
            results: encode_results_wake(*results, limits),
            questions: *questions,
            answers: *answers,
            timers: *timers,
        },
    )
}
pub(super) fn decode_wake_policy(value: wire::WakePolicy) -> domain::WakePolicy {
    let wire::WakePolicyParts { words, notices, news, results, questions, answers, timers } = value.into_parts();
    domain::WakePolicy {
        words: decode_wake_rule(words),
        notices: decode_wake_rule(notices),
        news: decode_wake_rule(news),
        results: decode_results_wake(results),
        questions,
        answers,
        timers,
    }
}

pub(super) fn encode_amendment(
    value: &domain::Amendment,
    limits: &wire::Limits,
) -> Result<wire::Amendment, wire::Problem> {
    let domain::Amendment { addressed, instructions, spec, wake, dependencies, authority, reason } = value;
    wire::Amendment::new(
        limits,
        wire::AmendmentParts {
            addressed: match addressed {
                Some(value) => Some(encode_address_change(value, limits)?),
                None => None,
            },
            instructions: instructions.clone(),
            spec: match spec {
                Some(value) => Some(encode_spec(value, limits)?),
                None => None,
            },
            wake: match wake {
                Some(value) => Some(encode_wake_policy(value, limits)?),
                None => None,
            },
            dependencies: match dependencies {
                Some(value) => Some(encode_numbers(value, limits)?),
                None => None,
            },
            authority: match authority {
                Some(value) => Some(encode_authority(value, limits)?),
                None => None,
            },
            reason: reason.clone(),
        },
    )
}
#[expect(clippy::manual_map, reason = "step code translates optional fields through exhaustive matches")]
pub(super) fn decode_amendment(value: wire::Amendment) -> domain::Amendment {
    let wire::AmendmentParts { addressed, instructions, spec, wake, dependencies, authority, reason } =
        value.into_parts();
    domain::Amendment {
        addressed: match addressed {
            Some(value) => Some(decode_address_change(value)),
            None => None,
        },
        instructions,
        spec: match spec {
            Some(value) => Some(decode_spec(value)),
            None => None,
        },
        wake: match wake {
            Some(value) => Some(decode_wake_policy(value)),
            None => None,
        },
        dependencies: match dependencies {
            Some(value) => Some(decode_numbers(value)),
            None => None,
        },
        authority: match authority {
            Some(value) => Some(decode_authority(value)),
            None => None,
        },
        reason,
    }
}
