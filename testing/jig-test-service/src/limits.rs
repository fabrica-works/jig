//! The application's limits, priced with jig's in one sum (configuration.md, 6).
use jig_protocol_config::{Profile, profile};
use jig_test_connector as connector;
use jig_test_domain as root;
use skein_lib::{Duration, Queue};

pub const SMALL: connector::Limits = connector::Limits {
    tasks: 8,
    adoptions: 16,
    subscriptions: 16,
    pools: 4,
    resources: 16,
    topics: 4,
    kinds: 8,
    requirements: 4,
    procedures: 4,
    actions_per_procedure: 8,
    facts: 16,
    judges: 4,
    values: 8,
    value_bytes: 4096,
    staged: 8,
    entries: 8,
    made: 16,
    resources_per_effect: 4,
    max_attempts: 3,
    write_lifetime: Duration::from_secs(30),
    clock_margin: Duration::from_secs(5),
    resources_per_task: 4,
    subscribers_per_topic: 8,
    lost_per_pool: 8,
    path_segments: 4,
    segment_bytes: 256,
};
pub const STANDARD: connector::Limits = connector::Limits {
    tasks: 32,
    adoptions: 64,
    subscriptions: 64,
    resources: 64,
    facts: 64,
    values: 32,
    staged: 32,
    entries: 32,
    made: 64,
    ..SMALL
};

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub root: root::Limits,
    pub io: skein_io::Limits,
}
#[must_use]
pub fn for_profile(name: Profile, jig: &jig_protocol_config::Limits) -> Limits {
    let connector = match name {
        Profile::Small => SMALL,
        Profile::Standard => STANDARD,
    };
    Limits { root: root::Limits { core: jig.core, connector, journal: jig.journal, routes: jig.routes }, io: jig.io }
}
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    let root = root::worst_case(&limits.root)?;
    let queue = root::max_out(&limits.root).checked_mul(2)?;
    let payload = jig_domain_core::worst_case(&limits.root.core)?
        .checked_add(connector::worst_case(&limits.root.connector)?.checked_mul(2)?)?;
    root.checked_add(skein_io::worst_case(&limits.io)?)?
        .checked_add(Queue::<crate::StoreRequest>::worst_case(queue)?)?
        .checked_add(Queue::<crate::StoreAnswer>::worst_case(queue)?)?
        .checked_add(Queue::<crate::PartyOutput>::worst_case(queue)?)?
        .checked_add(Queue::<jig_domain_core::Event>::worst_case(queue)?)?
        .checked_add(Queue::<root::Event>::worst_case(queue)?)?
        .checked_add(Queue::<root::Request>::worst_case(root::max_out(&limits.root))?)?
        .checked_add(payload.checked_mul(u64::from(queue).checked_mul(6)?.checked_add(1)?)?)?
        .checked_add(crate::system::worst_case(&limits.root.connector)?.checked_mul(2)?)?
        .checked_add(Queue::<skein_io::kernel::Complete>::worst_case(queue)?)?
        .checked_add(Queue::<skein_io::kernel::Submit>::worst_case(queue)?)?
        .checked_add(Queue::<skein_io::Event>::worst_case(queue)?)
}
#[must_use]
pub fn profile_limits(name: Profile) -> Limits {
    for_profile(name, &profile(name))
}
