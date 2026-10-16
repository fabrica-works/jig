//! Party proposal shape and durable withdrawal (domain/tasks.md, section 9).
use crate::{
    Domain, Limits, PersonAction, PersonProposal, PersonProposalState, ProposalOutcome, Refusal, Request, Stored,
};
use alloc::boxed::Box;
use skein_lib::{Env, List, Queue, ReplyTo};

pub(crate) fn valid_action(proposal: &PersonProposal, limits: &Limits) -> bool {
    let PersonAction::Batch(batch) = &proposal.action;
    if !crate::batch::bounded_batch(limits, batch) {
        return false;
    }
    for member in batch {
        if member.number == 0
            || member.project != proposal.project
            || member.recurring.is_some()
            || !crate::wake::valid(&member.wake)
            || member.numbers.spent != 0
            || member.numbers.spent_below != 0
            || member.numbers.reserved != 0
        {
            return false;
        }
        match member.executor {
            crate::Executor::Agent { charter: 0 }
            | crate::Executor::Procedure { connector: 0, .. }
            | crate::Executor::Procedure { code: 0, .. }
            | crate::Executor::Person(crate::PersonAddress::Person(0)) => return false,
            crate::Executor::Agent { .. } | crate::Executor::Procedure { .. } | crate::Executor::Person(_) => {}
        }
        match member.funder {
            crate::Funder::Pool { project, person, .. }
                if project == proposal.project && person == proposal.proposer => {}
            crate::Funder::Pool { .. }
            | crate::Funder::Task(_)
            | crate::Funder::Period { .. }
            | crate::Funder::Recurring { .. } => return false,
        }
    }
    true
}

pub(crate) fn made(domain: &Domain, action: &PersonAction) -> bool {
    let PersonAction::Batch(batch) = action;
    for member in batch {
        if crate::domain::record(domain, member.number).is_none() {
            return false;
        }
    }
    true
}

pub(crate) fn withdraw(
    domain: &mut Domain,
    env: &Env<Limits>,
    to: Option<ReplyTo>,
    proposer: u64,
    number: u64,
    out: &mut Queue<Request>,
) {
    let own = match domain.person_proposals.get(&number) {
        Some(row) => row.proposer == proposer,
        None => false,
    };
    let refusal = if !domain.ready() {
        Some(Refusal::NotReady)
    } else if own {
        None
    } else {
        Some(Refusal::Unknown)
    };
    if let Some(why) = refusal {
        if let Some(to) = to {
            crate::domain::refused(to, None, why, out);
        }
        return;
    }
    let mut proposal = domain.person_proposals.remove(&number).expect("pending proposal checked");
    proposal.state = PersonProposalState::Withdrawn { at: env.wall };
    out.push(Request::Erase { key: crate::Key::PersonProposal(number) });
    out.push(Request::Save { record: Stored::DecidedProposal(Box::new(proposal)) });
    crate::closing::prune_stubs(domain, env, out);
    if let Some(reply_to) = to {
        out.push(Request::PersonProposalDecided { reply_to, proposer, number, outcome: ProposalOutcome::Withdrawn });
    }
}

pub(crate) fn ended(domain: &mut Domain, env: &Env<Limits>, task: u64, out: &mut Queue<Request>) {
    let mut pending = List::with_capacity(env.limits.tasks);
    for (&number, proposal) in &domain.person_proposals {
        let PersonAction::Batch(batch) = &proposal.action;
        let mut names = false;
        for member in batch {
            if member.dependencies.contains(&task) {
                names = true;
                break;
            }
        }
        if names {
            pending.push((number, proposal.proposer)).expect("bounded pending proposals");
        }
    }
    for (number, proposer) in pending.into_boxed() {
        withdraw(domain, env, None, proposer, number, out);
    }
}
