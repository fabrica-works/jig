use crate::{Component, Limits, Worker};
use alloc::boxed::Box;
use skein_io::digest::digest;
use skein_lib::{Duration, Secret, Writer};
pub const WORKER_A: &[u8] = b"worker-a";
pub const WORKER_B: &[u8] = b"worker-b";
pub const SECRET_A: &[u8] = b"secret-a";
pub const SECRET_B: &[u8] = b"secret-b";
pub const PAYLOAD: u32 = 3;
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
pub fn workers() -> Box<[Worker]> {
    Box::new([
        Worker { name: Box::from(WORKER_A), digest: digest(SECRET_A) },
        Worker { name: Box::from(WORKER_B), digest: digest(SECRET_B) },
    ])
}
pub fn component() -> Component {
    Component::new(&LIMITS, workers(), PAYLOAD).expect("small component")
}
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
