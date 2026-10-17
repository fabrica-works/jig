//! Independent core literals for the single-read proof.
use super::{authority, chip, task_with_hold};
use alloc::boxed::Box;
use jig_domain_core::{documents as d, reads};
use jig_domain_tasks as t;
use skein_lib::{List, Wall};
pub fn pending_proposal() -> d::Proposal {
    let mut funding = List::with_capacity(2);
    for (source, needed, left, after) in
        [(t::Funder::Task(17), 19, None, Some(23)), (t::Funder::Period { project: 7, period: 29 }, 31, Some(0), None)]
    {
        funding.push(d::FundingOffer { source, needed, left, after }).expect("two offers");
    }
    d::Proposal {
        acts: List::with_capacity(0),
        proposer: d::Proposer::Party(11),
        number: 13,
        project: 7,
        reason: d::Text { bytes: Box::from(b"pending".as_slice()), cut: false },
        action: d::ProposalAction::Release { task: d::TaskLink::Stored { number: 37, status: None }, hold: None },
        as_holder: true,
        waiting: d::ProposalWaiting {
            holder: t::ProposalHolder::Policy { project: 7, kind: t::ProposalKind::Amend },
            since: Wall::from_nanos(41),
        },
        funding,
    }
}
pub fn failed_result() -> d::ResultDoc {
    d::ResultDoc {
        acts: List::with_capacity(0),
        task: chip(),
        contract: d::ContractLine::Report { words: 53 },
        ending: d::Ending::Failed(d::Text { bytes: Box::from(b"failed".as_slice()), cut: false }),
        at: Some(Wall::from_nanos(47)),
        requester: t::Party::Person(59),
        executor: t::Executor::Agent { charter: 7 },
        chosen_by: Some(61),
        followups: Some(List::with_capacity(0)),
        delegates: List::with_capacity(0),
        proposal: None,
        cut: true,
    }
}
pub fn historical_task() -> reads::Historical {
    reads::Historical {
        value: reads::Past::Task { document: Box::new(task_with_hold()), result: Box::new(failed_result()) },
        at: Some(Wall::from_nanos(67)),
        by: Some(t::Party::Person(71)),
    }
}
pub fn decided_escalation(decision: jig_domain_people::EscalationDecision) -> reads::Historical {
    reads::Historical {
        value: reads::Past::Escalation {
            task: Box::new(task_with_hold()),
            decision: jig_domain_core::EscalationDecisionRecord {
                project: 7,
                requester: 73,
                task: 7,
                revision: 79,
                by: 83,
                at: Wall::from_nanos(89),
                decision,
            },
            hold: None,
        },
        at: Some(Wall::from_nanos(97)),
        by: Some(t::Party::Person(101)),
    }
}
pub fn project_document() -> d::Project {
    d::Project {
        acts: List::with_capacity(0),
        number: 7,
        resources: List::with_capacity(0),
        policy: d::Policy {
            templates: List::with_capacity(0),
            ceiling: authority(),
            period_spend: 7,
            escalation_role: None,
            projections: List::with_capacity(0),
            roles: List::with_capacity(0),
            requirements: List::with_capacity(0),
            permissions: None,
            permissions_cut: false,
            cut: false,
        },
        members: List::with_capacity(0),
        current_period: None,
        pools: List::with_capacity(0),
        configured_period: 7,
        configured_pools: List::with_capacity(0),
        configured_pools_cut: false,
        spend: List::with_capacity(0),
        recurring: List::with_capacity(0),
        cut: false,
    }
}

pub fn template() -> jig_domain_core::Template {
    jig_domain_core::Template {
        number: 1,
        name: b"task".as_slice().into(),
        executor: jig_domain_people::TemplateExecutor::Charter(1),
        contract: t::Contract::Report { words: 32 },
        shallower: 1,
        budget: jig_domain_people::TemplateBudget::Amount(1),
        deadline: jig_domain_people::TemplateDeadline::Given,
        tracked: None,
        flags: t::Flags::PLAIN,
        label: 0,
        grace: None,
    }
}
