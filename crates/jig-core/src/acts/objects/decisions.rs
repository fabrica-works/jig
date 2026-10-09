//! Proposal and escalation choices use their current waiting holder.
use crate::acts::Act;
use crate::{Core, Limits, documents};
use jig_core_people as people;
use jig_core_tasks as tasks;

#[expect(clippy::too_many_lines, reason = "both origin families share the exhaustive closed decision vocabulary")]
pub(crate) fn proposal(
    core: &Core,
    limits: &Limits,
    party: u64,
    act: Act,
    proposer: documents::Proposer,
    number: u64,
) -> Result<(), people::Refusal> {
    match proposer {
        documents::Proposer::Party(person) => {
            let row = core.tasks.person_proposal(person, number).ok_or(people::Refusal::Unknown)?;
            let role = core.people.role(party, row.project);
            core.goal_standing(&row, row.project, role)?;
            match &row.state {
                tasks::PersonProposalState::Pending { .. } => {}
                tasks::PersonProposalState::Accepted { .. } | tasks::PersonProposalState::Rejected { .. } => {
                    return Err(people::Refusal::Unknown);
                }
            }
            match act {
                Act::Accept => {
                    if core.projections.len() >= limits.tasks.tasks {
                        return Err(people::Refusal::Limit { which: people::LimitKind::LiveTasks });
                    }
                    core.goal_accept_allowed(
                        &row,
                        row.project,
                        party,
                        role.expect("standing found role"),
                        limits.tasks.tree_tasks,
                    )?;
                    Ok(())
                }
                Act::Reject => Ok(()),
                Act::PassUp => Err(people::Refusal::NoFurther),
                Act::Write
                | Act::Answer
                | Act::Take
                | Act::HandBack
                | Act::Release
                | Act::LeaveHeld
                | Act::Amend
                | Act::AmendRelease
                | Act::Stop
                | Act::Cancel
                | Act::Close
                | Act::TakeOver
                | Act::SetGoal
                | Act::Prioritise
                | Act::MarkSeen
                | Act::EditNote
                | Act::DeleteNote
                | Act::Adopt
                | Act::ChangePolicy
                | Act::SetRoles
                | Act::MakeService
                | Act::SetPool => Err(people::Refusal::NotOffered),
            }
        }
        documents::Proposer::Task(task) => {
            let row = core.tasks.proposal(task, number).ok_or(people::Refusal::Unknown)?;
            let role = core.people.role(party, row.project);
            let current = match row.state {
                tasks::ProposalState::Pending { holder, .. } => holder,
                tasks::ProposalState::Accepted { .. }
                | tasks::ProposalState::Rejected { .. }
                | tasks::ProposalState::Withdrawn => return Err(people::Refusal::Unknown),
            };
            let (_, kind) = Core::proposal_kind(&row.action);
            let waiting = match current {
                tasks::ProposalHolder::Person(holder) => holder == party,
                tasks::ProposalHolder::Policy { project, .. } => {
                    project == row.project && core.proposal_policy_standing(party, row.project, kind)
                }
                tasks::ProposalHolder::Task(_) => false,
            };
            if !waiting || role.is_none() {
                return Err(people::Refusal::NotWaiting);
            }
            match act {
                Act::Accept => {
                    let action = Core::proposal_action_for_check(&row.action)
                        .ok_or(people::Refusal::Authority { proposable: None, lacking: people::Lack::Shape })?;
                    core.proposal_person_admit(
                        party,
                        row.project,
                        role.expect("standing found role"),
                        action,
                        limits.tasks.tree_tasks,
                    )
                }
                Act::Reject => Ok(()),
                Act::PassUp => match current {
                    tasks::ProposalHolder::Person(_) => Ok(()),
                    tasks::ProposalHolder::Policy { .. } => Err(people::Refusal::NoFurther),
                    tasks::ProposalHolder::Task(_) => unreachable!("party standing checked"),
                },
                Act::Write
                | Act::Answer
                | Act::Take
                | Act::HandBack
                | Act::Release
                | Act::LeaveHeld
                | Act::Amend
                | Act::AmendRelease
                | Act::Stop
                | Act::Cancel
                | Act::Close
                | Act::TakeOver
                | Act::SetGoal
                | Act::Prioritise
                | Act::MarkSeen
                | Act::EditNote
                | Act::DeleteNote
                | Act::Adopt
                | Act::ChangePolicy
                | Act::SetRoles
                | Act::MakeService
                | Act::SetPool => Err(people::Refusal::NotOffered),
            }
        }
    }
}

pub(crate) fn escalation(
    core: &Core,
    limits: &Limits,
    party: u64,
    act: Act,
    number: u64,
) -> Result<(), people::Refusal> {
    let context = core.tasks.escalation(number).ok_or(people::Refusal::Unknown)?;
    let role = core.people.role(party, context.project);
    if !core.escalation_visible(&context, party, role) {
        return Err(people::Refusal::Unknown);
    }
    match context.escalation {
        tasks::Escalation::Waiting { .. } => {}
        tasks::Escalation::Unheld { .. } | tasks::Escalation::Routing { .. } | tasks::Escalation::Rejected { .. } => {
            return Err(people::Refusal::Unknown);
        }
    }
    if !Core::escalation_standing(&context, party, role) {
        return Err(people::Refusal::NotWaiting);
    }
    match act {
        Act::Release => {
            if !core.escalation_covers(limits, &context, party, role) {
                return Err(people::Refusal::Role { held: role });
            }
            core.escalation_release_allowed(&context, party, role, limits.tasks.tree_tasks)
        }
        Act::LeaveHeld => Ok(()),
        Act::PassUp => {
            if core.escalation_fallback(context.project).is_some() {
                Ok(())
            } else {
                Err(people::Refusal::NoFurther)
            }
        }
        Act::Write
        | Act::Answer
        | Act::Take
        | Act::HandBack
        | Act::Accept
        | Act::Reject
        | Act::Amend
        | Act::AmendRelease
        | Act::Stop
        | Act::Cancel
        | Act::Close
        | Act::TakeOver
        | Act::SetGoal
        | Act::Prioritise
        | Act::MarkSeen
        | Act::EditNote
        | Act::DeleteNote
        | Act::Adopt
        | Act::ChangePolicy
        | Act::SetRoles
        | Act::MakeService
        | Act::SetPool => Err(people::Refusal::NotOffered),
    }
}
