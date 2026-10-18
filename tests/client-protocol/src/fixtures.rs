//! One bounded core, person and pair of protocol sides for every story.
use crate::Protocol;
use jig_client_world::{Settings, Test, World};
use jig_protocol_engine_clients::{Limits, Served};
use skein_lib::{Duration, List, Secret, Time, Wall};
pub const SEED: u64 = 8510;
pub const FIRST_OWNER: u64 = 1;
pub const HOST: &str = "127.0.0.1:8080";
#[must_use]
pub fn wall(now: Time) -> Wall {
    Wall::from_nanos(1_000_000_000_000 + now.as_nanos())
}
#[must_use]
pub fn address() -> skein_io::Addr {
    skein_io::Addr::Ip("127.0.0.1:8080".parse().expect("loopback"))
}
#[must_use]
pub fn owner_limits() -> jig_test_domain::Limits {
    let mut limits = jig_core_world::world::limits();
    // Client keeps closing watches until their engine terminals arrive.
    limits.core.views.watchers = jig_client_world::test::limits().client.domain.watches;
    limits.core.documents.rows = 2;
    limits.core.documents.lines = 2;
    limits.core.documents.chips = 2;
    limits.core.documents.text_bytes = 32;
    limits
}
#[must_use]
pub fn journal(mut limits: jig_test_domain::Limits) -> jig_test_domain::Limits {
    let room = jig_domain_core::room_max(&limits.core).expect("whole core room");
    let connectors = jig_test_connector::MAX_OUT.checked_mul(2).expect("two connector routes");
    limits.journal.writes = room.writes.checked_add(connectors).expect("whole writes");
    limits.journal.held = room
        .held
        .checked_add(connectors)
        .expect("whole held")
        .checked_mul(limits.journal.commits)
        .expect("bounded flights");
    limits
}
#[must_use]
pub fn peers() -> jig_core_world::peers::Peers {
    jig_core_world::peers::Peers::new(
        vec![jig_fake_workers::Worker::new(1, 1, vec![Box::new([jig_fake_workers::Script::Wait])])],
        Vec::new(),
    )
}
#[must_use]
pub fn settings(rows: u32) -> jig_protocol_engine_clients::Settings {
    jig_protocol_engine_clients::Settings {
        link: Some(jig_domain_people::Identity {
            key: jig_domain_people::IdentityKey { provider: 0, subject: 7_u64.to_be_bytes().into() },
            login: b"login".as_slice().into(),
            name: b"Person".as_slice().into(),
        }),
        deployment_provider: 2,
        heartbeat: Duration::from_millis(50),
        rows,
        served: jig_client_world::test::list([
            Served { family: None, first: 8, last: 8 },
            Served { family: Some(1), first: 1, last: 1 },
        ]),
        providers: List::with_capacity(0),
        seed: Secret::new(Box::new([7; 32])),
        stream: 31,
    }
}
// Match this world's HTTP intake and the small profile's component bound.
pub const MESSAGE_BYTES: u32 = 65536;
#[must_use]
pub fn engine_limits() -> Limits {
    Limits {
        streams: 4,
        watches_per_stream: 8,
        queued_bytes: 65536,
        identity_bytes: 32,
        families: 2,
        providers: 1,
        mints: 4,
        exchanges: 8,
        exchanges_per_client: 4,
        message_bytes: MESSAGE_BYTES,
        wire: jig_client_protocol_wire::v8::CEILINGS,
        link_span: Duration::from_secs(300),
    }
}
#[must_use]
pub fn connections_limits() -> jig_client_shell_native::Limits {
    jig_client_shell_native::Limits {
        exchanges: 8,
        body_bytes: 65536,
        deadline: Duration::from_secs(1),
        http: skein_http::client::Limits { head: 4096, headers: 16, request: 4096, read: 1024, send: 1024 },
    }
}
#[must_use]
pub fn link_limits() -> jig_client_protocol_link::Limits {
    jig_client_protocol_link::Limits::new(domain_limits(), 8).expect("bounded link")
}
#[must_use]
pub fn sign_in_then(
    mut steps: Vec<skein_fake_person::Step<jig_client_test_domain::Address>>,
) -> Vec<skein_fake_person::Step<jig_client_test_domain::Address>> {
    use skein_fake_person::{Find, Step};
    let find = Find::named(skein_ui::Role::Button, b"Sign in with Test");
    let mut before = vec![Step::See { find: find.clone(), within: Duration::from_secs(1) }, Step::Press { find }];
    before.append(&mut steps);
    before
}
#[must_use]
pub fn world(seed: u64) -> World<Test, Protocol> {
    World::new(Settings { seed, ..Settings::default() }, Protocol::new(seed))
}

#[must_use]
pub fn family_world(seed: u64) -> World<Test, Protocol> {
    let mut world = self::world(seed);
    let configuration = jig_core_world::world::config(seed);
    let resource = configuration.first.resources.first().expect("configured first object").path.clone();
    world.front().adopt_family_resource(resource);
    world
}

#[must_use]
pub fn domain_limits() -> jig_client_domain::Limits {
    let profile = jig_client_world::test::limits();
    let jig = profile.jig;
    jig_client_domain::Limits {
        owners: &[jig_client_domain::Spoken { family: 1, version: jig_client_test_protocol_wire::VERSION }],
        requests: profile.client.domain.requests,
        ask_bytes: 16384,
        handles: 8,
        watches: jig.watches,
        reads: jig.reads,
        turn_bytes: 32,
        document_bytes: jig.document_bytes,
        backoff: jig_client_world::test::limits().client.domain.backoff,
        rows: jig.rows,
        segments: jig.segments,
        text: jig.text,
        projects: jig.projects,
        fields: jig.fields,
        delegates: jig.delegates,
        records: jig.records,
        kept: jig.kept,
        parts: jig.parts,
        authority: 8,
        heartbeat: jig_client_world::test::limits().client.domain.heartbeat,
    }
}

pub const OWNER_SUBJECT: [u8; 8] = 7_u64.to_be_bytes();
pub const START_KEY: [u8; 16] = [63; 16];
#[must_use]
pub fn starter() -> jig_fake_parties::Party {
    use jig_fake_parties::{Act, Ask, Party, Role};
    Party::new(
        u32::try_from(FIRST_OWNER).expect("scripted party identifier fits"),
        Role::Owner,
        1_000_000,
        Box::new([
            Act::SignIn { provider: 0, subject: OWNER_SUBJECT.as_slice().into() },
            Act::Request { key: START_KEY, ask: Ask::Chat { words: b"hello".as_slice().into() } },
        ]),
    )
}
mod create;
pub use create::{batch_config, create_tasks, proposed_tasks};
mod queries;
pub use queries::{
    FIRST_QUERY_TASK, SECOND_QUERY_TASK, person_batch_config, person_tasks, query_limits, task_lines, task_query,
    tracked_amendment, tracked_rows, tracked_tasks,
};

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
pub fn server() -> jig_test_service::server::Server {
    let mut limits = jig_test_service::limits::profile_limits(jig_protocol_config::Profile::Small).server;
    limits.connections = 16;
    limits.http = server_http();
    jig_test_service::server::Server::new(address(), &limits)
}

#[must_use]
pub fn http_limits() -> jig_protocol_engine_clients_http::Limits {
    jig_protocol_engine_clients_http::Limits {
        calls: 16,
        streams: 4,
        body: MESSAGE_BYTES,
        read: 1024,
        send: 1024,
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
    let env = skein_lib::Env { now: Time::ZERO, wall: Wall::EPOCH, limits: http_limits() };
    mount.down(
        &env,
        jig_protocol_engine_clients_http::Event::Listening { hosts },
        &mut skein_lib::Queue::with_capacity(1),
    );
    mount
}
#[must_use]
pub fn server_http() -> skein_http::server::Limits {
    skein_http::server::Limits { head: 4096, headers: 32, body: 65536, read: 1024, response: 65536, send: 1024 }
}
