//! Whether a party's roots have an escalation holder.
use crate::{Proposals, tests_fixtures::domain_with_roles};

const PROJECT: u32 = 1;
const MEMBER: u32 = 2;
const HOLDER: u32 = 5;
const UNKNOWN: u32 = 99;

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
