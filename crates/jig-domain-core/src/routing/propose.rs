//! Explicit party proposals and withdrawals (domain/people.md, section 5.1).
use super::{create, party_inputs, refusals::pool_refused};
use crate::{Core, Event, GoalRoute, Limits, Now, Request};
use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Env, List, Queue, ReplyTo, Token};

pub(super) fn propose_request(
    core: &mut Core,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
    work: &mut Queue<Event>,
    mut creation: create::Creation,
    reason: Box<[u8]>,
) {
    let request = creation.request;
    let result = check(core, env, &mut creation);
    match result {
        Ok((members, to)) => party_inputs::begin(
            core,
            env,
            out,
            work,
            party_inputs::Inputs {
                person: creation.person,
                project: creation.project,
                role: creation.role,
                members,
                given: None,
                proposal: Some((reason, to)),
                ids: Box::new([]),
                at: 0,
                row: None,
                stubs: List::with_capacity(0),
            },
            request,
        ),
        Err(why) => pool_refused(work, request, why),
    }
}
fn check(
    core: &mut Core,
    env: &Env<Limits>,
    creation: &mut create::Creation,
) -> Result<(Box<[tasks::New]>, u32), people::Refusal> {
    let role = core
        .authority
        .role(creation.project, creation.role.number())
        .ok_or(people::Refusal::Role { held: Some(creation.role) })?;
    if !role.requests.allows(authority::RequestKind::Create) {
        return Err(people::Refusal::Role { held: Some(creation.role) });
    }
    for (at, member) in creation.batch.iter().enumerate() {
        match member.budget {
            people::Budget::AsAllowed { .. } => {
                return Err(people::Refusal::NotWellFormed {
                    at: u32::try_from(at).expect("bounded batch"),
                    why: people::Malformed::ProposedAsAllowed,
                });
            }
            people::Budget::Named => {}
        }
    }
    let (members, _) = create::checked(core, env, creation, true)?;
    let Some(crate::acts::Recipient::Roles { roles, .. }) =
        crate::acts::bounds::recipient(core, creation.project, authority::ProposalKind::Batch)
    else {
        return Err(nobody(people::Lack::Approver));
    };
    let mut to = *roles.get(0).expect("nonempty recipient roles");
    for &role in &roles {
        to = to.min(role);
    }
    Ok((members, to))
}
pub(super) fn allowed(core: &Core, project: u32, batch: Box<[authority::Delegate]>) -> Result<(), people::Refusal> {
    let needs = authority::needs(&authority::Action::Batch(batch))
        .ok_or(people::Refusal::Limit { which: people::LimitKind::Arithmetic })?;
    let policy = core.authority.policy(project).ok_or(people::Refusal::Unknown)?;
    let rules = core.authority.rules();
    let cap = policy.period_spend.min(rules.period_spend);
    if needs.budget.spend > cap {
        return Err(nobody(people::Lack::Budget { needs: needs.budget.spend, left: cap }));
    }

    if !authority::at_most(&needs, &policy.ceiling, &rules.implies)
        || !authority::at_most(&needs, &rules.ceiling, &rules.implies)
    {
        return Err(nobody(people::Lack::Shape));
    }
    Ok(())
}
fn nobody(lacking: people::Lack) -> people::Refusal {
    people::Refusal::Authority { lacking, proposable: Some(people::Accepter::Nobody) }
}
pub(super) struct Proposal {
    pub request: Token,
    pub person: u64,
    pub reason: Box<[u8]>,
    pub to: u32,
    pub batch: Box<[tasks::New]>,
    pub stubs: Box<[tasks::Stub]>,
}
pub(super) fn keep(core: &mut Core, env: &Env<Limits>, work: &mut Queue<Event>, value: Proposal) {
    let Proposal { request, person, reason, to, batch, stubs } = value;
    let Some(number) = crate::fresh(&mut core.counters, crate::Family::Message) else {
        pool_refused(work, request, people::Refusal::Limit { which: people::LimitKind::Arithmetic });
        return;
    };
    let project = batch.first().expect("nonempty proposed batch").project;
    assert!(
        core.goal_routes.insert(request, GoalRoute::Offering { proposal: number, to, stubs }).is_ok(),
        "one proposal flight"
    );
    work.push(Event::Tasks(tasks::Event::ProposePerson {
        reply_to: ReplyTo::new(request),
        proposal: tasks::PersonProposal {
            number,
            proposer: person,
            project,
            action: tasks::PersonAction::Batch(batch),
            reason,
            state: tasks::PersonProposalState::Pending { since: env.wall },
        },
    }));
}
pub(super) struct Withdrawal {
    pub request: Token,
    pub person: u64,
    pub project: u32,
    pub proposal: u64,
}
pub(super) fn withdraw_request(core: &mut Core, work: &mut Queue<Event>, out: &mut Queue<Request>, value: Withdrawal) {
    let Withdrawal { request, person, project, proposal } = value;
    if let Some(row) = core.tasks.person_proposal(person, proposal) {
        if row.project != project {
            pool_refused(work, request, people::Refusal::Unknown);
            return;
        }
        assert!(
            core.goal_routes.insert(request, GoalRoute::Withdrawing { proposer: person, proposal }).is_ok(),
            "one withdrawal flight"
        );
        work.push(Event::Tasks(tasks::Event::WithdrawPersonProposal {
            reply_to: ReplyTo::new(request),
            proposer: person,
            proposal,
        }));
    } else {
        out.push(Request::Now(Box::new(Now::HistoricalProposal {
            request,
            person,
            project,
            proposer: people::Proposer::Party(person),
            proposal,
        })));
    }
}
