use jig_protocol_worker_link::{Credential, Limits};
use skein_lib::{Duration, Secret};
pub const WORKER_A: &[u8] = b"worker-a";
pub const SECRET_A: &[u8] = b"secret-a";
pub const PAYLOAD: u32 = 3;
pub const SEED_ONE: u64 = 1;
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
    chunk: 64,
    handshake: Duration::from_secs(1),
    ping: Duration::from_secs(5),
    silence: Duration::from_secs(15),
    stall: Duration::from_secs(30),
    backoff: Duration::from_millis(100),
    ceiling: Duration::from_millis(1600),
    proved: Duration::from_secs(2),
    grace: Duration::from_secs(10),
    busy: Duration::from_millis(200),
};
#[must_use]
pub fn credential() -> Credential {
    Credential { name: WORKER_A.into(), secret: Secret::new(SECRET_A.into()), payload: PAYLOAD }
}

#[must_use]
pub fn limits() -> Limits {
    LIMITS
}
#[must_use]
pub fn channel_limits() -> skein_channel::Limits {
    jig_protocol_worker_link::channel_limits(&LIMITS).expect("small channel")
}
#[must_use]
pub fn schema() -> skein_channel::Schema {
    let codec = jig_protocol_worker_wire::sizes::limits(&SIZES).expect("small sizes");
    jig_protocol_worker_wire::schema(&codec).expect("small schema")
}
fn peer(schema: skein_channel::Schema) -> skein_fake_channel::ScriptedPeer {
    skein_fake_channel::ScriptedPeer::new(schema, skein_channel::Role::Responder, channel_limits(), 1, Box::new([]))
        .expect("scripted engine")
}
#[must_use]
pub fn engine_accepting() -> skein_fake_channel::ScriptedPeer {
    peer(schema())
}
#[must_use]
pub fn engine_refusing(reason: u16) -> skein_fake_channel::ScriptedPeer {
    let mut engine = engine_accepting();
    let frame =
        skein_channel::control_frame(&skein_channel::Control::Refuse { reason, text: Box::new([]) }, &channel_limits())
            .expect("refusal");
    engine.play(frame);
    engine
}
#[must_use]
pub fn engine_with_terms(kind: u16, largest: u32) -> skein_fake_channel::ScriptedPeer {
    let mut schema = schema();
    for index in 0..schema.versions.len() {
        let version = schema.versions.get_mut(index).expect("version index");
        for index in 0..version.kinds.len() {
            let entry = version.kinds.get_mut(index).expect("kind index");
            if entry.kind == kind {
                entry.largest = largest;
            }
        }
    }
    peer(schema)
}
