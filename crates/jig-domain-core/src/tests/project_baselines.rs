//! Project permission edit baselines are joined from their retained owner.
use super::*;

#[expect(clippy::wildcard_enum_match_arm, reason = "ordinary proof helper receives a Project document")]
fn project(core: &Core, limits: &Limits) -> Box<Project> {
    match make(core, limits, 1, &Subject::Project { project: 1 }, &window()).expect("project document") {
        Document::Project(value) => value,
        _ => panic!("project document"),
    }
}

#[test]
fn project_permission_baselines_keep_missing_and_supplied_empty_entries_distinct() {
    let (mut core, limits) = tree_core();
    core.permission_roles.remove(&1);
    let absent = project(&core, &limits);
    assert_eq!(absent.policy.permissions, None);
    assert!(!absent.policy.permissions_cut);
    core.permission_roles.insert(1, Box::new([])).unwrap();
    let empty = project(&core, &limits);
    let mappings = empty.policy.permissions.unwrap();
    assert_eq!(mappings.len(), 0);
    assert_eq!(mappings.capacity(), limits.documents.lines);
    assert!(
        List::<people::PermissionRole>::worst_case(mappings.capacity()).unwrap()
            > u64::try_from(size_of_val(mappings.as_slice())).unwrap()
    );
    assert!(!empty.policy.permissions_cut);
    assert!(!empty.policy.cut);
}

#[test]
fn project_permission_baselines_copy_restored_full_width_mappings_and_independent_loss() {
    let (mut core, mut limits) = tree_core();
    let mut policy = core.authority.policy(1).unwrap().clone();
    let mut full_width = policy.roles[0].clone();
    full_width.number = u32::MAX;
    policy.roles = Box::new([policy.roles[0].clone(), full_width]);
    limits.authority.roles = 2;
    core.authority = authority::Domain::new(core.authority.rules().clone(), limits.authority).unwrap();
    let mut facts = Queue::with_capacity(authority::POLICY_MAX_OUT);
    authority::step(&mut core.authority, authority::Event::Policy { project: 1, policy }, &mut facts);
    assert_eq!(facts.pop(), Some(authority::PolicyFact::Added { project: 1 }));
    let mappings = [
        people::PermissionRole { connector: u16::MAX, permission: u16::MAX, role: u32::MAX },
        people::PermissionRole { connector: 0, permission: 0, role: 0 },
        people::PermissionRole { connector: 7, permission: 9, role: u32::MAX },
    ];
    let committed = policy::snapshot(core.authority.policy(1).unwrap(), &mappings, &[]).unwrap();
    assert!(core.restore_policy(&limits, 1, committed));
    limits.documents.lines = 3;
    let complete = project(&core, &limits);
    assert_eq!(complete.policy.permissions.as_ref().unwrap().as_slice(), mappings);
    assert_eq!(complete.policy.permissions.as_ref().unwrap().capacity(), 3);
    assert!(!complete.policy.permissions_cut);
    assert!(!complete.policy.cut);
    limits.documents.lines = 2;
    let shortened = project(&core, &limits);
    assert_eq!(shortened.policy.permissions.as_ref().unwrap().as_slice(), &mappings[..2]);
    assert!(shortened.policy.permissions_cut);
    assert!(shortened.policy.cut);
    assert!(!shortened.policy.ceiling.cut);
    assert!(!shortened.configured_pools_cut);
    assert!(!shortened.cut);
}
