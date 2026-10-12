pub fn bounds() -> crate::Bounds {
    use skein_lib::Duration;
    crate::Bounds {
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

pub fn adopt(run: skein_lib::Token, attempt: skein_lib::Token, kept: u32) -> crate::Event {
    crate::Event::Adopt {
        reply_to: skein_lib::ReplyTo::new(attempt),
        run,
        attempt,
        kept,
        kind: crate::HostKind::Worker,
        worked: false,
        holds: Box::new([]),
        grants: Box::new([]),
    }
}
