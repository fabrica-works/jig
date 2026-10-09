//! Party goal, proposal and escalation choices routed to their owner
//! (`domain/people.md`, section 5.1). Person-origin proposals take precedence
//! over live task proposals, then historical reads. Current escalation
//! revisions take precedence over archived decisions, as in the former chain.

use super::PersonEscalation;
use super::refusals::pool_refused;
use crate::{Core, Event, GoalRoute, Limits, Now, PersonProposalRoute, Request};
use alloc::boxed::Box;
use jig_core_authority as authority;
use jig_core_people as people;
use jig_core_tasks as tasks;
use skein_lib::{Env, Queue, ReplyTo, Token};

#[expect(clippy::too_many_arguments, reason = "an exact pending goal carries the keyed holder decision")]
pub(super) fn goal_decide(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    request: Token,
    person: u64,
    role: Option<people::Role>,
    project: u32,
    proposer: u64,
    number: u64,
    choice: people::ProposalDecision,
) {
    let Some(proposal) = core.tasks.person_proposal(proposer, number) else {
        pool_refused(work, request, people::Refusal::Ended);
        return;
    };
    if let Err(why) = core.goal_standing(&proposal, project, role) {
        pool_refused(work, request, why);
        return;
    }
    let event = match choice {
        people::ProposalDecision::Accept => {
            if core.projections.len() >= env.limits.tasks.tasks {
                pool_refused(work, request, people::Refusal::Limit);
                return;
            }
            let source = match core.goal_accept_allowed(
                &proposal,
                project,
                person,
                role.expect("checked standing"),
                env.limits.tasks.tree_tasks,
            ) {
                Ok(source) => source,
                Err(why) => {
                    pool_refused(work, request, why);
                    return;
                }
            };
            let (period, pool) = core.goal_open_pool(project, person);
            if let Some(event) = period {
                work.push(Event::Tasks(event));
            }
            if let Some(event) = pool {
                work.push(Event::Tasks(event));
            }
            let mut goal = proposal.goal.clone();
            goal.funder = source;
            assert!(
                core.goal_routes.insert(
                    request,
                    GoalRoute::Accepting { proposer, proposal: number, by: person, task: goal.number },
                ) == Ok(None),
                "one goal acceptance"
            );
            tasks::Event::Make {
                reply_to: ReplyTo::new(request),
                creator: tasks::Party::Person(proposer),
                batch: Box::new([goal]),
            }
        }
        people::ProposalDecision::Reject { reason } => {
            let Some(message) = crate::fresh(&mut core.counters, crate::Family::Message) else {
                pool_refused(work, request, people::Refusal::Limit);
                return;
            };
            assert!(
                core.goal_routes.insert(request, GoalRoute::Deciding { proposer, proposal: number, by: person })
                    == Ok(None),
                "one goal decision"
            );
            tasks::Event::DecidePersonProposal {
                reply_to: ReplyTo::new(request),
                proposer,
                proposal: number,
                by: tasks::Party::Person(person),
                message: Some(message),
                decision: tasks::ProposalDecision::Reject { reason },
            }
        }
        people::ProposalDecision::Pass => {
            pool_refused(work, request, people::Refusal::NoFurther);
            return;
        }
    };
    work.push(Event::Tasks(event));
}

#[expect(clippy::too_many_arguments, reason = "a keyed proposal decision carries its holder and exact live proposal")]
pub(super) fn person_proposal_decide(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    request: Token,
    person: u64,
    role: Option<people::Role>,
    project: u32,
    proposer: u64,
    number: u64,
    choice: people::ProposalDecision,
) {
    let Some(proposal) = core.tasks.proposal(proposer, number) else {
        pool_refused(work, request, people::Refusal::Ended);
        return;
    };
    if proposal.project != project {
        pool_refused(work, request, people::Refusal::Standing);
        return;
    }
    let current = match proposal.state {
        tasks::ProposalState::Pending { holder, .. } => holder,
        tasks::ProposalState::Accepted { .. }
        | tasks::ProposalState::Rejected { .. }
        | tasks::ProposalState::Withdrawn => {
            pool_refused(work, request, people::Refusal::Ended);
            return;
        }
    };
    let (_, proposal_kind) = Core::proposal_kind(&proposal.action);
    let standing = match current {
        tasks::ProposalHolder::Person(holder) => holder == person,
        tasks::ProposalHolder::Policy { project: named, .. } => {
            named == project && core.proposal_policy_standing(person, project, proposal_kind)
        }
        tasks::ProposalHolder::Task(_) => false,
    };
    if !standing || role.is_none() {
        pool_refused(work, request, people::Refusal::Standing);
        return;
    }
    let next = match choice {
        people::ProposalDecision::Accept => {
            let Some(action) = Core::proposal_action_for_check(&proposal.action) else {
                pool_refused(work, request, people::Refusal::Authority);
                return;
            };
            if !core.proposal_covers_person(
                &env.limits,
                &authority::needs(&action).expect("admitted proposal needs"),
                person,
                project,
                proposal_kind,
            ) {
                pool_refused(work, request, people::Refusal::Authority);
                return;
            }
            person_proposal_accept(core, work, request, person, proposal);
            return;
        }
        people::ProposalDecision::Reject { reason } => tasks::ProposalDecision::Reject { reason },
        people::ProposalDecision::Pass => match current {
            tasks::ProposalHolder::Person(_) => tasks::ProposalDecision::Pass {
                holder: tasks::ProposalHolder::Policy { project, kind: Core::proposal_kind(&proposal.action).0 },
            },
            tasks::ProposalHolder::Policy { .. } => {
                pool_refused(work, request, people::Refusal::NoFurther);
                return;
            }
            tasks::ProposalHolder::Task(_) => unreachable!("person standing checked"),
        },
    };
    let message = match next {
        tasks::ProposalDecision::Reject { .. } => match crate::fresh(&mut core.counters, crate::Family::Message) {
            Some(message) => Some(message),
            None => {
                pool_refused(work, request, people::Refusal::Limit);
                return;
            }
        },
        tasks::ProposalDecision::Pass { .. } => None,
        tasks::ProposalDecision::Accept => unreachable!("accept routed separately"),
    };
    assert!(
        core.routing_people_proposals
            .insert(request, PersonProposalRoute::Deciding { request, proposer, proposal: number, by: person },)
            == Ok(None),
        "one person proposal route"
    );
    work.push(Event::PersonProposal(tasks::Event::DecideProposal {
        reply_to: ReplyTo::new(request),
        proposer,
        proposal: number,
        message,
        by: tasks::Party::Person(person),
        decision: next,
    }));
}

#[expect(clippy::too_many_lines, reason = "one exhaustive authenticated person proposal dispatcher")]
pub(super) fn person_proposal_accept(
    core: &mut Core,
    work: &mut Queue<Event>,
    request: Token,
    person: u64,
    proposal: tasks::Proposal,
) {
    let result_followups = core.tasks.result_proposal(proposal.proposer, proposal.number);
    match proposal.action {
        tasks::ProposalAction::Effect { connector, .. } => {
            work.push(Event::EffectStart {
                owner: request,
                connector,
                origin: crate::EffectOrigin::Accept {
                    to: ReplyTo::new(request),
                    key: None,
                    proposal: Box::new(proposal),
                    by: tasks::Party::Person(person),
                },
            });
            return;
        }
        tasks::ProposalAction::Batch(_)
        | tasks::ProposalAction::Amend { .. }
        | tasks::ProposalAction::Widen { .. }
        | tasks::ProposalAction::Release { .. } => {}
    }
    let event = match proposal.action {
        tasks::ProposalAction::Effect { .. } => unreachable!("effect accepted through its connector"),
        tasks::ProposalAction::Batch(mut batch) => {
            let creator =
                if proposal.as_holder { tasks::Party::Person(person) } else { tasks::Party::Task(proposal.proposer) };
            let (period, pool) = core.goal_open_pool(proposal.project, person);
            if let Some(event) = period {
                work.push(Event::Tasks(event));
            }
            if let Some(event) = pool {
                work.push(Event::Tasks(event));
            }
            for member in &mut batch {
                member.funder = tasks::Funder::Pool { project: proposal.project, person, period: core.settings.period };
            }
            if result_followups {
                tasks::Event::MakeResultFollowups {
                    reply_to: ReplyTo::new(request),
                    proposer: proposal.proposer,
                    proposal: proposal.number,
                    batch,
                }
            } else {
                tasks::Event::Make { reply_to: ReplyTo::new(request), creator, batch }
            }
        }
        tasks::ProposalAction::Amend { task, amendment } => {
            let Some(current) = core.tasks.delegation(task) else {
                pool_refused(work, request, people::Refusal::Ended);
                return;
            };
            let Some(amend_message) = crate::fresh(&mut core.counters, crate::Family::Message) else {
                pool_refused(work, request, people::Refusal::Limit);
                return;
            };
            let after = crate::translate::authority_value(amendment.authority.as_ref().expect("admitted authority"));
            let before = crate::translate::authority_value(&current.authority);
            let stop_run = !authority::at_most(&before, &after, &core.authority.rules().implies);
            tasks::Event::Amend {
                reply_to: ReplyTo::new(request),
                by: tasks::Party::Person(person),
                task,
                message: amend_message,
                stop_run,
                amendment,
            }
        }
        tasks::ProposalAction::Widen { task, authority } => {
            let Some(amend_message) = crate::fresh(&mut core.counters, crate::Family::Message) else {
                pool_refused(work, request, people::Refusal::Limit);
                return;
            };
            tasks::Event::Amend {
                reply_to: ReplyTo::new(request),
                by: tasks::Party::Person(person),
                task,
                message: amend_message,
                stop_run: false,
                amendment: tasks::Amendment {
                    spec: None,
                    wake: None,
                    dependencies: None,
                    authority: Some(authority),
                    reason: proposal.reason,
                },
            }
        }
        tasks::ProposalAction::Release { task } => tasks::Event::Control {
            reply_to: ReplyTo::new(request),
            by: tasks::Party::Person(person),
            task,
            control: tasks::Control::Release,
        },
    };
    let Some(message) = crate::fresh(&mut core.counters, crate::Family::Message) else {
        pool_refused(work, request, people::Refusal::Limit);
        return;
    };
    assert!(
        core.routing_people_proposals.insert(
            request,
            PersonProposalRoute::Accepting {
                request,
                person,
                proposer: proposal.proposer,
                proposal: proposal.number,
                message,
            },
        ) == Ok(None),
        "one person acceptance route"
    );
    work.push(Event::PersonProposal(event));
}

#[expect(clippy::too_many_arguments, reason = "one keyed held choice carries its person and exact semantic revision")]
pub(super) fn person_escalation_decide(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    request: Token,
    person: u64,
    role: Option<people::Role>,
    project: u32,
    task: u64,
    revision: u64,
    decision: people::EscalationDecision,
) {
    if let people::EscalationDecision::Reject { reason } = &decision
        && reason.len() > usize::try_from(env.limits.escalation_reason_bytes).expect("reason bound fits usize")
    {
        pool_refused(work, request, people::Refusal::Limit);
        return;
    }
    let Some(context) = core.tasks.escalation(task) else {
        pool_refused(work, request, people::Refusal::Unknown);
        return;
    };
    if context.project != project || !core.escalation_visible(&context, person, role) {
        pool_refused(work, request, people::Refusal::Standing);
        return;
    }
    let pending = match context.escalation {
        tasks::Escalation::Waiting { revision: current, .. } => current == revision,
        tasks::Escalation::Unheld { .. } | tasks::Escalation::Routing { .. } | tasks::Escalation::Rejected { .. } => {
            false
        }
    };
    if !pending {
        pool_refused(work, request, people::Refusal::Unknown);
        return;
    }
    if !Core::escalation_standing(&context, person, role) {
        pool_refused(work, request, people::Refusal::NotWaiting);
        return;
    }
    let semantic = match &decision {
        people::EscalationDecision::Release => {
            if !core.escalation_covers(&env.limits, &context, person, role)
                || !core.escalation_release_allowed(&context, person, role, env.limits.tasks.tree_tasks)
            {
                pool_refused(work, request, people::Refusal::Authority);
                return;
            }
            tasks::EscalationDecision::Release
        }
        people::EscalationDecision::Reject { reason } => tasks::EscalationDecision::Reject { reason: reason.clone() },
        people::EscalationDecision::Pass => {
            let Some(holder) = core.escalation_fallback(project) else {
                pool_refused(work, request, people::Refusal::Authority);
                return;
            };
            tasks::EscalationDecision::Pass { holder }
        }
    };
    let entry = match semantic {
        tasks::EscalationDecision::Pass { .. } => match crate::fresh(&mut core.counters, crate::Family::Message) {
            Some(entry) => Some(entry),
            None => {
                pool_refused(work, request, people::Refusal::Limit);
                return;
            }
        },
        tasks::EscalationDecision::Release | tasks::EscalationDecision::Reject { .. } => None,
    };
    assert!(
        core.person_escalations.insert(
            request,
            PersonEscalation { request, requester: context.requester, person, project, task, revision, decision },
        ) == Ok(None),
        "one keyed escalation choice"
    );
    work.push(Event::PersonEscalation(tasks::Event::DecideEscalation {
        reply_to: ReplyTo::new(request),
        task,
        revision,
        by: person,
        entry,
        decision: semantic,
    }));
}

pub(super) fn current_person_escalation(core: &Core, task: u64, revision: u64) -> bool {
    let Some(context) = core.tasks.escalation(task) else { return false };
    match context.escalation {
        tasks::Escalation::Waiting { revision: current, .. } => current == revision,
        tasks::Escalation::Unheld { .. } | tasks::Escalation::Routing { .. } | tasks::Escalation::Rejected { .. } => {
            false
        }
    }
}

#[expect(clippy::too_many_arguments, reason = "the route retains the admitted party and request fields")]
pub(super) fn proposal_request(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    request: Token,
    person: u64,
    project: u32,
    role: Option<people::Role>,
    out: &mut Queue<Request>,
    proposer: u64,
    proposal: u64,
    decision: people::ProposalDecision,
) {
    if core.tasks.person_proposal(proposer, proposal).is_some() {
        goal_decide(core, env, work, request, person, role, project, proposer, proposal, decision);
    } else if core.tasks.proposal(proposer, proposal).is_some() {
        person_proposal_decide(core, env, work, request, person, role, project, proposer, proposal, decision);
    } else {
        out.push(Request::Now(Box::new(Now::HistoricalProposal { request, person, project, proposer, proposal })));
    }
}

#[expect(clippy::too_many_arguments, reason = "the route retains the admitted party and request fields")]
pub(super) fn escalation_request(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    request: Token,
    person: u64,
    project: u32,
    role: Option<people::Role>,
    out: &mut Queue<Request>,
    task: u64,
    revision: u64,
    decision: people::EscalationDecision,
) {
    if current_person_escalation(core, task, revision) {
        person_escalation_decide(core, env, work, request, person, role, project, task, revision, decision);
        return;
    }

    if match &decision {
        people::EscalationDecision::Reject { reason } => {
            reason.len() > usize::try_from(env.limits.escalation_reason_bytes).expect("reason bound fits usize")
        }
        people::EscalationDecision::Release | people::EscalationDecision::Pass => false,
    } {
        pool_refused(work, request, people::Refusal::Limit);
        return;
    }
    if let Some(context) = core.tasks.escalation(task)
        && (context.project != project || !core.escalation_visible(&context, person, role))
    {
        pool_refused(work, request, people::Refusal::Standing);
        return;
    }
    if let Some(context) = core.tasks.escalation(task)
        && context.escalation.revision() != revision
    {
        pool_refused(work, request, crate::revisions::moved_hold(context.escalation.revision(), context.why));
        return;
    }
    out.push(Request::Now(Box::new(Now::HistoricalEscalation { request, person, project, task, revision })));
}
