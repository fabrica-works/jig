//! Historical decision-race answers routed back to authenticated parties
//! (`domain/people.md`, section 5.1). These complete keyed decisions, not reads.

use crate::{Core, EscalationDecisionRecord, Event, Limits, Request, Requests};
use jig_core_authority as authority;
use jig_core_people as people;
use jig_core_tasks as tasks;
use skein_lib::{Env, Queue, Token};

#[expect(clippy::too_many_arguments, reason = "the route retains its event fields")]
pub(super) fn route_historical_escalation(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    request: Token,
    person: u64,
    project: u32,
    task: u64,
    revision: u64,
    row: Option<EscalationDecisionRecord>,
    busy: bool,
) -> Requests {
    let outcome = if busy {
        people::Outcome::Again(people::Again::Busy)
    } else if let Some(row) = row {
        let named = row.task == task
            && row.revision == revision
            && row.task <= core.counters.deployment().tasks
            && row.by != 0
            && row.by <= core.counters.deployment().people;
        if named {
            let bounded = match &row.decision {
                people::EscalationDecision::Release | people::EscalationDecision::Pass => true,
                people::EscalationDecision::Reject { reason } => {
                    reason.len()
                        <= usize::try_from(env.limits.escalation_reason_bytes).expect("reason bound fits usize")
                }
            };
            let private = row.project == project
                && row.requester != 0
                && row.requester <= core.counters.deployment().people
                && (row.requester == person
                    || match core.escalation_fallback(project) {
                        Some(tasks::EscalationHolder::Role { role: selected, .. }) => {
                            match core.people.role(person, project) {
                                Some(role) => role.number() == selected,
                                None => false,
                            }
                        }
                        Some(tasks::EscalationHolder::Task(_) | tasks::EscalationHolder::Person(_)) | None => false,
                    });
            if !private {
                people::Outcome::Refused(people::Refusal::Unknown)
            } else if bounded {
                let choice = match row.decision {
                    people::EscalationDecision::Release => people::EscalationChoice::Released,
                    people::EscalationDecision::Reject { .. } => people::EscalationChoice::Rejected,
                    people::EscalationDecision::Pass => people::EscalationChoice::Passed,
                };
                people::Outcome::EscalationDecided { task, revision, by: row.by, at: row.at, choice }
            } else {
                people::Outcome::Refused(people::Refusal::Unknown)
            }
        } else {
            people::Outcome::Refused(people::Refusal::Unknown)
        }
    } else {
        people::Outcome::Refused(people::Refusal::Unknown)
    };
    work.push(Event::People(people::Event::Decided { request, outcome }));
    let mut out = Queue::with_capacity(1);
    out.push(Request::Decided);
    Requests::Out(out)
}

#[expect(clippy::too_many_arguments, reason = "the route retains its event fields")]
pub(super) fn route_historical_proposal(
    core: &mut Core,
    work: &mut Queue<Event>,
    request: Token,
    person: u64,
    project: u32,
    proposer: people::Proposer,
    proposal: u64,
    row: Option<crate::ProposalDecisionRecord>,
    busy: bool,
) -> Requests {
    let outcome = if busy {
        people::Outcome::Again(people::Again::Busy)
    } else if let Some(row) = row {
        let named = row.proposal == proposal
            && row.project == project
            && row.proposal <= core.counters.deployment().messages
            && row.by != 0
            && row.by <= core.counters.deployment().people
            && row.proposer == proposer;
        if named {
            let allowed = person == row.by
                || core.proposal_policy_standing(
                    person,
                    project,
                    match row.kind {
                        tasks::ProposalKind::Batch => authority::ProposalKind::Batch,
                        tasks::ProposalKind::Effect => authority::ProposalKind::Effect,
                        tasks::ProposalKind::Amend => authority::ProposalKind::Amend,
                        tasks::ProposalKind::Widen => authority::ProposalKind::Widen,
                        tasks::ProposalKind::Release => authority::ProposalKind::Escalation,
                    },
                );
            if allowed {
                people::Outcome::ProposalDecided { proposer, proposal, by: row.by, at: row.at, choice: row.choice }
            } else {
                people::Outcome::Refused(people::Refusal::Unknown)
            }
        } else {
            people::Outcome::Refused(people::Refusal::Unknown)
        }
    } else {
        people::Outcome::Refused(people::Refusal::Unknown)
    };
    work.push(Event::People(people::Event::Decided { request, outcome }));
    let mut out = Queue::with_capacity(1);
    out.push(Request::Decided);
    Requests::Out(out)
}
