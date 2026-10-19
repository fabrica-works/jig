use skein_lib::{Duration, Secret, Time, bytes};
use skein_oauth_accounts as driver;
pub const ACCOUNT_CHAT: u32 = 0;
pub const ACCOUNT_DELEGATE: u32 = 1;
pub const TOKEN_FIRST: &[u8] = b"token-first";
pub const TOKEN_SECOND: &[u8] = b"token-second";
pub const NOW: Time = Time::from_nanos(5);
pub fn granted(account: u32, token: &[u8], generation: u64, valid: Duration) -> driver::Event {
    driver::Event::Granted { account, token: bytes::copy_of(token), generation, valid }
}

pub fn value(token: &[u8]) -> crate::table::Value {
    crate::table::Value { token: Secret::new(bytes::copy_of(token)), account_id: Secret::new(bytes::copy_of(b"")) }
}

pub fn driver_limits() -> driver::Limits {
    driver::Limits {
        accounts: 2,
        file_stall: Duration::from_secs(1),
        exchanges: 2,
        listeners: 1,
        server: skein_http::server::Limits {
            head: 2048,
            headers: 16,
            body: 1024,
            read: 256,
            response: 2048,
            send: 256,
        },
        refresh_lead: Duration::from_secs(10),
        client: skein_oauth::ClientLimits {
            document: skein_oauth::Limits {
                document_bytes: 1024,
                string_bytes: 256,
                token_bytes: 256,
                client_bytes: 64,
                detail_bytes: 64,
                record_bytes: 1024,
                issued_client_bytes: 0,
                granted_bytes: 64,
                subject_bytes: 0,
                id_token_bytes: 256,
                depth: 8,
                tokens: 64,
            },
            uri_bytes: 256,
            scope_bytes: 64,
            state_bytes: 64,
            code_bytes: 64,
            url_bytes: 1024,
            request_bytes: 1024,
            parameter_bytes: 0,
            resource_bytes: 0,
            nonce_bytes: 0,
            sign_in_time: Duration::from_secs(120),
            request_time: Duration::from_secs(10),
            backoff_base: Duration::from_secs(1),
            backoff_ceiling: Duration::from_secs(4),
            max_attempts: 2,
        },
        http: skein_http::client::Limits { request: 2048, head: 2048, headers: 16, read: 256, send: 256 },
        tls: skein_tls::client::Limits { read: 2048, send: 2048, records: 131_072 },
        io: skein_io::Limits {
            sockets: 4,
            refusals: 1,
            intake: 32_768,
            receive: 1024,
            output: 32_768,
            sends: 4,
            accepts: 1,
            backlog: 2,
            close_timeout: Duration::from_secs(1),
            names: 0,
            retry: Duration::from_millis(1),
        },
    }
}

pub fn handed_request(token: &[u8], lapses: skein_lib::Wall) -> Result<driver::Request, crate::handed_in::Unfit> {
    crate::handed_in::hand_in(
        ACCOUNT_CHAT,
        Secret::new(bytes::copy_of(token)),
        lapses,
        Some(bytes::copy_of(b"account-id")),
        &driver_limits().client.document,
    )
}

pub fn first_grant(request: driver::Request, wall: skein_lib::Wall) -> driver::Event {
    let limits = driver_limits();
    let env = skein_lib::Env { now: NOW, wall, limits };
    let mut configured = skein_lib::List::with_capacity(1);
    assert!(configured.push(driver::Account::HandedIn).is_ok());
    let crypto = skein_lib::CryptoRng::new(&Secret::new(bytes::copy_of(&[7; 32])), 0).expect("crypto seed");
    let mut component = driver::Component::new(configured, &limits, crypto).expect("driver");
    let mut above = skein_lib::Queue::with_capacity(16);
    let mut io = skein_lib::Queue::with_capacity(16);
    let mut files = skein_lib::Queue::with_capacity(16);
    component.down(&env, request, &mut above, &mut io, &mut files);
    assert!(above.is_empty(), "checked HandIn is admitted");
    component.down(&env, driver::Request::Grant { account: ACCOUNT_CHAT }, &mut above, &mut io, &mut files);
    above.pop().expect("grant terminal")
}
