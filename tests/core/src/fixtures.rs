//! Shared host bounds for the domain world.
pub const WORKSPACE_CREDENTIAL: u32 = 7;
pub const CREDENTIAL_RESOURCE: u64 = 1;
#[must_use]
pub fn bounds() -> jig_domain_fleet::Bounds {
    use skein_lib::Duration;
    jig_domain_fleet::Bounds {
        models: Box::new([
            jig_domain_fleet::ModelBound { name: b"scripted".as_slice().into(), window: 8192, output: 1024 },
            jig_domain_fleet::ModelBound { name: b"research".as_slice().into(), window: 8192, output: 1024 },
        ]),
        turns: u32::MAX,
        spend: u64::MAX,
        time: Duration::from_secs(3600),
        waiting: Duration::from_secs(3600),
        checks: Duration::from_nanos(1),
        message_bytes: u32::MAX,
        inbox: u32::MAX,
        unacknowledged: u32::MAX,
    }
}

#[must_use]
pub fn live_row(world: &crate::effects::World, task: u64) -> &jig_domain_tasks::TaskRecord {
    match world
        .store
        .rows
        .get(&jig_test_domain::Key::Core(jig_domain_core::Key::Tasks(jig_domain_tasks::Key::Live(task))))
    {
        Some(jig_test_domain::Record::Core(jig_domain_core::Record::Tasks(jig_domain_tasks::Stored::Live(row)))) => row,
        row => panic!("live task row missing: {row:?}"),
    }
}

pub const QUERY_REQUESTER: u64 = 1;

/// A requesting party's task query, shared by the independent watch stories.
pub const OWN_TASKS: jig_domain_core::documents::TaskQuery = jig_domain_core::documents::TaskQuery {
    project: None,
    requester: Some(jig_domain_tasks::Party::Person(QUERY_REQUESTER)),
    root: None,
    parent: None,
    label: None,
    phase: None,
    executor: None,
    tracked: None,
    order: jig_domain_core::documents::QueryOrder::Number,
};
