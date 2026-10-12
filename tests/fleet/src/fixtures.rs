//! Shared host bounds for the domain world.
#[must_use]
pub fn bounds() -> jig_domain_fleet::Bounds {
    use skein_lib::Duration;
    jig_domain_fleet::Bounds {
        models: Box::new([]),
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

/// A live run and an answered run named by the memory scenario.
#[must_use]
pub fn hosting_pair(
    r1: skein_lib::Token,
    a1: skein_lib::Token,
    r2: skein_lib::Token,
    a2: skein_lib::Token,
) -> Vec<jig_domain_fleet::Hosted> {
    use jig_domain_fleet::{Hosted, Phase};
    vec![Hosted { run: r1, attempt: a1, phase: Phase::Active }, Hosted { run: r2, attempt: a2, phase: Phase::Answered }]
}

/// The account granted to the fleet world's scripted attempts.
pub const ANALYST: jig_domain_fleet::Grant = jig_domain_fleet::Grant {
    table: jig_domain_fleet::GrantTable::Accounts,
    account: 1,
    generation: 1,
    valid: skein_lib::Duration::from_secs(60),
};
