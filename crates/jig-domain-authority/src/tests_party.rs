//! Whether a party's roots have an escalation holder, and how much their next task may take.
use alloc::boxed::Box;

use crate::{
    Domain, Event, Executor, Models, Numbers, PersonAsk, PersonRequest, PolicyFact, Proposals, as_allowed,
    tests_fixtures::{self as fixtures, LIMITS, apply, authority, domain_with_role_spend, domain_with_roles, numbers},
};

const SPENDS: [u64; 3] = [20, 60, 110];
const PROJECT: u32 = 1;
const MEMBER: u32 = 2;
const HOLDER: u32 = 5;
const UNKNOWN: u32 = 99;
const POOLS: [(u64, u64); 2] = [(100, 80), (10, 10)];
const CAPS: [(u64, u64); 3] = [(0, 0), (20, 20), (100, 50)];
const UNKNOWN_ROLES: [(u32, u32); 2] = [(UNKNOWN, MEMBER), (PROJECT, UNKNOWN)];
const NAMED: [u64; 3] = [80, 100, u64::MAX];

#[test]
fn a_role_deciding_escalations_holds_its_own_roots() {
    let domain = domain_with_roles(Proposals(16), None);
    assert!(domain.escalations_held(PROJECT, MEMBER));
}

#[test]
fn a_policy_escalation_role_holds_any_roles_roots() {
    let domain = domain_with_roles(Proposals(0), Some(HOLDER));
    assert!(domain.escalations_held(PROJECT, MEMBER));
}

#[test]
fn no_escalation_role_and_no_deciding_role_holds_nothing() {
    let domain = domain_with_roles(Proposals(0), None);
    assert!(!domain.escalations_held(PROJECT, MEMBER));
}

#[test]
fn an_unknown_project_or_role_holds_nothing() {
    let domain = domain_with_roles(Proposals(16), Some(HOLDER));
    for (project, role) in [(UNKNOWN, MEMBER), (PROJECT, UNKNOWN)] {
        assert!(!domain.escalations_held(project, role), "project {project}, role {role}");
    }
}

/// Three configured models, the deployment's rules naming them all.
const CONFIGURED: Models = Models::Named(0b111);

/// A domain whose rules, policy ceiling and member role name these models.
fn domain_naming(rules: Models, policy: Models, role: Models) -> Domain {
    let mut deployment = authority();
    deployment.models = rules;
    let mut ceiling = authority();
    ceiling.models = policy;
    let mut member = fixtures::role(authority());
    member.number = MEMBER;
    member.authority.models = role;
    let mut project = fixtures::policy(ceiling);
    project.roles = Box::new([member]);
    let mut domain = Domain::new(fixtures::rules(deployment), LIMITS).unwrap();
    assert_eq!(
        apply(&mut domain, Event::Policy { project: PROJECT, policy: project }),
        PolicyFact::Added { project: PROJECT }
    );
    domain
}

#[test]
fn a_party_may_use_every_configured_model_when_nothing_narrows_them() {
    let domain = domain_naming(CONFIGURED, Models::ALL, Models::ALL);
    assert_eq!(domain.models(PROJECT, MEMBER), Some(CONFIGURED));
}

#[test]
fn a_policy_naming_fewer_models_narrows_a_partys() {
    let domain = domain_naming(CONFIGURED, Models::Named(0b101), Models::ALL);
    assert_eq!(domain.models(PROJECT, MEMBER), Some(Models::Named(0b101)));
}

#[test]
fn rules_and_policy_narrow_to_their_intersection_in_configurations_order() {
    for (rules, policy) in [(Models::Named(0b011), Models::Named(0b110)), (Models::Named(0b110), Models::Named(0b011))]
    {
        let domain = domain_naming(rules, policy, Models::ALL);
        assert_eq!(domain.models(PROJECT, MEMBER), Some(Models::Named(0b010)), "rules {rules:?}, policy {policy:?}");
    }
}

#[test]
fn a_role_naming_fewer_models_narrows_its_partys() {
    let domain = domain_naming(CONFIGURED, Models::Named(0b011), Models::Named(0b110));
    assert_eq!(domain.models(PROJECT, MEMBER), Some(Models::Named(0b010)));
}

#[test]
fn a_procedures_models_are_empty() {
    let domain = domain_naming(CONFIGURED, Models::ALL, Models::ALL);
    let models = domain.models(PROJECT, MEMBER).unwrap();
    assert_eq!(models.run_by(Executor::Procedure(2)), Models::NONE);
    for executor in [Executor::Charter(1), Executor::Role(3)] {
        assert_eq!(models.run_by(executor), CONFIGURED, "{executor:?}");
    }
}

#[test]
fn an_unknown_project_or_role_has_no_models() {
    let domain = domain_naming(CONFIGURED, Models::ALL, Models::ALL);
    for (project, role) in UNKNOWN_ROLES {
        assert_eq!(domain.models(project, role), None, "project {project}, role {role}");
    }
}

#[test]
fn as_allowed_gives_what_the_pool_has_left_under_the_role() {
    let domain = domain_with_role_spend(80);
    for (budget, given) in POOLS {
        let ask = PersonAsk {
            project: PROJECT,
            role: MEMBER,
            pool: numbers(budget),
            tasks_left: 8,
            request: PersonRequest::Watch,
        };
        assert_eq!(as_allowed(&domain, &ask, 0, None), given, "pool budget {budget}");
    }
}

#[test]
fn as_allowed_counts_the_batchs_named_amounts_first() {
    let domain = domain_with_role_spend(80);
    let ask =
        PersonAsk { project: PROJECT, role: MEMBER, pool: numbers(100), tasks_left: 8, request: PersonRequest::Watch };
    assert_eq!(as_allowed(&domain, &ask, 30, None), 50);
}

#[test]
fn as_allowed_stops_at_the_cap_the_batch_names() {
    let domain = domain_with_role_spend(80);
    let ask =
        PersonAsk { project: PROJECT, role: MEMBER, pool: numbers(100), tasks_left: 8, request: PersonRequest::Watch };
    for (cap, given) in CAPS {
        assert_eq!(as_allowed(&domain, &ask, 30, Some(cap)), given, "cap {cap}");
    }
}

#[test]
fn as_allowed_for_an_unknown_role_gives_nothing() {
    let domain = domain_with_role_spend(80);
    for (project, role) in UNKNOWN_ROLES {
        let ask = PersonAsk { project, role, pool: numbers(100), tasks_left: 8, request: PersonRequest::Watch };
        assert_eq!(as_allowed(&domain, &ask, 0, None), 0, "project {project}, role {role}");
    }
}

#[test]
fn as_allowed_gives_nothing_when_named_amounts_take_the_pool() {
    let domain = domain_with_role_spend(80);
    let ask =
        PersonAsk { project: PROJECT, role: MEMBER, pool: numbers(100), tasks_left: 8, request: PersonRequest::Watch };
    for named in NAMED {
        assert_eq!(as_allowed(&domain, &ask, named, None), 0, "named {named}");
    }
}

#[test]
fn as_allowed_over_generated_pools_roles_and_caps_is_the_least_bound() {
    let mut rng = skein_lib::Rng::new(24);
    for case in 0_u32..64 {
        let spend = rng.below(101);
        let domain = domain_with_role_spend(spend);
        let pool = Numbers {
            budget: rng.below(201),
            spent: rng.below(51),
            spent_below: rng.below(51),
            reserved: rng.below(51),
        };
        let named = rng.below(101);
        let cap = if rng.below(2) == 0 { Some(rng.below(101)) } else { None };
        let ask = PersonAsk { project: PROJECT, role: MEMBER, pool, tasks_left: 8, request: PersonRequest::Watch };
        let remaining =
            pool.budget.saturating_sub(pool.spent).saturating_sub(pool.spent_below).saturating_sub(pool.reserved);
        let expected = remaining.min(spend).saturating_sub(named).min(cap.unwrap_or(u64::MAX));
        assert_eq!(as_allowed(&domain, &ask, named, cap), expected, "case {case}: {ask:?}, named {named}, cap {cap:?}");
    }
}

fn batch_request(spend: u64) -> PersonAsk {
    PersonAsk {
        project: PROJECT,
        role: MEMBER,
        pool: numbers(100),
        tasks_left: 8,
        request: PersonRequest::Create(Box::new([crate::tests_fixtures::delegate(spend)])),
    }
}

#[test]
fn a_partys_batch_beyond_their_role_answers_propose() {
    let domain = domain_with_role_spend(40);
    let mut why = skein_lib::Queue::with_capacity(crate::max_out(domain.limits()).unwrap());
    assert_eq!(crate::check_request(&domain, &batch_request(60), &mut why).answer, crate::Answer::Propose);
}

#[test]
fn a_partys_batch_beyond_the_project_ceiling_answers_refuse() {
    let domain = domain_with_role_spend(40);
    let mut why = skein_lib::Queue::with_capacity(crate::max_out(domain.limits()).unwrap());
    assert_eq!(crate::check_request(&domain, &batch_request(110), &mut why).answer, crate::Answer::Refuse);
}

#[test]
fn a_partys_widening_beyond_their_role_answers_propose() {
    let domain = domain_with_role_spend(40);
    let mut ask = batch_request(60);
    ask.request = PersonRequest::Amend(crate::tests_fixtures::delegate(60).authority);
    let mut why = skein_lib::Queue::with_capacity(crate::max_out(domain.limits()).unwrap());
    assert_eq!(crate::check_request(&domain, &ask, &mut why).answer, crate::Answer::Propose);
}

#[test]
fn one_batch_answers_alike_under_a_task_and_under_a_party() {
    let domain = domain_with_role_spend(40);
    for spend in SPENDS {
        let person = batch_request(spend);
        let PersonRequest::Create(tasks) = &person.request else { unreachable!("batch fixture") };
        let task = crate::BatchAsk {
            project: PROJECT,
            creator: domain.role(PROJECT, MEMBER).unwrap().authority.clone(),
            numbers: person.pool,
            tasks_left: person.tasks_left,
            tasks: tasks.clone(),
        };
        let mut task_why = skein_lib::Queue::with_capacity(crate::max_out(domain.limits()).unwrap());
        let mut person_why = skein_lib::Queue::with_capacity(crate::max_out(domain.limits()).unwrap());
        assert_eq!(
            crate::check_batch(&domain, &task, &mut task_why),
            crate::check_request(&domain, &person, &mut person_why),
            "spend {spend}"
        );
        assert_eq!(task_why.len(), person_why.len(), "spend {spend}");
        for (task_finding, person_finding) in task_why.iter().zip(&person_why) {
            assert_eq!(*task_finding, task_source(*person_finding), "spend {spend}");
        }
    }
}

#[test]
fn a_partys_widening_beyond_their_pool_answers_propose() {
    let domain = domain_with_role_spend(80);
    let mut ask = batch_request(60);
    ask.pool = numbers(40);
    ask.request = PersonRequest::Amend(crate::tests_fixtures::delegate(60).authority);
    let mut why = skein_lib::Queue::with_capacity(crate::max_out(domain.limits()).unwrap());
    assert_eq!(crate::check_request(&domain, &ask, &mut why).answer, crate::Answer::Propose);
}

fn task_source(finding: crate::Finding) -> crate::Finding {
    use crate::Finding;
    let source = |source| if source == crate::Source::Role { crate::Source::Task } else { source };
    match finding {
        Finding::Authority { source: origin, lacks } => Finding::Authority { source: source(origin), lacks },
        Finding::Executor { source: origin } => Finding::Executor { source: source(origin) },
        Finding::Tasks { source: origin } => Finding::Tasks { source: source(origin) },
        Finding::Spend { source: origin } => Finding::Spend { source: source(origin) },
        Finding::Grant { source: origin } => Finding::Grant { source: source(origin) },
        Finding::Price { source: origin } => Finding::Price { source: source(origin) },
        Finding::Scope { source: origin } => Finding::Scope { source: source(origin) },
        Finding::Oversized
        | Finding::UnknownProject
        | Finding::UnknownRole
        | Finding::Arithmetic
        | Finding::RunBudget
        | Finding::RunCap
        | Finding::Deadline
        | Finding::Account
        | Finding::Writer
        | Finding::Tool
        | Finding::ResourceAccess
        | Finding::ResourceUnavailable
        | Finding::Reference
        | Finding::Required { .. }
        | Finding::Failed { .. }
        | Finding::Unguarded { .. }
        | Finding::Unpermitted
        | Finding::Undecidable
        | Finding::PeriodSpend => finding,
    }
}
