#![expect(clippy::manual_map, reason = "step code matches optional owner facts without closures")]
//! Whole single-read owner documents (protocol/client.md, 3 and 6).
use super::super::{chips, owner, task, w};
use alloc::boxed::Box;
use jig_domain_core::documents as d;
use jig_domain_tasks as t;
use skein_lib::{List, Writer};
pub fn project(value: &d::Project, limits: &w::Limits) -> Result<w::OwnerProject, w::Problem> {
    let mut resources = List::with_capacity(value.resources.len());
    for row in &value.resources {
        resources
            .push(w::OwnerProjectResource::new(
                limits,
                w::OwnerProjectResourceParts {
                    part: owner::part(&row.part, limits)?,
                    role: match row.role {
                        d::ResourceRole::Owned => w::OwnerProjectResourceRole::Owned,
                        d::ResourceRole::Participant => w::OwnerProjectResourceRole::Participant,
                        d::ResourceRole::Unavailable => w::OwnerProjectResourceRole::Unavailable,
                        d::ResourceRole::Context => w::OwnerProjectResourceRole::Context,
                    },
                },
            )?)
            .expect("supplied rows fit");
    }
    let resources =
        owner_project_resource_pieces(&resources, limits.owner_project_resources, w::Path::OwnerProjectResources)?;
    let mut members = List::with_capacity(value.members.len());
    for row in &value.members {
        members
            .push(w::OwnerProjectMember::new(
                limits,
                w::OwnerProjectMemberParts {
                    party: row.party,
                    kind: match row.kind {
                        jig_domain_people::Kind::Person => w::PartyKind::Person,
                        jig_domain_people::Kind::Service => w::PartyKind::Service,
                    },
                    role: owner::held_role(row.role, limits)?,
                    login: owner::text(&row.login, limits)?,
                    name: owner::text(&row.name, limits)?,
                },
            )?)
            .expect("supplied rows fit");
    }
    let mut pools = List::with_capacity(value.pools.len());
    for row in &value.pools {
        pools.push(owner::ledger(row, limits)?).expect("supplied rows fit");
    }
    let mut configured_pools = List::with_capacity(value.configured_pools.len());
    for row in &value.configured_pools {
        configured_pools.push(owner::ledger(row, limits)?).expect("supplied rows fit");
    }
    let mut spend = List::with_capacity(value.spend.len());
    for row in &value.spend {
        spend
            .push(w::OwnerProjectSpend::new(
                limits,
                w::OwnerProjectSpendParts {
                    chip: chips::chip(&row.chip, limits)?,
                    funder: owner::funder(row.funder, limits)?,
                    numbers: owner::numbers(row.numbers, limits)?,
                },
            )?)
            .expect("supplied rows fit");
    }
    let spend = owner_project_spend_pieces(&spend, limits.owner_project_spend, w::Path::OwnerProjectSpend)?;
    let mut recurring = List::with_capacity(value.recurring.len());
    for row in &value.recurring {
        recurring.push(recurrence(row, limits)?).expect("supplied rows fit");
    }
    let recurring = recurring_pieces(&recurring, limits.owner_project_recurring, w::Path::OwnerProjectRecurring)?;

    w::OwnerProject::new(
        limits,
        w::OwnerProjectParts {
            acts: task::offered(&value.acts, limits, limits.owner_project_acts, w::Path::OwnerProjectActs)?,
            number: value.number,
            resources,
            policy: policy(&value.policy, limits)?,
            members,
            current_period: match &value.current_period {
                Some(item) => Some(owner::ledger(item, limits)?),
                None => None,
            },
            pools,
            configured_period: value.configured_period,
            configured_pools,
            configured_pools_cut: value.configured_pools_cut,
            spend,
            recurring,
            cut: value.cut,
        },
    )
}
pub fn policy(value: &d::Policy, limits: &w::Limits) -> Result<w::OwnerProjectPolicy, w::Problem> {
    let mut projections = List::with_capacity(value.projections.len());
    for row in &value.projections {
        projections.push(owner::grant(row, limits)?).expect("supplied rows fit");
    }
    let projections = owner_grant_pieces(
        &projections,
        limits.owner_project_policy_projections,
        w::Path::OwnerProjectPolicyProjections,
    )?;
    let mut roles = List::with_capacity(value.roles.len());
    for row in &value.roles {
        roles
            .push(w::OwnerProjectPolicyRole::new(
                limits,
                w::OwnerProjectPolicyRoleParts {
                    number: row.number,
                    authority: owner::authority(&row.authority, limits)?,
                    period_spend: row.period_spend,
                    requests: row.requests.0,
                    decides: row.decides.0,
                },
            )?)
            .expect("supplied rows fit");
    }
    let roles =
        owner_project_policy_role_pieces(&roles, limits.owner_project_policy_roles, w::Path::OwnerProjectPolicyRoles)?;
    let mut requirements = List::with_capacity(value.requirements.len());
    for row in &value.requirements {
        requirements
            .push(w::PolicyRequirement::new(
                limits,
                w::PolicyRequirementParts {
                    connector: row.grant.connector,
                    kind: row.grant.kind,
                    pattern: owner::pattern(&row.grant.segments, &row.grant.last, limits)?,
                    judge: w::PolicyJudge::new(
                        limits,
                        w::PolicyJudgeParts {
                            connector: row.judge.connector,
                            requirement: row.judge.requirement,
                            parameters: row.judge.parameters,
                        },
                    )?,
                    guard: match row.guard {
                        jig_domain_authority::Guard::Guarded => w::PolicyGuard::Guarded,
                        jig_domain_authority::Guard::Observed { freshness } => w::PolicyGuard::Observed(
                            w::PolicyGuardObserved::new(limits, w::PolicyGuardObservedParts { freshness })?,
                        ),
                    },
                    must_be_guarded: row.must_be_guarded,
                },
            )?)
            .expect("supplied rows fit");
    }
    let requirements = policy_requirement_pieces(
        &requirements,
        limits.owner_project_policy_requirements,
        w::Path::OwnerProjectPolicyRequirements,
    )?;
    let permissions = match &value.permissions {
        Some(rows) => {
            let mut copied = List::with_capacity(rows.len());
            for row in rows {
                copied
                    .push(w::PolicyPermission::new(
                        limits,
                        w::PolicyPermissionParts {
                            connector: row.connector,
                            permission: row.permission,
                            role: row.role,
                        },
                    )?)
                    .expect("supplied permissions fit");
            }
            Some(copied)
        }
        None => None,
    };
    w::OwnerProjectPolicy::new(
        limits,
        w::OwnerProjectPolicyParts {
            templates: super::super::me::overrides(&value.templates, limits)?,
            ceiling: owner::authority(&value.ceiling, limits)?,
            period_spend: value.period_spend,
            escalation_role: value.escalation_role,
            projections,
            roles,
            requirements,
            permissions,
            permissions_cut: value.permissions_cut,
            cut: value.cut,
        },
    )
}
pub fn recurrence(value: &d::Recurring, limits: &w::Limits) -> Result<w::Recurring, w::Problem> {
    let mut batch = List::with_capacity(value.batch.len());
    for row in &value.batch {
        batch.push(super::proposals::proposed(row, limits)?).expect("supplied rows fit");
    }
    let batch = super::proposals::proposed_task_pieces(&batch, limits.recurring_batch, w::Path::RecurringBatch)?;
    let mut last_batch = List::with_capacity(value.last_batch.len());
    for row in &value.last_batch {
        last_batch.push(*row).expect("supplied rows fit");
    }

    w::Recurring::new(
        limits,
        w::RecurringParts {
            chip: chips::chip(&value.chip, limits)?,
            key: value.key,
            batch,
            budget: value.budget,
            overlap: match value.overlap {
                t::RecurringOverlap::Skip => w::RecurringOverlap::Skip,
                t::RecurringOverlap::Wait => w::RecurringOverlap::Wait,
            },
            current_period: value.current_period,
            began: match value.began {
                Some(item) => Some(item.as_nanos()),
                None => None,
            },
            next: match value.next {
                Some(item) => Some(item.as_nanos()),
                None => None,
            },
            last_period: value.last_period,
            pending_period: value.pending_period,
            last_batch,
            cut: value.cut,
        },
    )
}
pub(super) fn owner_project_resource_pieces(
    rows: &List<w::OwnerProjectResource>,
    bound: u32,
    path: w::Path,
) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(usize::try_from(row.measure()).expect("u32 measure fits usize"));
        match row.encode(&mut out) {
            Ok(()) => (),
            Err(_) => return Err(w::Problem { path, reason: skein_codec::Reason::Bound }),
        }
        encoded.push(out.finish()).expect("supplied rows fit");
    }
    super::super::pieces(&encoded, bound, path)
}
pub(super) fn owner_project_spend_pieces(
    rows: &List<w::OwnerProjectSpend>,
    bound: u32,
    path: w::Path,
) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(usize::try_from(row.measure()).expect("u32 measure fits usize"));
        match row.encode(&mut out) {
            Ok(()) => (),
            Err(_) => return Err(w::Problem { path, reason: skein_codec::Reason::Bound }),
        }
        encoded.push(out.finish()).expect("supplied rows fit");
    }
    super::super::pieces(&encoded, bound, path)
}
pub(super) fn recurring_pieces(rows: &List<w::Recurring>, bound: u32, path: w::Path) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(usize::try_from(row.measure()).expect("u32 measure fits usize"));
        match row.encode(&mut out) {
            Ok(()) => (),
            Err(_) => return Err(w::Problem { path, reason: skein_codec::Reason::Bound }),
        }
        encoded.push(out.finish()).expect("supplied rows fit");
    }
    super::super::pieces(&encoded, bound, path)
}
pub(super) fn owner_grant_pieces(
    rows: &List<w::OwnerGrant>,
    bound: u32,
    path: w::Path,
) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(usize::try_from(row.measure()).expect("u32 measure fits usize"));
        match row.encode(&mut out) {
            Ok(()) => (),
            Err(_) => return Err(w::Problem { path, reason: skein_codec::Reason::Bound }),
        }
        encoded.push(out.finish()).expect("supplied rows fit");
    }
    super::super::pieces(&encoded, bound, path)
}
pub(super) fn owner_project_policy_role_pieces(
    rows: &List<w::OwnerProjectPolicyRole>,
    bound: u32,
    path: w::Path,
) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(usize::try_from(row.measure()).expect("u32 measure fits usize"));
        match row.encode(&mut out) {
            Ok(()) => (),
            Err(_) => return Err(w::Problem { path, reason: skein_codec::Reason::Bound }),
        }
        encoded.push(out.finish()).expect("supplied rows fit");
    }
    super::super::pieces(&encoded, bound, path)
}
pub(super) fn policy_requirement_pieces(
    rows: &List<w::PolicyRequirement>,
    bound: u32,
    path: w::Path,
) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(usize::try_from(row.measure()).expect("u32 measure fits usize"));
        match row.encode(&mut out) {
            Ok(()) => (),
            Err(_) => return Err(w::Problem { path, reason: skein_codec::Reason::Bound }),
        }
        encoded.push(out.finish()).expect("supplied rows fit");
    }
    super::super::pieces(&encoded, bound, path)
}
