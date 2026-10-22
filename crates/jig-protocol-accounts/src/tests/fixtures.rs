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
    crate::table::Value { token: Secret::new(bytes::copy_of(token)), scheme: crate::table::Scheme::Bearer }
}

pub fn driver_limits() -> driver::Limits {
    driver::Limits {
        accounts: 2,
        file_stall: Duration::from_secs(1),
        lock_wait: Duration::from_secs(1),
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
            metadata_bytes: 0,
            key_set_bytes: 0,
            keys: 0,
            modulus_bits: 2048,
            skew: Duration::ZERO,
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

pub fn record(account: u32, token: &[u8], generation: u64) -> skein_oauth::SavedToken {
    skein_oauth::SavedToken {
        key: account,
        generation,
        access_token: bytes::copy_of(token),
        refresh_token: Some(bytes::copy_of(b"refresh-token")),
        metadata: None,
        expires_at: skein_lib::Wall::from_nanos(Duration::from_secs(60).as_nanos()),
        issued_client_id: None,
        granted_scopes: None,
        subject: None,
        id_token: None,
        earliest_refresh_at: None,
    }
}
