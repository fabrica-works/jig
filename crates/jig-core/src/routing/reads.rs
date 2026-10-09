//! Historical proposal and escalation answers and party read reservations
//! routed back to authenticated parties (`domain/engine.md`, sections 5.3 and 4.4).

use crate::{Core, EscalationDecisionRecord, Event, Limits, Now, Request, Requests};
use alloc::boxed::Box;
use jig_core_authority as authority;
use jig_core_people as people;
use jig_core_tasks as tasks;
use skein_lib::{Env, Queue, ReplyTo, Token};

pub(super) fn route_escalation_read(
    core: &mut Core,
    to: ReplyTo,
    person: u64,
    task: u64,
    context: Option<Box<tasks::EscalationContext>>,
) -> Requests {
    let answer = match context {
        Some(context) => {
            let role = core.people.role(person, context.project);
            if context.task == task && core.escalation_visible(&context, person, role) {
                Now::EscalationReply { to, person, context }
            } else {
                Now::EscalationRefused { to, why: people::Refusal::Standing }
            }
        }
        None => Now::EscalationRefused { to, why: people::Refusal::Unknown },
    };
    let mut out = Queue::with_capacity(2);
    out.push(Request::Now(Box::new(answer)));
    out.push(Request::Decided);
    Requests::Out(out)
}

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
        people::Outcome::Refused(people::Refusal::Busy)
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
                people::Outcome::Refused(people::Refusal::Standing)
            } else if bounded {
                let choice = match row.decision {
                    people::EscalationDecision::Release => people::EscalationChoice::Released,
                    people::EscalationDecision::Reject { .. } => people::EscalationChoice::Rejected,
                    people::EscalationDecision::Pass => people::EscalationChoice::Passed,
                };
                people::Outcome::EscalationDecided { task, revision, by: row.by, choice }
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
    proposer: u64,
    proposal: u64,
    row: Option<crate::ProposalDecisionRecord>,
    busy: bool,
) -> Requests {
    let outcome = if busy {
        people::Outcome::Refused(people::Refusal::Busy)
    } else if let Some(row) = row {
        let named = row.proposal == proposal
            && row.project == project
            && row.proposal <= core.counters.deployment().messages
            && row.by != 0
            && row.by <= core.counters.deployment().people
            && match row.proposer {
                tasks::Party::Person(number) | tasks::Party::Task(number) => number == proposer,
                tasks::Party::Deployment { .. } => false,
            };
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
                people::Outcome::ProposalDecided { proposer, proposal, by: row.by, choice: row.choice }
            } else {
                people::Outcome::Refused(people::Refusal::Standing)
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

impl Core {
    /// Whether this party already owns a historical result or inbox read.
    #[must_use]
    pub fn reading_result(&self, person: u64) -> bool {
        self.reading_results.contains_key(&person)
    }

    /// Reserve one root store read for an authenticated party.
    #[must_use]
    pub fn reserve_result_read(&mut self, person: u64, waiter: Token) -> bool {
        self.reading_results.insert(person, waiter) == Ok(None)
    }

    /// Release a completed or failed historical read.
    #[must_use]
    pub fn release_result_read(&mut self, person: u64, waiter: Token) -> bool {
        self.reading_results.remove(&person) == Some(waiter)
    }

    /// Advance the committed inbox cursor after the root assembled its final page.
    #[must_use]
    pub fn advance_inbox_read(&mut self, person: u64, high: u64) -> Option<people::Stored> {
        self.people.advance_read_position(person, high)
    }
}
