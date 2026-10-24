//! Names and whole policy rows restore together (core.md, 6.1; store.md, 4.4).
use super::*;

const FIRST_PROJECT: u32 = 1;
const SECOND_PROJECT: u32 = 2;
const TWO_PROJECTS: u32 = 2;

fn restoring() -> (Core, Limits, people::PolicyValue) {
    let mut limits = limits();
    limits.authority.projects = TWO_PROJECTS;
    let mut config = config(41);
    let value = policy::snapshot(&config.seed_projects[0].policy.policy, &[], &[]).expect("bounded policy");
    config.defaults.rules = (authority::Domain::new(config.defaults.rules.clone(), limits.authority)
        .expect("two policies"))
    .rules()
    .clone();
    config.seed_projects = Box::new([]);
    let mut core = Core::new(config, &limits).expect("restoring core");
    core.counters.deployment.projects = u64::from(SECOND_PROJECT);
    core.restart_begin();
    core.restore_core(CoreRecord::Deployment(core.counters.deployment()), &limits);
    (core, limits, value)
}

#[test]
fn a_restored_project_name_cannot_collide_with_another_projects() {
    let (mut core, limits, value) = restoring();
    core.restore_policy(&limits, FIRST_PROJECT, Box::from("Tally"), value.clone());
    assert!(!core.restore_policy(&limits, SECOND_PROJECT, Box::from("tally"), value));
    assert_eq!(core.authority.policy(SECOND_PROJECT), None);
    assert_eq!(core.names.get(&SECOND_PROJECT), None);
    assert_eq!(core.projects.as_slice(), [FIRST_PROJECT]);
    assert_eq!(core.names.get(&FIRST_PROJECT).expect("restored name").as_ref(), "Tally");
}

#[test]
fn a_restored_name_out_of_shape_keeps_the_project_absent() {
    let (mut core, limits, value) = restoring();
    let long = "a".repeat(65); // One byte past core.md, 6.1's 64.
    for name in ["", long.as_str(), "Ta\u{7}lly", " Tally", "12"] {
        assert!(!core.restore_policy(&limits, FIRST_PROJECT, Box::from(name), value.clone()), "{name:?}");
        assert_eq!(core.authority.policy(FIRST_PROJECT), None);
        assert!(core.projects.is_empty());
        assert!(core.names.is_empty());
    }
}

#[test]
fn a_duplicate_project_row_keeps_the_first_whole_policy() {
    let (mut core, limits, value) = restoring();
    core.restore_policy(&limits, FIRST_PROJECT, Box::from("Tally"), value.clone());
    let before = core.authority.policy(FIRST_PROJECT).cloned();
    let mut changed = value;
    changed.period_spend = 1;
    assert!(!core.restore_policy(&limits, FIRST_PROJECT, Box::from("Other"), changed));
    assert_eq!(core.authority.policy(FIRST_PROJECT).cloned(), before);
    assert_eq!(core.names.get(&FIRST_PROJECT).expect("first name").as_ref(), "Tally");
    assert_eq!(core.projects.as_slice(), [FIRST_PROJECT]);
}

#[test]
fn a_restored_policy_past_the_current_rules_keeps_its_live_project() {
    let (mut core, limits, mut value) = restoring();
    value.ceiling.spend = core.authority.rules().ceiling.budget.spend + 1;
    assert!(core.restore_policy(&limits, FIRST_PROJECT, Box::from("Tally"), value));
    assert!(core.authority.policy(FIRST_PROJECT).is_some());
    assert_eq!(core.projects.as_slice(), [FIRST_PROJECT]);
    assert_eq!(core.names.get(&FIRST_PROJECT).unwrap().as_ref(), "Tally");
}
