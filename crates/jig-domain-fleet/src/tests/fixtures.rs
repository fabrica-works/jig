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

pub const ANALYST_FIRST: crate::Grant = crate::Grant {
    table: crate::GrantTable::Accounts,
    account: 1,
    generation: 1,
    valid: skein_lib::Duration::from_secs(60),
};
pub const ANALYST_SECOND: crate::Grant = crate::Grant { generation: 2, ..ANALYST_FIRST };
pub const OTHER_ACCOUNT: crate::Grant = crate::Grant { account: 2, ..ANALYST_FIRST };

#[must_use]
pub fn needs() -> crate::Needs {
    crate::Needs { charters: Box::new([]), spend: 0, message_bytes: 0, start_messages: 0, unacknowledged: 0 }
}
