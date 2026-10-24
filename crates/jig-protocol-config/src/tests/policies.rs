//! Configured policies use the seed policy's parser and checks (configuration.md, 3).
use crate::{
    Path, Problem, Segment, What,
    tests::{domain, fixtures},
};

const FIRST: u32 = 1;
const SECOND: u32 = 2;
const POLICY_COUNT: u32 = 2;

fn configured() -> Box<[u8]> {
    // The policies have the same parts as seed projects, without names or owners.
    fixtures::with(&domain::two_projects(b'2'), br#""projects": ["#, br#""projects": [], "policies": ["#)
}
fn path(index: u32) -> Path {
    fixtures::path(&[b"policies"]).child(Segment::Index(index))
}
fn refuses(document: &[u8], path: Path, what: What) {
    let problems = fixtures::engine(document).expect_err("invalid configured policy");
    assert_eq!(problems.problems.as_slice(), &[Problem { path, what }]);
}
fn override_document(template: u32) -> Box<[u8]> {
    let entry = fixtures::with(
        br#""templates": [{"template": TEMPLATE, "executor": null, "budget": null, "deadline": null, "priority": 5}]"#,
        b"TEMPLATE",
        skein_lib::Decimal::of(u64::from(template)).as_bytes(),
    );
    fixtures::with(&configured(), br#""templates": []"#, &entry)
}

#[test]
fn two_configured_policies_parse_roles_and_template_overrides() {
    let engine = fixtures::engine(&override_document(SECOND)).expect("configured policies");
    assert_eq!(engine.policies.len(), usize::try_from(POLICY_COUNT).unwrap());
    assert_eq!([engine.policies[0].number, engine.policies[1].number], [FIRST, SECOND]);
    assert_eq!(
        engine.policies[0].roles.as_ref(),
        &[jig_domain_people::PermissionRole { connector: 1, permission: 1, role: 1 }]
    );
    assert_eq!(
        engine.policies[0].templates.as_ref(),
        &[jig_domain_people::TemplateOverride {
            template: SECOND,
            executor: None,
            budget: None,
            deadline: None,
            priority: Some(5)
        }]
    );
    assert_eq!(engine.policies[1].policy.period_spend, 100);
}
#[test]
fn an_absent_policies_member_is_empty() {
    assert!(fixtures::engine(fixtures::ENGINE_SMALL).expect("no configured policies").policies.is_empty());
}
#[test]
fn a_configured_policy_number_zero_is_refused() {
    refuses(
        &fixtures::with(&configured(), br#""policies": [{"number": 1"#, br#""policies": [{"number": 0"#),
        path(0).member(b"number"),
        What::Bound,
    );
}
#[test]
fn a_duplicate_policy_number_is_refused_at_the_second() {
    refuses(
        &fixtures::with(&configured(), br#""number": 2, "roles""#, br#""number": 1, "roles""#),
        path(1).member(b"number"),
        What::Unique,
    );
}
#[test]
fn a_policy_past_the_rules_ceiling_refuses_startup() {
    let document = fixtures::with(&configured(), br#""ceiling": {"tools": 1023"#, br#""ceiling": {"tools": 511"#);
    let problems = fixtures::engine(&document).expect_err("policy ceilings exceed the rules");
    assert_eq!(
        problems.problems.as_slice(),
        &[Problem { path: path(0), what: What::Bound }, Problem { path: path(1), what: What::Bound },]
    );
}
#[test]
fn an_override_must_name_a_configured_template() {
    refuses(
        &override_document(99),
        path(0).member(b"policy").member(b"templates").child(Segment::Index(0)).member(b"template"),
        What::Refers,
    );
}
#[test]
fn configured_policies_at_the_capacity_are_admitted() {
    let document = fixtures::with(
        &configured(),
        br#""profile": "small""#,
        br#""profile": "small", "overrides": {"authority.projects": 2}"#,
    );
    assert_eq!(
        fixtures::engine(&document).expect("policies at capacity").policies.len(),
        usize::try_from(POLICY_COUNT).unwrap()
    );
}
#[test]
fn too_many_configured_policies_are_refused_at_the_list() {
    let document = fixtures::with(
        &configured(),
        br#""profile": "small""#,
        br#""profile": "small", "overrides": {"authority.projects": 1}"#,
    );
    refuses(&document, fixtures::path(&[b"policies"]), What::Bound);
}
#[test]
fn configured_policies_reach_the_core_configuration() {
    let config = domain::core_config(&override_document(SECOND));
    let policy = config.defaults.policies.get(&FIRST).expect("configured policy");
    assert_eq!(policy.policy.period_spend, 100);
    assert_eq!(
        policy.permissions.as_ref(),
        &[jig_domain_people::PermissionRole { connector: 1, permission: 1, role: 1 }]
    );
    assert_eq!(policy.templates[0].template, SECOND);
    assert_eq!(policy.templates[0].priority, Some(5));
}

#[test]
fn configured_permissions_must_name_the_policys_roles() {
    let document = fixtures::with(&configured(), br#""permission": 1, "role": 1"#, br#""permission": 1, "role": 9"#);
    refuses(&document, path(0).member(b"roles").child(Segment::Index(0)).member(b"role"), What::Refers);
}

#[test]
fn seed_and_configured_policies_have_separate_capacity() {
    let document = fixtures::with(
        &configured(),
        br#""profile": "small""#,
        br#""profile": "small", "overrides": {"authority.projects": 2}"#,
    );
    let mut engine = fixtures::engine(&document).expect("configured policies at capacity");
    let seed = fixtures::with(fixtures::ENGINE_SMALL, br#""projects": [{"number": 1"#, br#""projects": [{"number": 5"#);
    engine.projects = fixtures::engine(&seed).expect("distinct seed project").projects;
    let mut problems = crate::Problems::empty();
    crate::domain::whole(&engine, &mut problems);
    assert!(problems.is_empty(), "{problems:?}");
}

#[test]
fn the_document_supplies_all_four_defaults_exactly() {
    let engine = fixtures::engine(&override_document(SECOND)).expect("document defaults");
    let config = crate::core(&engine, [3; 16], 11);
    assert_eq!(config.defaults.charters, engine.charters);
    assert_eq!(config.defaults.templates, engine.templates);
    assert_eq!(config.defaults.rules, engine.rules);
    assert_eq!(config.defaults.policies.len(), u32::try_from(engine.policies.len()).unwrap());
    for policy in &engine.policies {
        let value = config.defaults.policies.get(&policy.number).expect("numbered policy default");
        assert_eq!(value.policy, policy.policy);
        assert_eq!(value.permissions, policy.roles);
        assert_eq!(value.templates, policy.templates);
    }
}
