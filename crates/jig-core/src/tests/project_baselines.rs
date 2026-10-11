//! Project edit baselines are joined from their retained owners, without
//! equating the configured target with the observed display period.
use super::*;

fn project(core: &Core, limits: &Limits) -> Box<Project> {
    match make(core, limits, 1, &Subject::Project { project: 1 }, &window()).expect("project document") {
        Document::Project(value) => value,
        _ => panic!("project document"),
    }
}

fn period(project: u32, number: u64, reserved: u64, closed: bool) -> tasks::FundingRecord {
    tasks::FundingRecord {
        period: Some(tasks::PeriodTimes { began: Wall::EPOCH, next: Some(Wall::EPOCH) }),
        funder: tasks::Funder::Period { project, period: number },
        parent: None,
        numbers: tasks::Numbers { budget: 1000, spent: 0, spent_below: 17, reserved },
        made: 0,
        closed,
    }
}

fn pool(project: u32, person: u64, number: u64, budget: u64, closed: bool) -> tasks::FundingRecord {
    tasks::FundingRecord {
        period: None,
        funder: tasks::Funder::Pool { project, person, period: number },
        parent: Some(tasks::Funder::Period { project, period: number }),
        numbers: tasks::Numbers { budget, spent: 0, spent_below: 7, reserved: 0 },
        made: 0,
        closed,
    }
}

fn restored_core(configured: u64, rows: &[tasks::FundingRecord]) -> (Core, Limits) {
    let (old, mut limits) = core();
    limits.tasks.funders = 8;
    let mut config = config(17);
    config.settings.period = configured;
    let mut core = Core::new(config, &limits);
    core.people = old.people;
    let mut out = Queue::with_capacity(tasks::max_out(&limits.tasks));
    let env = Env { now: Time::ZERO, wall: Wall::from_nanos(9999), limits: limits.tasks };
    for row in rows {
        tasks::step(&mut core.tasks, &env, tasks::Event::Restore { record: tasks::Stored::Ledger(*row) }, &mut out);
        assert_eq!(out.pop(), None, "each source ledger is admitted at restore");
    }
    tasks::step(&mut core.tasks, &env, tasks::Event::Restored, &mut out);
    assert_eq!(out.pop(), None, "complete source funding links are valid");
    (core, limits)
}

#[test]
fn project_pool_target_keeps_configured_zero_distinct_from_the_observed_period_and_closed_pool() {
    let closed = pool(1, 1, 0, 177, true);
    let observed = pool(1, 1, 9, 66, false);
    let (core, limits) = restored_core(0, &[period(1, 0, 0, true), closed, period(1, 9, 66, false), observed]);
    let value = project(&core, &limits);
    assert_eq!(value.configured_period, 0);
    assert_eq!(value.current_period.unwrap().source, tasks::Funder::Period { project: 1, period: 9 });
    assert_eq!(value.current_period.unwrap().began, Some(Wall::EPOCH));
    assert_eq!(value.current_period.unwrap().next, Some(Wall::EPOCH));
    assert_eq!(value.pools.get(0).unwrap().source, observed.funder);
    let target = value.configured_pools.get(0).unwrap();
    assert_eq!(target.source, closed.funder);
    assert_eq!(target.parent, closed.parent);
    assert_eq!(target.numbers, closed.numbers);
    assert_eq!(target.left, 170);
    assert!(target.closed);
    assert_eq!(target.began, None);
    assert_eq!(target.next, None);
    assert!(!value.configured_pools_cut);
}

#[test]
fn project_configured_target_absence_and_full_width_period_do_not_borrow_an_observed_pool() {
    let observed = pool(1, 1, 9, 66, false);
    for configured in [0, u64::MAX] {
        let (core, limits) = restored_core(configured, &[period(1, 9, 66, false), observed]);
        let value = project(&core, &limits);
        assert_eq!(value.configured_period, configured);
        assert_eq!(value.configured_pools.len(), 0);
        assert_eq!(value.configured_pools.capacity(), limits.documents.lines);
        assert!(
            List::<Ledger>::worst_case(value.configured_pools.capacity()).unwrap()
                > u64::try_from(size_of_val(value.configured_pools.as_slice())).unwrap()
        );
        assert!(!value.configured_pools_cut);
        assert_eq!(value.current_period.unwrap().source, tasks::Funder::Period { project: 1, period: 9 });
        assert_eq!(value.pools.get(0).unwrap().numbers, observed.numbers);
    }
}

#[test]
fn project_configured_pools_keep_source_order_and_report_their_loss_independently() {
    let rows = [
        period(1, 0, 40, false),
        pool(1, u64::MAX, 0, 23, false),
        pool(1, 1, 0, 19, true),
        pool(1, 7, 0, 17, false),
        period(1, 9, 0, false),
        period(2, 0, 31, false),
        pool(2, 1, 0, 31, false),
    ];
    let (mut core, mut limits) = restored_core(0, &rows);
    core.permission_roles.insert(1, Box::new([])).unwrap();
    limits.documents.lines = 3;
    let complete = project(&core, &limits);
    assert_eq!(complete.configured_pools.len(), 3);
    assert_eq!(complete.configured_pools.capacity(), 3);
    assert!(!complete.configured_pools_cut);
    let expected = [rows[2], rows[3], rows[1]];
    for (actual, expected) in complete.configured_pools.iter().zip(expected) {
        assert_eq!(actual.source, expected.funder);
        assert_eq!(actual.numbers, expected.numbers);
        assert_eq!(actual.closed, expected.closed);
    }
    assert_eq!(complete.pools.len(), 0, "observed period9 has no pools");
    limits.documents.lines = 2;
    let shortened = project(&core, &limits);
    assert_eq!(shortened.configured_pools.as_slice(), complete.configured_pools.as_slice().get(..2).unwrap());
    assert!(shortened.configured_pools_cut);
    assert!(shortened.cut);
    assert!(!shortened.policy.cut);
    assert_eq!(shortened.pools.len(), 0);
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
    let committed = crate::policy::snapshot(core.authority.policy(1).unwrap(), &mappings).unwrap();
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
