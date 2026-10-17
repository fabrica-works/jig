pub use crate::Peer;
mod receiving;
use jig_domain_people::{Identity, IdentityKey};
use jig_protocol_engine_clients::{Limits, Served, Settings};
pub use receiving::signed_small_core;
use skein_lib::{Duration, List, Secret};
pub const FIRST_SIGN_IN: u64 = 41;
pub const FIRST_PERSON: u64 = 7;
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
    served.push(Served { family: None, first: 8, last: 8 }).expect("one range");
    Settings {
        deployment_provider: 2,
        listener: skein_io::Addr::Ip("127.0.0.1:8080".parse().expect("loopback")),
        link: Some(first_owner()),
        heartbeat: Duration::from_secs(5),
        rows: jig_protocol_config::profile(jig_protocol_config::Profile::Small).core.documents.rows,
        served,
        providers: List::with_capacity(0),
        seed: Secret::new(Box::new([7; 32])),
        stream: 31,
    }
}
#[must_use]
pub fn limits() -> Limits {
    Limits {
        connections: 8,
        streams: 2,
        watches_per_stream: 4,
        queued_bytes: 65536,
        identity_bytes: 1024,
        families: 32,
        providers: 32,
        exchanges: 4,
        exchanges_per_client: 2,
        http: skein_http::server::Limits { head: 4096, headers: 32, body: 1024, read: 256, response: 1024, send: 32 },
        wire: jig_client_protocol_wire::v8::CEILINGS,
        link_span: Duration::from_secs(300),
        stall: Duration::from_secs(30),
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
pub fn frame_subject() -> jig_client_protocol_wire::v8::Subject {
    jig_client_protocol_wire::v8::Subject::Frame
}
#[must_use]
pub fn frame_document(needs: u32) -> jig_domain_core::documents::Document {
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
        charters: List::with_capacity(0),
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
    skein_lib::base64::encode(skein_lib::base64::Alphabet::Url, answer.token().expose(), &mut out).expect("bearer");
    out.finish()
}
#[must_use]
pub fn window(rows: u32) -> jig_client_protocol_wire::v8::Window {
    use jig_client_protocol_wire::v8 as w;
    w::Window::new(&limits().wire, w::WindowParts { rows, after: None }).expect("bounded window")
}
#[must_use]
pub fn family_read(
    scope: jig_client_protocol_wire::v8::Scope,
    window: jig_client_protocol_wire::v8::Window,
) -> Box<[u8]> {
    use jig_client_protocol_wire::v8 as w;
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
