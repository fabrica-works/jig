//! Names are restored with policy and refused before either changes (core.md, 6.1).
use super::*;

const FIRST_PROJECT: u32 = 1;
const SECOND_PROJECT: u32 = 2;
const TWO_PROJECTS: u32 = 2;

fn restoring() -> (Core, Limits) {
    let (mut core, mut limits) = core();
    let policy = core.authority.policy(FIRST_PROJECT).expect("fixture policy").clone();
    limits.authority.projects = TWO_PROJECTS;
    core.authority = authority::Domain::new(core.authority.rules().clone(), limits.authority).expect("two policies");
    let mut out = Queue::with_capacity(authority::POLICY_MAX_OUT);
    for project in [FIRST_PROJECT, SECOND_PROJECT] {
        authority::step(&mut core.authority, authority::Event::Policy { project, policy: policy.clone() }, &mut out);
        out.pop().expect("fixture policy admitted");
    }
    core.permission_roles = Map::with_capacity(TWO_PROJECTS);
    core.template_overrides = Map::with_capacity(TWO_PROJECTS);
    core.restart_begin();
    core.restore_core(CoreRecord::Deployment(core.counters.deployment()), &limits);
    (core, limits)
}

fn snapshot(core: &Core, project: u32) -> people::PolicyValue {
    policy::snapshot(core.authority.policy(project).expect("fixture policy"), &[], &[]).expect("bounded policy")
}

#[test]
fn a_restored_project_name_cannot_collide_with_another_projects() {
    let (mut core, limits) = restoring();
    let first = snapshot(&core, FIRST_PROJECT);
    assert!(core.restore_policy(&limits, FIRST_PROJECT, Box::from("Tally"), first));
    let before = snapshot(&core, SECOND_PROJECT);
    let mut changed = before.clone();
    changed.period_spend = 1;
    assert!(!core.restore_policy(&limits, SECOND_PROJECT, Box::from("tally"), changed));
    assert_eq!(snapshot(&core, SECOND_PROJECT), before);
    assert_eq!(core.names.get(&SECOND_PROJECT), None);
    assert_eq!(core.names.get(&FIRST_PROJECT).expect("restored name").as_ref(), "Tally");
}

#[test]
fn a_restored_name_out_of_shape_keeps_policy_and_name() {
    let (mut core, limits) = restoring();
    let before = snapshot(&core, FIRST_PROJECT);
    assert!(core.restore_policy(&limits, FIRST_PROJECT, Box::from("Tally"), before.clone()));
    let long = "a".repeat(65); // One byte past core.md, 6.1's 64.
    for name in ["", long.as_str(), "Ta\u{7}lly", " Tally", "12"] {
        let mut changed = before.clone();
        changed.period_spend = 1;
        assert!(!core.restore_policy(&limits, FIRST_PROJECT, Box::from(name), changed), "{name:?}");
        assert_eq!(snapshot(&core, FIRST_PROJECT), before);
        assert_eq!(core.names.get(&FIRST_PROJECT).expect("restored name").as_ref(), "Tally");
    }
}
