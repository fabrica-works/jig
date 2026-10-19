use jig_domain_accounts as child;
use skein_fake_oauth as fake;
use skein_io as io;
use skein_lib::{Duration, Wall, bytes};
use skein_oauth as oauth;
use skein_oauth_accounts as accounts;
pub const ACCOUNT_CHAT: u32 = 0;
pub const ROOM: u32 = 64;
pub const TOKEN_FIRST: &[u8] = b"access-first";
pub const TOKEN_SECOND: &[u8] = b"access-second";
pub const AUTHORIZE: &[u8] = b"http://127.0.0.1:31000/authorize";
pub const TOKEN: &[u8] = b"http://127.0.0.1:31000/token";
pub const REDIRECT: &[u8] = b"http://localhost:31234/callback";
#[must_use]
pub fn limits() -> accounts::Limits {
    accounts::Limits {
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
        client: oauth::ClientLimits {
            document: oauth::Limits {
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
        io: io::Limits {
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

#[must_use]
pub fn child_limits() -> child::Limits {
    child::Limits {
        accounts: 2,
        credentials: 0,
        backoff_base: Duration::from_secs(2),
        backoff_max: Duration::from_secs(60),
        retry: Duration::from_secs(30),
        spent_attention: Duration::from_secs(300),
        facts: 16,
    }
}
#[must_use]
pub fn registration() -> oauth::Registration {
    oauth::Registration {
        authorization_url: bytes::copy_of(AUTHORIZE),
        token_endpoint: bytes::copy_of(TOKEN),
        client_id: bytes::copy_of(b"client"),
        redirect_uri: bytes::copy_of(REDIRECT),
        scope: bytes::copy_of(b"read"),
        wire: oauth::WireFormat::Form,
        client_secret: None,
        pkce_for_confidential: false,
        metadata_claim: None,
        terms: None,
    }
}
#[must_use]
pub fn kept() -> oauth::SavedToken {
    oauth::SavedToken {
        key: ACCOUNT_CHAT,
        generation: 1,
        access_token: bytes::copy_of(TOKEN_FIRST),
        refresh_token: Some(bytes::copy_of(b"seed")),
        metadata: None,
        expires_at: Wall::from_nanos(Duration::from_secs(300).as_nanos()),
        issued_client_id: None,
        granted_scopes: None,
        subject: None,
        id_token: None,
        earliest_refresh_at: None,
    }
}
#[must_use]
pub fn rotate() -> fake::Plan {
    fake::Plan {
        status: 200,
        body: fake::Body::Token(oauth::TokenResponse {
            access_token: bytes::copy_of(TOKEN_SECOND),
            refresh_token: Some(bytes::copy_of(b"next")),
            expires_in: 300,
            scope: None,
            id_token: None,
            earliest_refresh_at: None,
        }),
        delay: Duration::from_millis(5),
        retry_after: Duration::ZERO,
    }
}
#[must_use]
pub fn refuse() -> fake::Plan {
    fake::Plan {
        status: 400,
        body: fake::Body::Raw(bytes::copy_of(b"{\"error\":\"invalid_grant\"}")),
        delay: Duration::from_millis(5),
        retry_after: Duration::ZERO,
    }
}

pub const ACCOUNT_HANDED: u32 = 0;
#[must_use]
pub fn handed_lapse(story: crate::world::Story) -> Option<Duration> {
    match story {
        crate::world::Story::HandedIn { lapses_in } => Some(lapses_in),
        crate::world::Story::HandedInRejected => Some(Duration::from_secs(60)),
        crate::world::Story::Granted
        | crate::world::Story::Refreshed
        | crate::world::Story::Closed
        | crate::world::Story::RateLimitedRefresh
        | crate::world::Story::RefusedRefresh
        | crate::world::Story::RejectedClient
        | crate::world::Story::DamagedRecord
        | crate::world::Story::ClosedExchange => None,
    }
}
#[must_use]
pub fn rejects(story: crate::world::Story) -> bool {
    matches!(
        story,
        crate::world::Story::Refreshed
            | crate::world::Story::HandedInRejected
            | crate::world::Story::RateLimitedRefresh
            | crate::world::Story::RefusedRefresh
            | crate::world::Story::RejectedClient
            | crate::world::Story::ClosedExchange
    )
}
#[must_use]
pub fn wanted(story: crate::world::Story) -> usize {
    match story {
        crate::world::Story::HandedIn { .. } => 3,
        crate::world::Story::Granted | crate::world::Story::Closed | crate::world::Story::DamagedRecord => 1,
        crate::world::Story::Refreshed
        | crate::world::Story::HandedInRejected
        | crate::world::Story::RateLimitedRefresh
        | crate::world::Story::RefusedRefresh
        | crate::world::Story::RejectedClient
        | crate::world::Story::ClosedExchange => 2,
    }
}
#[must_use]
pub fn expected(story: crate::world::Story) -> Vec<crate::owner::Heard> {
    use crate::{owner::Heard, world::Story};
    match story {
        Story::Refreshed => vec![
            Heard::Granted { account: ACCOUNT_CHAT, generation: 1 },
            Heard::Granted { account: ACCOUNT_CHAT, generation: 2 },
        ],
        Story::HandedIn { .. } => vec![
            Heard::Granted { account: ACCOUNT_HANDED, generation: 1 },
            Heard::Expiring { account: ACCOUNT_HANDED, generation: 1 },
            Heard::Failed { account: ACCOUNT_HANDED, failure: child::Failure::SignInAgain },
        ],
        Story::HandedInRejected => vec![
            Heard::Granted { account: ACCOUNT_HANDED, generation: 1 },
            Heard::Failed { account: ACCOUNT_HANDED, failure: child::Failure::SignInAgain },
        ],
        Story::RateLimitedRefresh => vec![
            Heard::Granted { account: ACCOUNT_CHAT, generation: 1 },
            Heard::Failed {
                account: ACCOUNT_CHAT,
                failure: child::Failure::Later { after: Some(Duration::from_secs(7)) },
            },
        ],
        Story::RefusedRefresh => vec![
            Heard::Granted { account: ACCOUNT_CHAT, generation: 1 },
            Heard::Failed { account: ACCOUNT_CHAT, failure: child::Failure::SignInAgain },
        ],
        Story::RejectedClient => vec![
            Heard::Granted { account: ACCOUNT_CHAT, generation: 1 },
            Heard::Failed { account: ACCOUNT_CHAT, failure: child::Failure::Misconfigured },
        ],
        Story::DamagedRecord => vec![Heard::Failed {
            account: ACCOUNT_CHAT,
            failure: child::Failure::Unloaded { at: child::Place::File, why: child::Unread::Unreadable },
        }],
        Story::Granted | Story::Closed | Story::ClosedExchange => {
            vec![Heard::Granted { account: ACCOUNT_CHAT, generation: 1 }]
        }
    }
}

#[must_use]
pub fn issuer_plans(story: crate::world::Story) -> Vec<fake::Plan> {
    use crate::world::Story;
    match story {
        Story::RateLimitedRefresh => vec![rate_limit(), rate_limit()],
        Story::RefusedRefresh => vec![refuse()],
        Story::RejectedClient => vec![client_rejected()],
        Story::ClosedExchange => {
            let mut plan = rotate();
            plan.delay = Duration::from_secs(1);
            vec![plan]
        }
        Story::Granted
        | Story::Refreshed
        | Story::Closed
        | Story::HandedIn { .. }
        | Story::HandedInRejected
        | Story::DamagedRecord => vec![rotate()],
    }
}
fn rate_limit() -> fake::Plan {
    fake::Plan {
        status: 429,
        body: fake::Body::Raw(bytes::copy_of(b"{\"error\":\"temporarily_unavailable\"}")),
        delay: Duration::from_millis(5),
        retry_after: Duration::from_secs(7),
    }
}
fn client_rejected() -> fake::Plan {
    fake::Plan {
        status: 401,
        body: fake::Body::Raw(bytes::copy_of(b"{\"error\":\"invalid_client\"}")),
        delay: Duration::from_millis(5),
        retry_after: Duration::ZERO,
    }
}
