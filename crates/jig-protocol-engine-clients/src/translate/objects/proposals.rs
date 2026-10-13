//! Whole single-read owner documents (protocol/client.md, 3 and 6).
use super::super::{chips, owner, task, w};
use alloc::boxed::Box;
use jig_domain_core::documents as d;
use jig_domain_tasks as t;
use skein_lib::{List, Writer};
pub fn amendment(value: &d::AmendmentState, limits: &w::Limits) -> Result<w::OwnerAmendmentState, w::Problem> {
    let mut dependencies = List::with_capacity(value.dependencies.len());
    for row in &value.dependencies {
        dependencies.push(*row).expect("supplied rows fit");
    }

    w::OwnerAmendmentState::new(
        limits,
        w::OwnerAmendmentStateParts {
            addressed: match value.addressed {
                Some(item) => Some(chips::address(item, limits)?),
                None => None,
            },
            taken_by: value.taken_by,
            instructions: match &value.instructions {
                Some(item) => Some(owner::text(item, limits)?),
                None => None,
            },
            spec: owner::specification(&value.spec, limits)?,
            authority: owner::authority(&value.authority, limits)?,
            wake: owner::wake(value.wake, limits)?,
            dependencies,
            cut: value.cut,
        },
    )
}
pub fn kind(value: t::ProposalKind) -> w::OwnerMessageProposalKind {
    match value {
        t::ProposalKind::Batch => w::OwnerMessageProposalKind::Batch,
        t::ProposalKind::Effect => w::OwnerMessageProposalKind::Effect,
        t::ProposalKind::Amend => w::OwnerMessageProposalKind::Amend,
        t::ProposalKind::Widen => w::OwnerMessageProposalKind::Widen,
        t::ProposalKind::Release => w::OwnerMessageProposalKind::Release,
    }
}
pub fn holder(value: t::ProposalHolder, limits: &w::Limits) -> Result<w::OwnerHistoryHolder, w::Problem> {
    Ok(match value {
        t::ProposalHolder::Task(value) => w::OwnerHistoryHolder::Task(w::OwnerHistoryHolderTask::new(
            limits,
            w::OwnerHistoryHolderTaskParts { value },
        )?),
        t::ProposalHolder::Person(value) => w::OwnerHistoryHolder::Person(w::OwnerHistoryHolderPerson::new(
            limits,
            w::OwnerHistoryHolderPersonParts { value },
        )?),
        t::ProposalHolder::Policy { project, kind: value } => w::OwnerHistoryHolder::Policy(
            w::OwnerHistoryHolderPolicy::new(limits, w::OwnerHistoryHolderPolicyParts { project, kind: kind(value) })?,
        ),
    })
}
pub fn proposed(value: &d::ProposedTask, limits: &w::Limits) -> Result<w::ProposedTask, w::Problem> {
    let mut dependencies = List::with_capacity(value.dependencies.len());
    for row in &value.dependencies {
        dependencies.push(*row).expect("supplied rows fit");
    }
    let mut resources = List::with_capacity(value.resources.len());
    for row in &value.resources {
        resources.push(owner::part(row, limits)?).expect("supplied rows fit");
    }
    let resources = owner_part_pieces(&resources, limits.proposed_task_resources, w::Path::ProposedTaskResources)?;

    w::ProposedTask::new(
        limits,
        w::ProposedTaskParts {
            number: value.number,
            spec: owner::specification(&value.spec, limits)?,
            executor: owner::executor(value.executor, limits)?,
            contract: owner::contract(&value.contract, limits)?,
            authority: owner::authority(&value.authority, limits)?,
            budget: value.budget,
            funder: owner::funder(value.funder, limits)?,
            dependencies,
            resources,
            wake: owner::wake(value.wake, limits)?,
            priority: value.priority,
            cut: value.cut,
        },
    )
}
pub fn action(value: &d::ProposalAction, limits: &w::Limits) -> Result<w::OwnerProposalAction, w::Problem> {
    Ok(match value {
        d::ProposalAction::Batch(value) => {
            let mut rows = List::with_capacity(value.len());
            for row in value {
                rows.push(proposed(row, limits)?).expect("supplied rows fit");
            }
            let rows = proposed_task_pieces(
                &rows,
                limits.owner_proposal_action_batch_value,
                w::Path::OwnerProposalActionBatchValue,
            )?;
            w::OwnerProposalAction::Batch(w::OwnerProposalActionBatch::new(
                limits,
                w::OwnerProposalActionBatchParts { value: rows },
            )?)
        }
        d::ProposalAction::Effect { part, authority, attempt, completion, position, purpose } => {
            w::OwnerProposalAction::Effect(w::OwnerProposalActionEffect::new(
                limits,
                w::OwnerProposalActionEffectParts {
                    part: owner::part(part, limits)?,
                    authority: owner::authority(authority, limits)?,
                    attempt: *attempt,
                    completion: *completion,
                    position: *position,
                    purpose: *purpose,
                },
            )?)
        }
        d::ProposalAction::ProcedureEffect { part, authority, step, purpose } => {
            w::OwnerProposalAction::ProcedureEffect(w::OwnerProposalActionProcedureEffect::new(
                limits,
                w::OwnerProposalActionProcedureEffectParts {
                    part: owner::part(part, limits)?,
                    authority: owner::authority(authority, limits)?,
                    step: *step,
                    purpose: *purpose,
                },
            )?)
        }
        d::ProposalAction::Amend { task: target, addressed, before, after, reason, release } => {
            w::OwnerProposalAction::Amend(Box::new(w::OwnerProposalActionAmend::new(
                limits,
                w::OwnerProposalActionAmendParts {
                    task: task::link(target, limits)?,
                    addressed: match *addressed {
                        Some(item) => Some(w::AddressChange::new(
                            limits,
                            w::AddressChangeParts {
                                revision: w::Revision::new(limits, w::RevisionParts { value: item.revision })?,
                                address: chips::address(item.address, limits)?,
                            },
                        )?),
                        None => None,
                    },
                    before: amendment(before, limits)?,
                    after: amendment(after, limits)?,
                    reason: owner::text(reason, limits)?,
                    release: *release,
                },
            )?))
        }
        d::ProposalAction::Widen { task: target, before, after } => {
            w::OwnerProposalAction::Widen(w::OwnerProposalActionWiden::new(
                limits,
                w::OwnerProposalActionWidenParts {
                    task: task::link(target, limits)?,
                    before: owner::authority(before, limits)?,
                    after: owner::authority(after, limits)?,
                },
            )?)
        }
        d::ProposalAction::Release { task: target, hold } => {
            w::OwnerProposalAction::Release(w::OwnerProposalActionRelease::new(
                limits,
                w::OwnerProposalActionReleaseParts {
                    task: task::link(target, limits)?,
                    hold: match hold {
                        Some(item) => Some(chips::held(item, limits)?),
                        None => None,
                    },
                },
            )?)
        }
    })
}
pub fn proposal(value: &d::Proposal, limits: &w::Limits) -> Result<w::OwnerProposal, w::Problem> {
    let mut funding = List::with_capacity(value.funding.len());
    for row in &value.funding {
        funding
            .push(w::OwnerFundingOffer::new(
                limits,
                w::OwnerFundingOfferParts {
                    source: owner::funder(row.source, limits)?,
                    needed: row.needed,
                    left: row.left,
                    after: row.after,
                },
            )?)
            .expect("supplied rows fit");
    }

    w::OwnerProposal::new(
        limits,
        w::OwnerProposalParts {
            key: task::proposal(&d::ProposalRef { proposer: value.proposer, number: value.number }, limits)?,
            project: value.project,
            reason: owner::text(&value.reason, limits)?,
            action: action(&value.action, limits)?,
            as_holder: value.as_holder,
            waiting: w::OwnerProposalWaiting::new(
                limits,
                w::OwnerProposalWaitingParts {
                    holder: holder(value.waiting.holder, limits)?,
                    since: value.waiting.since.as_nanos(),
                },
            )?,
            funding,
            acts: task::offered(&value.acts, limits, limits.owner_proposal_acts, w::Path::OwnerProposalActs)?,
        },
    )
}
pub(super) fn proposed_task_pieces(
    rows: &List<w::ProposedTask>,
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
pub(super) fn owner_part_pieces(rows: &List<w::OwnerPart>, bound: u32, path: w::Path) -> Result<Box<[u8]>, w::Problem> {
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
