use super::{Queue, authority};
pub(super) fn set_role_authority(domain: &mut authority::Domain, value: authority::Authority) {
    let mut policy = domain.policy(1).unwrap().clone();
    policy.roles[0].authority = value;
    let mut rules = domain.rules().clone();
    let delegation = &policy.roles[0].authority.delegation;
    rules.ceiling.delegation.tasks = rules.ceiling.delegation.tasks.max(delegation.tasks);
    rules.ceiling.delegation.depth = rules.ceiling.delegation.depth.max(delegation.depth);
    policy.ceiling.delegation.tasks = policy.ceiling.delegation.tasks.max(delegation.tasks);
    policy.ceiling.delegation.depth = policy.ceiling.delegation.depth.max(delegation.depth);
    let bounds = *domain.limits();
    *domain = authority::Domain::new(rules, bounds).unwrap();
    let mut out = Queue::with_capacity(authority::POLICY_MAX_OUT);
    authority::step(domain, authority::Event::Policy { project: 1, policy }, &mut out);
    assert_eq!(out.pop(), Some(authority::PolicyFact::Added { project: 1 }));
}

pub(super) fn set_seed_role_authority(config: &mut super::Config, value: authority::Authority) {
    let mut rules = config.authority.rules().clone();
    let policy = &mut config.seed_projects[0].policy.policy;
    let delegation = &value.delegation;
    rules.ceiling.delegation.tasks = rules.ceiling.delegation.tasks.max(delegation.tasks);
    rules.ceiling.delegation.depth = rules.ceiling.delegation.depth.max(delegation.depth);
    policy.ceiling.delegation.tasks = policy.ceiling.delegation.tasks.max(delegation.tasks);
    policy.ceiling.delegation.depth = policy.ceiling.delegation.depth.max(delegation.depth);
    policy.roles[0].authority = value;
    config.authority = authority::Domain::new(rules, *config.authority.limits()).expect("seed role rules");
}
