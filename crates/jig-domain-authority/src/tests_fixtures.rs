//! Shared admitted policy values for the authority unit tests.
use crate::{
    Authority, Budget, Delegation, Domain, Event, Grant, Guard, Implication, Implies, Judge, Last, Limits, Models,
    Numbers, POLICY_MAX_OUT, Pattern, Policy, PolicyFact, Proposals, Requests, Requirement, Role, Rules, Scopes, Tools,
    step,
};
use alloc::boxed::Box;
use skein_lib::{Duration, Queue, bytes::copy_of};

pub(crate) const LIMITS: Limits = Limits {
    projects: 2,
    roles: 2,
    requirements: 2,
    facts: 4,
    grants: 3,
    executors: 3,
    segments: 4,
    segment_bytes: 8,
    implications: 3,
    batch: 3,
    accounts: 2,
    writes: 2,
};

pub(crate) fn numbers(budget: u64) -> Numbers {
    Numbers { budget, spent: 0, spent_below: 0, reserved: 0 }
}

pub(crate) fn pattern() -> Pattern {
    Pattern { segments: Box::new([]), last: Last::Open(copy_of(b"")) }
}

pub(crate) fn authority() -> Authority {
    Authority {
        tools: Tools(u64::MAX),
        models: Models::ALL,
        grants: Box::new([Grant { connector: 1, kind: 3, pattern: pattern() }]),
        delegation: Delegation {
            kinds: Box::new([crate::Executor::Charter(1), crate::Executor::Procedure(2), crate::Executor::Role(3)]),
            tasks: 10,
            depth: 5,
        },
        budget: Budget { spend: 100, deadline: None },
        notes: Scopes(7),
        note_resources: Box::new([crate::ResourceScope { connector: 1, pattern: pattern() }]),
    }
}

pub(crate) fn requirement(fact: u16) -> Requirement {
    Requirement {
        connector: 1,
        kind: 1,
        pattern: pattern(),
        judge: Judge { connector: 1, requirement: fact, parameters: 0 },
        guard: Guard::Observed { freshness: Duration::from_secs(1) },
        must_be_guarded: false,
    }
}

pub(crate) fn rules(ceiling: Authority) -> Rules {
    Rules {
        ceiling,
        period_spend: 1_000,
        minimum_run_spend: 1,
        maximum_run_spend: 80,
        implies: Implies::new(
            Box::new([
                Implication { connector: 1, kind: 3, implies: 2 },
                Implication { connector: 1, kind: 2, implies: 1 },
                Implication { connector: 1, kind: 3, implies: 1 },
            ]),
            3,
        )
        .unwrap(),
        requirements: Box::new([requirement(7)]),
    }
}

pub(crate) fn role(ceiling: Authority) -> Role {
    Role { number: 7, authority: ceiling, period_spend: 100, requests: Requests::ALL, decides: Proposals::ALL }
}

pub(crate) fn policy(ceiling: Authority) -> Policy {
    Policy {
        projections: Box::new([]),
        escalation_role: None,
        roles: Box::new([role(ceiling.clone())]),
        ceiling,
        period_spend: 500,
        requirements: Box::new([requirement(8)]),
    }
}

pub(crate) fn apply(domain: &mut Domain, event: Event) -> PolicyFact {
    let mut out = Queue::with_capacity(POLICY_MAX_OUT);
    step(domain, event, &mut out);
    out.pop().unwrap()
}

pub(crate) fn domain_with(deployment: Authority, project: Authority) -> Domain {
    let mut domain = Domain::new(rules(deployment), LIMITS).unwrap();
    assert_eq!(
        apply(&mut domain, Event::Policy { project: 1, policy: policy(project) }),
        PolicyFact::Added { project: 1 }
    );
    domain
}

pub(crate) fn domain() -> Domain {
    domain_with(authority(), authority())
}

pub(crate) fn domain_with_roles(decides: Proposals, escalation_role: Option<u32>) -> Domain {
    let mut policy = policy(authority());
    let mut member = role(authority());
    member.number = 2;
    member.decides = decides;
    let mut holder = role(authority());
    holder.number = 5;
    policy.roles = Box::new([member, holder]);
    policy.escalation_role = escalation_role;
    let mut domain = Domain::new(rules(authority()), LIMITS).unwrap();
    assert_eq!(apply(&mut domain, Event::Policy { project: 1, policy }), PolicyFact::Added { project: 1 });
    domain
}

pub(crate) fn domain_with_role_spend(spend: u64) -> Domain {
    let mut member = role(authority());
    member.number = 2;
    member.authority.budget.spend = spend;
    let mut policy = policy(authority());
    policy.roles = Box::new([member]);
    let mut domain = Domain::new(rules(authority()), LIMITS).unwrap();
    assert_eq!(apply(&mut domain, Event::Policy { project: 1, policy }), PolicyFact::Added { project: 1 });
    domain
}

pub(crate) fn delegate(spend: u64) -> crate::Delegate {
    let mut given = authority();
    given.budget.spend = spend;
    given.delegation.tasks = 0;
    given.delegation.depth = 0;
    crate::Delegate { executor: crate::Executor::Charter(1), authority: given, symbolic: Box::new([]) }
}
