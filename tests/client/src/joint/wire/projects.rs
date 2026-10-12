//! Whole project owner facts (contract.md, 5.1–5.3).
//! Current and configured ledgers remain independent; this boundary neither
//! infers a project name nor admits a public Project watch.
use super::{chips, history, offers, owner};
use jig_domain_authority as authority;
use jig_domain_client::{Limits, contract as client};
use jig_domain_core::documents;
use jig_domain_people as people;
use jig_domain_tasks as tasks;

/// # Errors
/// Refuses inconsistent owner facts or any whole reserved allocation above bounds.
pub fn document(source: &documents::Project, limits: &Limits) -> Result<client::OwnerProject, owner::Invalid> {
    Ok(client::OwnerProject {
        acts: offers::offered(&source.acts, limits)?,
        number: source.number,
        resources: owner::list(&source.resources, limits, |value| {
            Ok(client::OwnerProjectResource {
                part: owner::part(&value.part, limits)?,
                role: match value.role {
                    documents::ResourceRole::Owned => client::OwnerProjectResourceRole::Owned,
                    documents::ResourceRole::Participant => client::OwnerProjectResourceRole::Participant,
                    documents::ResourceRole::Unavailable => client::OwnerProjectResourceRole::Unavailable,
                    documents::ResourceRole::Context => client::OwnerProjectResourceRole::Context,
                },
            })
        })?,
        policy: policy(&source.policy, limits)?,
        members: owner::list(&source.members, limits, |value| {
            Ok(client::OwnerProjectMember {
                party: value.party,
                kind: match value.kind {
                    people::Kind::Person => client::PartyKind::Person,
                    people::Kind::Service => client::PartyKind::Service,
                },
                role: owner::held_role(value.role),
                login: owner::text(&value.login, limits)?,
                name: owner::text(&value.name, limits)?,
            })
        })?,
        current_period: source.current_period.as_ref().map(owner::ledger),
        pools: owner::list(&source.pools, limits, |value| Ok(owner::ledger(value)))?,
        configured_period: source.configured_period,
        configured_pools: owner::list(&source.configured_pools, limits, |value| Ok(owner::ledger(value)))?,
        configured_pools_cut: source.configured_pools_cut,
        spend: owner::list(&source.spend, limits, |value| {
            Ok(client::OwnerProjectSpend {
                chip: chips::chip(&value.chip, limits)?,
                funder: owner::funder(value.funder),
                numbers: owner::numbers(value.numbers),
            })
        })?,
        recurring: owner::list(&source.recurring, limits, |value| recurring(value, limits))?,
        cut: source.cut,
    })
}

fn policy(source: &documents::Policy, limits: &Limits) -> Result<client::OwnerProjectPolicy, owner::Invalid> {
    Ok(client::OwnerProjectPolicy {
        ceiling: owner::authority(&source.ceiling, limits)?,
        period_spend: source.period_spend,
        escalation_role: source.escalation_role,
        projections: owner::list(&source.projections, limits, |value| grant(value, limits))?,
        roles: owner::list(&source.roles, limits, |value| {
            Ok(client::OwnerProjectPolicyRole {
                number: value.number,
                authority: owner::authority(&value.authority, limits)?,
                period_spend: value.period_spend,
                requests: value.requests.0,
                decides: value.decides.0,
            })
        })?,
        requirements: owner::list(&source.requirements, limits, |value| {
            Ok(client::PolicyRequirement {
                connector: value.grant.connector,
                kind: value.grant.kind,
                pattern: owner::pattern(&value.grant.segments, &value.grant.last, limits)?,
                judge: client::PolicyJudge {
                    connector: value.judge.connector,
                    requirement: value.judge.requirement,
                    parameters: value.judge.parameters,
                },
                guard: match value.guard {
                    authority::Guard::Guarded => client::PolicyGuard::Guarded,
                    authority::Guard::Observed { freshness } => client::PolicyGuard::Observed { freshness },
                },
                must_be_guarded: value.must_be_guarded,
            })
        })?,
        permissions: source
            .permissions
            .as_ref()
            .map(|values| {
                owner::list(values, limits, |value| {
                    Ok(client::PolicyPermission {
                        connector: value.connector,
                        permission: value.permission,
                        role: value.role,
                    })
                })
            })
            .transpose()?,
        permissions_cut: source.permissions_cut,
        cut: source.cut,
    })
}

fn grant(source: &documents::GrantLine, limits: &Limits) -> Result<client::OwnerGrant, owner::Invalid> {
    Ok(client::OwnerGrant {
        connector: source.connector,
        kind: source.kind,
        pattern: owner::pattern(&source.segments, &source.last, limits)?,
    })
}

fn recurring(source: &documents::Recurring, limits: &Limits) -> Result<client::Recurring, owner::Invalid> {
    Ok(client::Recurring {
        chip: chips::chip(&source.chip, limits)?,
        key: source.key,
        batch: owner::list(&source.batch, limits, |value| history::proposed(value, limits))?,
        budget: source.budget,
        overlap: match source.overlap {
            tasks::RecurringOverlap::Skip => client::RecurringOverlap::Skip,
            tasks::RecurringOverlap::Wait => client::RecurringOverlap::Wait,
        },
        current_period: source.current_period,
        began: source.began,
        next: source.next,
        last_period: source.last_period,
        pending_period: source.pending_period,
        last_batch: owner::list(&source.last_batch, limits, |value| Ok(*value))?,
        cut: source.cut,
    })
}

#[cfg(test)]
pub(crate) mod tests;
