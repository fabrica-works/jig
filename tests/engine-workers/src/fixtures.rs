use jig_protocol_engine_workers::{Limits, Worker};
use skein_io::digest::digest;
use skein_lib::{Duration, Secret, Writer};
pub const WORKER_A: &[u8] = b"worker-a";
pub const WORKER_B: &[u8] = b"worker-b";
pub const WORKER_C: &[u8] = b"worker-c";
pub const SECRET_C: &[u8] = b"secret-c";
pub const SECRET_A: &[u8] = b"secret-a";
pub const SECRET_B: &[u8] = b"secret-b";
pub const PAYLOAD: u32 = 3;
pub const OTHER_PAYLOAD: u32 = 4;
pub const SIZES: jig_protocol_worker_wire::sizes::Sizes = jig_protocol_worker_wire::sizes::Sizes {
    charter: 16,
    transcript: 16,
    turn: 16,
    message: 16,
    sender: 16,
    call_name: 16,
    tool_name: 16,
    call_input: 16,
    answer: 16,
    result: 16,
    detail: 16,
    grant: 16,
    account_id: 16,
    name: 16,
    secret: 16,
    model_name: 16,
    items: 16,
    left: 16,
    saved: 16,
    evidence: 16,
    resource: 16,
    slots: 1,
    calls: 1,
    turns: 1,
    inbox: 1,
    answered: 1,
    models: 1,
    workstreams: 1,
    accounts: 1,
    credentials: 1,
    facts: 16,
};

pub const LIMITS: Limits = Limits {
    sizes: SIZES,
    names: 3,
    workers: 2,
    opening: 1,
    chunk: 64,
    handshake: Duration::from_secs(1),
    hello: Duration::from_secs(1),
    ping: Duration::from_secs(5),
    silence: Duration::from_secs(15),
    stall: Duration::from_secs(30),
};
#[must_use]
pub fn workers() -> Box<[Worker]> {
    Box::new([
        Worker { name: Box::from(WORKER_A), digest: digest(SECRET_A) },
        Worker { name: Box::from(WORKER_B), digest: digest(SECRET_B) },
        Worker { name: Box::from(WORKER_C), digest: digest(SECRET_C) },
    ])
}

#[must_use]
pub fn credential(name: &[u8], secret: &[u8], payload: u32) -> Box<[u8]> {
    let limits = jig_protocol_worker_wire::sizes::limits(&SIZES).expect("small codec limits");
    let value = jig_protocol_worker_wire::Credential::new(
        &limits,
        jig_protocol_worker_wire::CredentialParts {
            name: Box::from(name),
            secret: Secret::new(Box::from(secret)),
            payload,
        },
    )
    .expect("small credential");
    let mut writer = Writer::new(usize::try_from(value.measure()).expect("small credential"));
    value.encode(&mut writer).expect("measured credential");
    writer.finish()
}

#[must_use]
pub fn limits() -> Limits {
    LIMITS
}
#[must_use]
pub fn channel_limits() -> skein_channel::Limits {
    let codec = jig_protocol_worker_wire::sizes::limits(&SIZES).expect("small codec");
    let cap = jig_protocol_worker_wire::sizes::worker_cap(&SIZES).expect("small cap").cap;
    skein_channel::Limits {
        chunk: 64,
        credential: u32::try_from(
            jig_protocol_worker_wire::Credential::worst_case_bytes(&codec).expect("small credential"),
        )
        .expect("fits"),
        skip: 64,
        output_bytes: cap.bytes,
        output_frames: cap.frames,
        kinds: 15,
    }
}
#[must_use]
pub fn schema() -> skein_channel::Schema {
    jig_protocol_worker_wire::schema(&jig_protocol_worker_wire::sizes::limits(&SIZES).expect("small codec"))
        .expect("small schema")
}
#[must_use]
pub fn peer(
    schema: skein_channel::Schema,
    version: u16,
    name: &[u8],
    secret: &[u8],
    payload: u32,
) -> skein_fake_channel::ScriptedPeer {
    skein_fake_channel::ScriptedPeer::new(
        schema,
        skein_channel::Role::Initiator,
        channel_limits(),
        version,
        credential(name, secret, payload),
    )
    .expect("scripted worker")
}
#[must_use]
pub fn worker(name: &[u8], secret: &[u8], payload: u32) -> skein_fake_channel::ScriptedPeer {
    peer(schema(), 1, name, secret, payload)
}
