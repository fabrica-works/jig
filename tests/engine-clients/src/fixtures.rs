pub use crate::Peer;
mod receiving;
use jig_domain_people::{Identity, IdentityKey};
use jig_protocol_engine_clients::{Limits, Served, Settings};
pub use receiving::signed_small_core;
use skein_lib::{Duration, List, Secret};
pub const FIRST_SIGN_IN: u64 = 41;
pub const FIRST_PERSON: u64 = 7;
pub const APPLICATION_PROVIDER: u16 = 5;
pub const HOST: &str = "127.0.0.1:8080";
#[must_use]
pub fn first_owner() -> Identity {
    Identity {
        key: IdentityKey { provider: 0, subject: Box::from(b"owner".as_slice()) },
        login: Box::from(b"owner".as_slice()),
        name: Box::from(b"Owner".as_slice()),
    }
}
#[must_use]
pub fn settings() -> Settings {
    let mut served = List::with_capacity(1);
    served.push(Served { family: None, first: 9, last: 9 }).expect("one range");
    let mut providers = List::with_capacity(1);
    providers.push(APPLICATION_PROVIDER).expect("application provider");
    Settings {
        deployment_provider: 2,

        link: Some(first_owner()),
        heartbeat: Duration::from_secs(5),
        rows: jig_protocol_config::profile(jig_protocol_config::Profile::Small).core.documents.rows,
        served,
        providers,
        seed: Secret::new(Box::new([7; 32])),
        stream: 31,
    }
}
// Match this world's HTTP intake so it exercises the same component bound.
pub const MESSAGE_BYTES: u32 = 1024;
#[must_use]
pub fn limits() -> Limits {
    Limits {
        streams: 2,
        watches_per_stream: 4,
        queued_bytes: 65536,
        identity_bytes: 1024,
        families: 32,
        providers: 32,
        mints: 4,
        exchanges: 4,
        exchanges_per_client: 2,
        message_bytes: MESSAGE_BYTES,
        wire: jig_client_protocol_wire::v9::CEILINGS,
        link_span: Duration::from_secs(300),
    }
}
#[must_use]
pub fn client_limits() -> skein_http::client::Limits {
    skein_http::client::Limits { head: 4096, headers: 32, request: 4096, read: 256, send: 256 }
}
#[must_use]
pub fn expires() -> skein_lib::Wall {
    skein_lib::Wall::from_nanos(86_400_000_000_000)
}

#[must_use]
pub fn me_subject() -> jig_client_protocol_wire::v9::Subject {
    jig_client_protocol_wire::v9::Subject::Me
}
#[must_use]
pub fn me_document(needs: u32) -> jig_domain_core::documents::Document {
    use jig_domain_core::documents as d;
    d::Document::Frame(d::Frame {
        party: 7,
        kind: jig_domain_people::Kind::Person,
        login: d::Text { bytes: Box::new([]), cut: false },
        name: d::Text { bytes: Box::new([]), cut: false },
        deployment_role: None,
        projects: List::with_capacity(0),
        needs,
        need_counts: jig_domain_people::Needs { questions: 7, proposals: 7, escalations: 7, person_tasks: 7 },
        updates: d::Count::Known(7),
        results_seen: 7,
        retention: Duration::from_nanos(19),
        cut: false,
    })
}

#[must_use]
pub fn signed_in(peer: &mut Peer) -> Box<[u8]> {
    let link = peer.printed_link();
    let exchange = peer.post_link(&link);
    peer.answer_signed_in(FIRST_SIGN_IN, FIRST_PERSON);
    peer.answer_kept(FIRST_SIGN_IN);
    let answer = peer.sign_in_answer(exchange);
    let mut out = skein_lib::Writer::new(61);
    out.put(b"Bearer ").expect("prefix");
    skein_lib::base64::encode(
        skein_lib::base64::Alphabet::Url,
        answer.token().as_ref().expect("native token").expose(),
        &mut out,
    )
    .expect("bearer");
    out.finish()
}
#[must_use]
pub fn window(rows: u32) -> jig_client_protocol_wire::v9::Window {
    use jig_client_protocol_wire::v9 as w;
    w::Window::new(&limits().wire, w::WindowParts { rows, after: None }).expect("bounded window")
}
#[must_use]
pub fn family_read(
    scope: jig_client_protocol_wire::v9::Scope,
    window: jig_client_protocol_wire::v9::Window,
) -> Box<[u8]> {
    use jig_client_protocol_wire::v9 as w;
    let record = w::FamilyRecord::new(&limits().wire, w::FamilyRecordParts { scope, window, record: Box::new([255]) })
        .expect("family record");
    let mut bytes = skein_lib::Writer::new(usize::try_from(record.measure()).expect("record size"));
    record.encode(&mut bytes).expect("measured record");
    let owner = w::Owner::Family(w::Family::new(&limits().wire, w::FamilyParts { number: 1 }).expect("family"));
    let record =
        w::Owned::new(&limits().wire, w::OwnedParts { owner, version: 1, record: bytes.finish() }).expect("owned");
    let mut bytes = skein_lib::Writer::new(usize::try_from(record.measure()).expect("record size"));
    record.encode(&mut bytes).expect("measured record");
    bytes.finish()
}

#[must_use]
pub fn application_person() -> Identity {
    Identity {
        key: IdentityKey { provider: APPLICATION_PROVIDER, subject: Box::from(b"person".as_slice()) },
        login: Box::from(b"person".as_slice()),
        name: Box::from(b"Person".as_slice()),
    }
}

#[must_use]
pub fn printed(secret: &skein_lib::Secret) -> Box<[u8]> {
    const PREFIX: &[u8] = b"http://127.0.0.1:8080/jig/v1/sign-in/link?secret=";
    let encoded = skein_lib::base64::encoded_len(
        skein_lib::base64::Alphabet::Url,
        u32::try_from(secret.expose().len()).expect("short secret"),
    )
    .expect("encoded secret");
    let mut bytes = skein_lib::Writer::new(PREFIX.len() + usize::try_from(encoded).expect("u32 fits"));
    bytes.put(PREFIX).expect("prefix fits");
    skein_lib::base64::encode(skein_lib::base64::Alphabet::Url, secret.expose(), &mut bytes).expect("secret fits");
    bytes.finish()
}

#[must_use]
pub fn address() -> skein_io::Addr {
    skein_io::Addr::Ip("127.0.0.1:8080".parse().expect("loopback"))
}
#[must_use]
pub fn server_limits() -> jig_test_service::server::Limits {
    let mut limits = jig_test_service::limits::profile_limits(jig_protocol_config::Profile::Small).server;
    limits.connections = 8;
    limits.http = server_http();
    limits
}
#[must_use]
pub fn server() -> jig_test_service::server::Server {
    jig_test_service::server::Server::new(address(), &server_limits())
}

#[must_use]
pub fn http_limits() -> jig_protocol_engine_clients_http::Limits {
    jig_protocol_engine_clients_http::Limits {
        calls: 16,
        streams: 2,
        body: MESSAGE_BYTES,
        read: 256,
        send: 32,
        event: 87416,
    }
}
#[must_use]
pub fn mount() -> jig_protocol_engine_clients_http::Mount {
    let mut mount = jig_protocol_engine_clients_http::Mount::new(
        jig_protocol_engine_clients_http::Settings { prefix: Box::from(b"/jig".as_slice()), loopback: true },
        &http_limits(),
    );
    let mut hosts = skein_lib::List::with_capacity(1);
    hosts.push(Box::from(HOST.as_bytes())).expect("one host");
    let env = skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits: http_limits() };
    mount.down(
        &env,
        jig_protocol_engine_clients_http::Event::Listening { hosts },
        &mut skein_lib::Queue::with_capacity(1),
    );
    mount
}
#[must_use]
pub fn server_http() -> skein_http::server::Limits {
    skein_http::server::Limits { head: 4096, headers: 32, body: 1024, read: 256, response: 65536, send: 32 }
}
