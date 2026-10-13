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

#[must_use]
pub fn hello_record() -> jig_protocol_worker_wire::Hello {
    use jig_protocol_worker_wire as wire;
    use skein_lib::List;
    let codec = codec();
    let mut models = List::with_capacity(1);
    models
        .push(
            wire::ModelBound::new(
                &codec,
                wire::ModelBoundParts { name: Box::from(b"model".as_slice()), window: 8192, output: 1024 },
            )
            .expect("model"),
        )
        .expect("one model");
    let bounds = wire::Bounds::new(
        &codec,
        wire::BoundsParts {
            models,
            turns: 8,
            spend: 100,
            time: Duration::from_secs(60),
            waiting: Duration::from_secs(10),
            checks: Duration::from_secs(2),
            message_bytes: 16,
            inbox: 1,
            unacknowledged: 1,
        },
    )
    .expect("bounds");
    let mut hosting = List::with_capacity(1);
    hosting
        .push(
            wire::Hosting::new(&codec, wire::HostingParts { task: 7, attempt: 2, phase: wire::Phase::Waiting })
                .expect("hosting"),
        )
        .expect("one hosted run");
    let mut workstreams = List::with_capacity(1);
    workstreams.push(9).expect("one workstream");
    wire::Hello::new(
        &codec,
        wire::HelloParts { slots: 1, stop_bound: Duration::from_secs(20), workstreams, hosting, bounds },
    )
    .expect("hello")
}

#[must_use]
pub fn frame(kind: u16, bytes: &[u8]) -> skein_channel::Frame {
    let mut writer = skein_channel::frame_writer(kind, u32::try_from(bytes.len()).expect("small body")).expect("frame");
    writer.put(bytes).expect("measured frame");
    writer.finish().expect("complete frame")
}
#[must_use]
pub fn hello_frame() -> skein_channel::Frame {
    let hello = hello_record();
    let mut writer = Writer::new(usize::try_from(hello.measure()).expect("small hello"));
    hello.encode(&mut writer).expect("measured hello");
    frame(0x0100, &writer.finish())
}
#[must_use]
pub fn reporting_worker(name: &[u8], secret: &[u8]) -> skein_fake_channel::ScriptedPeer {
    let mut peer = worker(name, secret, PAYLOAD);
    peer.play(hello_frame());
    peer
}

#[must_use]
pub fn turn_record() -> jig_protocol_worker_wire::Turn {
    use jig_protocol_worker_wire as wire;
    let codec = codec();
    let refused = refused_messages();
    wire::Turn::new(
        &codec,
        wire::TurnParts {
            task: 7,
            attempt: 2,
            number: 3,
            spent: 29,
            read: Some(9),
            refused,
            opens_window: true,
            body: Box::from(b"opaque-turn".as_slice()),
        },
    )
    .expect("turn")
}
#[must_use]
pub fn fact_record() -> jig_protocol_worker_wire::Fact {
    use jig_protocol_worker_wire as wire;
    let codec = codec();
    let count = wire::Count::new(
        &codec,
        wire::CountParts { kind: wire::CountKind::LlmStarted, elapsed: Duration::from_secs(2), value: 3 },
    )
    .expect("count");
    wire::Fact::new(&codec, wire::FactParts { task: 7, attempt: 2, activity: wire::Activity::Count(count) })
        .expect("fact")
}
#[must_use]
pub fn rejected_record() -> jig_protocol_worker_wire::Rejected {
    use jig_protocol_worker_wire as wire;
    let codec = codec();
    wire::Rejected::new(&codec, wire::RejectedParts { task: 7, attempt: 2, account: 1, generation: 4 })
        .expect("rejected")
}
#[must_use]
pub fn exhausted_record() -> jig_protocol_worker_wire::Exhausted {
    use jig_protocol_worker_wire as wire;
    let codec = codec();
    wire::Exhausted::new(
        &codec,
        wire::ExhaustedParts { task: 7, attempt: 2, account: 1, retry_after: Duration::from_secs(5) },
    )
    .expect("exhausted")
}

fn codec() -> jig_protocol_worker_wire::v1::Limits {
    jig_protocol_worker_wire::sizes::limits(&SIZES).expect("small codec")
}
fn refused_messages() -> skein_lib::List<jig_protocol_worker_wire::Refused> {
    use jig_protocol_worker_wire as wire;
    let mut refused = skein_lib::List::with_capacity(1);
    refused
        .push(
            wire::Refused::new(&codec(), wire::RefusedParts { name: 8, reason: wire::MessageRefusal::Full })
                .expect("refusal"),
        )
        .expect("one refusal");
    refused
}

fn encoded(
    kind: u16,
    measure: u32,
    encode: impl FnOnce(&mut Writer) -> Result<(), skein_lib::Overflow>,
) -> skein_channel::Frame {
    let mut writer = Writer::new(usize::try_from(measure).expect("small record"));
    encode(&mut writer).expect("measured record");
    frame(kind, &writer.finish())
}
#[must_use]
pub fn up_frames() -> Vec<skein_channel::Frame> {
    let fact = fact_record();
    let rejected = rejected_record();
    let exhausted = exhausted_record();
    vec![
        turn_frame(),
        encoded(0x0104, fact.measure(), |writer| fact.encode(writer)),
        encoded(0x0105, rejected.measure(), |writer| rejected.encode(writer)),
        encoded(0x0106, exhausted.measure(), |writer| exhausted.encode(writer)),
    ]
}

#[must_use]
pub fn turn_frame() -> skein_channel::Frame {
    let turn = turn_record();
    encoded(0x0101, turn.measure(), |writer| turn.encode(writer))
}
