pub use crate::Peer;
use jig_domain_people::{Identity, IdentityKey};
use jig_protocol_engine_clients::{Limits, Served, Settings};
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
    served.push(Served { family: None, first: 4, last: 4 }).expect("one range");
    Settings {
        listener: skein_io::Addr::Ip("127.0.0.1:8080".parse().expect("loopback")),
        link: Some(first_owner()),
        heartbeat: Duration::from_secs(5),
        served,
        providers: List::with_capacity(0),
        seed: Secret::new(Box::new([7; 32])),
        stream: 31,
    }
}
#[must_use]
pub fn limits() -> Limits {
    Limits {
        connections: 4,
        identity_bytes: 1024,
        families: 32,
        providers: 32,
        exchanges: 4,
        exchanges_per_client: 2,
        http: skein_http::server::Limits { head: 4096, headers: 32, body: 1024, read: 256, response: 1024, send: 256 },
        wire: jig_protocol_client_wire::v5::CEILINGS,
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
