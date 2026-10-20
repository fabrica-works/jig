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
/// Every token the issuer gives and the record it starts from: the access
/// tokens and the refresh tokens.
pub const VALUES: [&[u8]; 4] = [TOKEN_FIRST, TOKEN_SECOND, b"seed", b"next"];
pub const AUTHORIZE: &[u8] = b"http://127.0.0.1:31000/authorize";
pub const TOKEN: &[u8] = b"http://127.0.0.1:31000/token";
pub const REDIRECT: &[u8] = b"http://localhost:31234/callback";
/// The deployment's identifier in the header the owner commits at an empty
/// store.
pub const DEPLOYMENT: [u8; 16] = [18; 16];
/// The store's application bounds: this world keeps no application rows.
pub const APPLICATION: jig_protocol_store::sizes::Application =
    jig_protocol_store::sizes::Application { version: 1, key: 16, value: 64 };
/// The small profile's store: its memory maps grow only as rows arrive.
#[must_use]
pub fn store_limits() -> jig_protocol_store::Limits {
    jig_protocol_config::profile(jig_protocol_config::Profile::Small).store
}
#[must_use]
pub fn limits() -> accounts::Limits {
    accounts::Limits {
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
        client: oauth::ClientLimits {
            document: oauth::Limits {
                document_bytes: 8192,
                string_bytes: 2048,
                token_bytes: 256,
                client_bytes: 64,
                detail_bytes: 64,
                record_bytes: 8192,
                issued_client_bytes: 256,
                granted_bytes: 64,
                subject_bytes: 256,
                id_token_bytes: 2048,
                depth: 8,
                tokens: 64,
            },
            uri_bytes: 256,
            scope_bytes: 64,
            state_bytes: 64,
            code_bytes: 64,
            url_bytes: 1024,
            request_bytes: 1024,
            parameter_bytes: jig_protocol_accounts::installation::PARAMETER_BYTES + 15 + 256,
            resource_bytes: 256,
            nonce_bytes: 32,
            metadata_bytes: 8192,
            key_set_bytes: 8192,
            keys: 2,
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
        holder_accounts: 2,
        program_names: 12,
        program_hosts: 65,
        sources: 3,
        sign_ins: 1,
        credentials: 0,
        backoff_base: Duration::from_secs(2),
        backoff_max: Duration::from_secs(60),
        retry: Duration::from_secs(30),
        spent_attention: Duration::from_secs(300),
        facts: 16,
    }
}
/// The registration, its terms the installation's identifier's.
#[must_use]
pub fn registration(terms: oauth::Terms) -> oauth::Registration {
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
        terms: Some(terms),
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

/// The record the store holds when a story starts: the kept record for a
/// sign-in, bytes that decode as no record when it is damaged, and none
/// before a handed-in record is first kept.
#[must_use]
pub fn held(story: crate::world::Story) -> Option<Box<[u8]>> {
    use crate::world::Story;
    match story {
        Story::DamagedRecord => Some(bytes::copy_of(b"damaged record")),
        Story::RefusedGrant => {
            let mut record = kept();
            record.expires_at = Wall::EPOCH;
            Some(oauth::encode_record(&record, &limits().client.document).expect("expired record"))
        }
        Story::NoRecord
        | Story::HandedIn { .. }
        | Story::HandedInRejected
        | Story::SignIn
        | Story::AccountSignInMissingScope
        | Story::AccountSignInVaryingPort
        | Story::AccountSignOut { .. }
        | Story::AccountSignInFull => None,
        Story::Granted
        | Story::Refreshed
        | Story::Closed
        | Story::RateLimitedRefresh
        | Story::RefusedRefresh
        | Story::RejectedClient
        | Story::ClosedExchange => {
            Some(oauth::encode_record(&kept(), &limits().client.document).expect("kept fixture"))
        }
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
        | crate::world::Story::RefusedGrant
        | crate::world::Story::NoRecord
        | crate::world::Story::RejectedClient
        | crate::world::Story::DamagedRecord
        | crate::world::Story::ClosedExchange
        | crate::world::Story::SignIn
        | crate::world::Story::AccountSignInMissingScope
        | crate::world::Story::AccountSignInVaryingPort
        | crate::world::Story::AccountSignOut { .. }
        | crate::world::Story::AccountSignInFull => None,
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
        // The visit's answer closes a sign-in's story.
        crate::world::Story::SignIn
        | crate::world::Story::AccountSignInMissingScope
        | crate::world::Story::AccountSignInVaryingPort
        | crate::world::Story::AccountSignOut { .. }
        | crate::world::Story::AccountSignInFull => 0,
        crate::world::Story::Granted
        | crate::world::Story::Closed
        | crate::world::Story::DamagedRecord
        | crate::world::Story::RefusedGrant
        | crate::world::Story::NoRecord => 1,
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
        // Account sign-out expires the held grant before revoking the record.
        Story::RefusedRefresh | Story::AccountSignOut { .. } => vec![
            Heard::Granted { account: ACCOUNT_CHAT, generation: 1 },
            Heard::Failed { account: ACCOUNT_CHAT, failure: child::Failure::SignInAgain },
        ],
        Story::RejectedClient => vec![
            Heard::Granted { account: ACCOUNT_CHAT, generation: 1 },
            Heard::Failed { account: ACCOUNT_CHAT, failure: child::Failure::Misconfigured },
        ],
        Story::RefusedGrant | Story::NoRecord => {
            vec![Heard::Failed { account: ACCOUNT_CHAT, failure: child::Failure::SignInAgain }]
        }
        Story::DamagedRecord => vec![Heard::Failed {
            account: ACCOUNT_CHAT,
            failure: child::Failure::Unloaded { at: child::Place::File, why: child::Unread::Unreadable },
        }],
        Story::Granted | Story::Closed | Story::ClosedExchange | Story::SignIn | Story::AccountSignInVaryingPort => {
            vec![Heard::Granted { account: ACCOUNT_CHAT, generation: 1 }]
        }
        Story::AccountSignInMissingScope | Story::AccountSignInFull => vec![],
    }
}

#[must_use]
pub fn issuer_plans(story: crate::world::Story) -> Vec<fake::Plan> {
    use crate::world::Story;
    match story {
        Story::RateLimitedRefresh => vec![rate_limit(), rate_limit()],
        Story::RefusedRefresh | Story::RefusedGrant => vec![refuse()],
        Story::NoRecord => vec![],
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
        Story::SignIn
        | Story::AccountSignInMissingScope
        | Story::AccountSignInVaryingPort
        | Story::AccountSignOut { .. }
        | Story::AccountSignInFull => account_sign_in_plans(story),
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

pub const PARTY_REDIRECT: &[u8] = b"http://127.0.0.1:31234/callback";
#[must_use]
pub fn account_sign_in(story: crate::world::Story) -> bool {
    matches!(
        story,
        crate::world::Story::SignIn
            | crate::world::Story::AccountSignInMissingScope
            | crate::world::Story::AccountSignInVaryingPort
            | crate::world::Story::AccountSignOut { .. }
            | crate::world::Story::AccountSignInFull
    )
}
#[must_use]
pub fn party_registration(story: crate::world::Story) -> jig_protocol_accounts::party::Registration {
    jig_protocol_accounts::party::Registration {
        issuer: bytes::copy_of(b"http://127.0.0.1:31000"),
        client: bytes::copy_of(b"client"),
        scopes: Box::new([bytes::copy_of(b"openid"), bytes::copy_of(b"read")]),
        name_hint: Some(bytes::copy_of(b"jig-test")),
        authorization: bytes::copy_of(AUTHORIZE),
        token: bytes::copy_of(TOKEN),
        revocation: Some(bytes::copy_of(b"http://127.0.0.1:31000/revoke")),
        keys: bytes::copy_of(b"http://127.0.0.1:31000/keys"),
        endpoint_transport: accounts::Transport::Plaintext,
        must_grant: Box::new([bytes::copy_of(b"openid"), bytes::copy_of(b"read")]),
        resource: Some(bytes::copy_of(b"api")),
        redirect_path: bytes::copy_of(b"callback"),
        redirect_port: (story != crate::world::Story::AccountSignInVaryingPort).then_some(31234),
    }
}
#[must_use]
pub fn issuer_terms(story: crate::world::Story) -> Option<fake::Terms> {
    account_sign_in(story).then(|| fake::Terms {
        issued: Box::new([bytes::copy_of(b"issued-client")]),
        omit_issued: false,
        varying_port: true,
        revocation_endpoint: Some(bytes::copy_of(b"http://127.0.0.1:31000/revoke")),
        openid: Some(fake::OpenId {
            issuer: bytes::copy_of(b"http://127.0.0.1:31000"),
            key_set_url: bytes::copy_of(b"http://127.0.0.1:31000/keys"),
            subject: bytes::copy_of(b"subject"),
            lifetime: Duration::from_secs(300),
            header_kid: true,
            fault: None,
        }),
    })
}
fn account_sign_in_plans(story: crate::world::Story) -> Vec<fake::Plan> {
    let mut first = rotate();
    if let fake::Body::Token(token) = &mut first.body {
        token.scope = Some(bytes::copy_of(if story == crate::world::Story::AccountSignInMissingScope {
            b"openid"
        } else {
            b"openid read"
        }));
    }
    let revoke = fake::Plan {
        status: if story == (crate::world::Story::AccountSignOut { confirmed: false }) { 400 } else { 200 },
        body: fake::Body::Raw(bytes::copy_of(b"{}")),
        delay: Duration::ZERO,
        retry_after: Duration::ZERO,
    };
    vec![first, revoke]
}
