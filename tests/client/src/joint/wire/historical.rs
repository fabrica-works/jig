//! Whole immutable historical sources (contract.md, 6; objects.md, 11).
//! Original query identities precede admission; every optional source is copied
//! independently. The source reader authenticates Question answer order while
//! joining its stored rows; this layer cannot reconstruct its absent position.
use super::{activity, documents, historical_goal, history, owner, person_tasks, results, task};
use jig_client_domain::contract as client;
use jig_client_scripted_engine as admission;
use jig_client_view::Limits;
use jig_domain_core::reads;
use jig_domain_people as people;
use jig_domain_tasks as tasks;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Invalid {
    Query,
    Identity,
    Unsupported,
    Recurrence,
    Owner(owner::Invalid),
    Document(documents::Invalid),
    Activity(activity::Invalid),
    Payload,
}

fn key(query: &client::Read) -> Result<client::ObjectKey, Invalid> {
    Ok(match query {
        client::Read::Task { task } => client::ObjectKey::Task(*task),
        client::Read::Proposal { proposal } => client::ObjectKey::Proposal(*proposal),
        client::Read::Escalation { task } => client::ObjectKey::Escalation(*task),
        client::Read::Question { task, message } => client::ObjectKey::Question { task: *task, message: *message },
        client::Read::PersonTask { task } => client::ObjectKey::PersonTask(*task),
        client::Read::Result { task } => client::ObjectKey::Result(*task),
        client::Read::Tasks { .. }
        | client::Read::Me
        | client::Read::Activity { .. }
        | client::Read::Notes { .. }
        | client::Read::Inbox { .. }
        | client::Read::History { .. }
        | client::Read::Ended { .. }
        | client::Read::Spend { .. }
        | client::Read::Project { .. }
        | client::Read::Deployment => return Err(Invalid::Query),
    })
}

/// # Errors
/// Refuses wrong original identities, unsupported archive carriers, unadmitted
/// recurrence and any whole source or receiving allocation above its bound.
pub fn read(query: &client::Read, source: &reads::Historical, limits: &Limits) -> Result<client::ReadValue, Invalid> {
    let key = key(query)?;
    let supplied = match &source.value {
        reads::Past::Task { document, .. } => client::ObjectKey::Task(document.chip.number),
        reads::Past::Proposal(value) => client::ObjectKey::Proposal(client::ProposalKey {
            proposer: client::Proposer::Task(value.proposer),
            number: value.number,
        }),
        reads::Past::PersonProposal { proposal, .. } => client::ObjectKey::Proposal(client::ProposalKey {
            proposer: client::Proposer::Party(proposal.proposer),
            number: proposal.number,
        }),
        reads::Past::Escalation { task, .. } => client::ObjectKey::Escalation(task.chip.number),
        reads::Past::Question { question, .. } => {
            client::ObjectKey::Question { task: question.task.number, message: question.message }
        }
        reads::Past::PersonTask { document, .. } => client::ObjectKey::PersonTask(document.task.number),
        reads::Past::Reply(_) | reads::Past::Turn(_) => return Err(Invalid::Unsupported),
    };
    if key != supplied {
        return Err(Invalid::Identity);
    }
    let value = match &source.value {
        reads::Past::Task { document, result } => client::OwnerHistoricalValue::Task {
            document: Box::new(task::document(document, limits).map_err(Invalid::Document)?),
            result: Box::new(results::document(result, limits).map_err(Invalid::Owner)?),
        },
        reads::Past::Proposal(value) => {
            client::OwnerHistoricalValue::Proposal(history::proposal(value, limits).map_err(Invalid::Owner)?)
        }
        reads::Past::PersonProposal { proposal, decision } => client::OwnerHistoricalValue::PersonProposal {
            proposal: Box::new(person_proposal(proposal, limits)?),
            decision: decision.map(proposal_decision),
        },
        reads::Past::Escalation { task: document, decision, hold } => client::OwnerHistoricalValue::Escalation {
            task: Box::new(task::document(document, limits).map_err(Invalid::Document)?),
            decision: escalation_decision(decision, limits)?,
            hold: hold.as_ref().map(|value| history::milestone(value, limits).map_err(Invalid::Owner)).transpose()?,
        },
        reads::Past::Question { question, answer, ending } => {
            let client::ReadValue::Question(question) =
                documents::question_read(question, limits).map_err(Invalid::Document)?
            else {
                unreachable!("whole Question translator returns its declared carrier")
            };
            let answer = answer
                .as_ref()
                .map(|value| {
                    let value = activity::line(question.task.task, value, limits).map_err(Invalid::Activity)?;
                    let client::Activity::Owner { source, decoded: _ } = value.event else {
                        unreachable!("owner activity translator returns its declared source")
                    };
                    Ok(*source)
                })
                .transpose()?;
            client::OwnerHistoricalValue::Question {
                question: *question,
                answer,
                ending: ending
                    .as_ref()
                    .map(|value| results::document(value, limits).map(Box::new).map_err(Invalid::Owner))
                    .transpose()?,
            }
        }
        reads::Past::PersonTask { document, result } => client::OwnerHistoricalValue::PersonTask {
            document: Box::new(person_tasks::document(document, limits).map_err(Invalid::Owner)?),
            result: Box::new(results::document(result, limits).map_err(Invalid::Owner)?),
        },
        // Archive references have their own exact protocol routes; these are not
        // near neighbours of the six historical object documents.
        reads::Past::Reply(_) | reads::Past::Turn(_) => return Err(Invalid::Unsupported),
    };
    let value = client::OwnerHistorical { key, value, at: source.at, by: source.by.map(results::requester) };
    if !value.matches(key) {
        return Err(Invalid::Identity);
    }
    let value = client::ReadValue::Historical(Box::new(value));
    admission::admit_read_value(
        &value,
        &admission::Bounds {
            text: limits.text,
            rows: limits.rows,
            segments: limits.segments,
            document_bytes: limits.document_bytes,
            ..admission::Bounds::default()
        },
    )
    .map_err(|_| Invalid::Payload)?;
    Ok(value)
}

fn person_proposal(value: &tasks::PersonProposal, limits: &Limits) -> Result<client::OwnerPersonProposal, Invalid> {
    Ok(client::OwnerPersonProposal {
        number: value.number,
        proposer: value.proposer,
        project: value.project,
        goal: historical_goal::goal(value.batch().first().expect("one v8 goal"), limits)?,
        state: match &value.state {
            tasks::PersonProposalState::Withdrawn { .. } => unreachable!("no v8 withdrawal"),
            tasks::PersonProposalState::Pending { since } => {
                client::OwnerPersonProposalState::Pending { since: *since }
            }
            tasks::PersonProposalState::Accepted { by } => {
                client::OwnerPersonProposalState::Accepted { by: results::requester(*by) }
            }
            tasks::PersonProposalState::Rejected { by, reason, message, at } => {
                client::OwnerPersonProposalState::Rejected {
                    by: results::requester(*by),
                    reason: historical_goal::text(reason, limits).map_err(Invalid::Owner)?,
                    message: *message,
                    at: *at,
                }
            }
        },
    })
}

fn proposal_decision(value: jig_domain_core::ProposalDecisionRecord) -> client::OwnerProposalDecision {
    client::OwnerProposalDecision {
        project: value.project,
        proposer: match value.proposer {
            people::Proposer::Task(value) => client::Proposer::Task(value),
            people::Proposer::Party(value) => client::Proposer::Party(value),
        },
        proposal: value.proposal,
        kind: match value.kind {
            tasks::ProposalKind::Batch => client::OwnerMessageProposalKind::Batch,
            tasks::ProposalKind::Effect => client::OwnerMessageProposalKind::Effect,
            tasks::ProposalKind::Amend => client::OwnerMessageProposalKind::Amend,
            tasks::ProposalKind::Widen => client::OwnerMessageProposalKind::Widen,
            tasks::ProposalKind::Release => client::OwnerMessageProposalKind::Release,
        },
        by: value.by,
        at: value.at,
        choice: match value.choice {
            people::ProposalChoice::Accepted => client::OwnerProposalChoice::Accepted,
            people::ProposalChoice::Rejected => client::OwnerProposalChoice::Rejected,
            people::ProposalChoice::Passed => client::OwnerProposalChoice::Passed,
            people::ProposalChoice::Withdrawn => client::OwnerProposalChoice::Withdrawn,
            people::ProposalChoice::Stale => client::OwnerProposalChoice::Stale,
        },
        created: value.created,
    }
}

fn escalation_decision(
    value: &jig_domain_core::EscalationDecisionRecord,
    limits: &Limits,
) -> Result<client::OwnerEscalationDecisionRecord, Invalid> {
    Ok(client::OwnerEscalationDecisionRecord {
        project: value.project,
        requester: value.requester,
        task: value.task,
        revision: value.revision,
        by: value.by,
        at: value.at,
        decision: match &value.decision {
            people::EscalationDecision::Release => client::OwnerEscalationDecision::Release,
            people::EscalationDecision::Pass => client::OwnerEscalationDecision::Pass,
            people::EscalationDecision::Reject { reason } => client::OwnerEscalationDecision::Reject {
                reason: historical_goal::text(reason, limits).map_err(Invalid::Owner)?,
            },
        },
    })
}

#[cfg(test)]
pub(crate) mod tests;
