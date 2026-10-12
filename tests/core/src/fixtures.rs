//! Shared host bounds for the domain world.
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
