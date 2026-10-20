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
    program_sources: 2,
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
        kinds: u32::try_from(jig_protocol_worker_wire::KINDS.len()).expect("kinds fit"),
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
    peer(schema(), jig_protocol_worker_wire::VERSION, name, secret, payload)
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
pub fn answer_record() -> jig_protocol_worker_wire::Answer {
    use jig_protocol_worker_wire as wire;
    let codec = codec();
    let work = wire::Work::new(
        &codec,
        wire::WorkParts { left: Some(Box::from(b"left".as_slice())), saved: Some(Box::from(b"saved".as_slice())) },
    )
    .expect("work");
    let ended = wire::Ended::new(&codec, wire::EndedParts { outcome: Box::from(b"opaque-result".as_slice()), work })
        .expect("ended");
    let refused = refused_messages();
    wire::Answer::new(
        &codec,
        wire::AnswerParts {
            task: 7,
            attempt: 2,
            turns: 3,
            spent: 31,
            read: Some(9),
            refused,
            ending: wire::Ending::Ended(ended),
        },
    )
    .expect("answer")
}
#[must_use]
pub fn call_record() -> jig_protocol_worker_wire::Call {
    use jig_protocol_worker_wire as wire;
    let codec = codec();
    wire::Call::new(
        &codec,
        wire::CallParts {
            task: 7,
            attempt: 2,
            name: Box::from([0, 2, 7]),
            tool: Box::from(b"read".as_slice()),
            writes: false,
            input: Box::from(b"input".as_slice()),
            deadline: Duration::from_secs(4),
        },
    )
    .expect("call")
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
pub fn unauthorized_record() -> jig_protocol_worker_wire::Unauthorized {
    use jig_protocol_worker_wire as wire;
    let codec = codec();
    wire::Unauthorized::new(&codec, wire::UnauthorizedParts { task: 7, attempt: 2, account: 1, generation: 4 })
        .expect("unauthorized")
}
#[must_use]
pub fn limited_record() -> jig_protocol_worker_wire::Limited {
    use jig_protocol_worker_wire as wire;
    let codec = codec();
    wire::Limited::new(
        &codec,
        wire::LimitedParts { task: 7, attempt: 2, account: 1, reset: Some(Duration::from_secs(5)) },
    )
    .expect("limited")
}
#[must_use]
pub fn account_refused_record() -> jig_protocol_worker_wire::AccountRefused {
    use jig_protocol_worker_wire as wire;
    let codec = codec();
    wire::AccountRefused::new(
        &codec,
        wire::AccountRefusedParts {
            task: 7,
            attempt: 2,
            account: 1,
            code: b"forbidden".as_slice().into(),
            reason: b"suspended".as_slice().into(),
        },
    )
    .expect("account refused")
}

fn codec() -> jig_protocol_worker_wire::v3::Limits {
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
    let call = call_record();
    let answer = answer_record();
    let fact = fact_record();
    let unauthorized = unauthorized_record();
    let limited = limited_record();
    let refused = account_refused_record();
    vec![
        turn_frame(),
        encoded(0x0102, call.measure(), |writer| call.encode(writer)),
        encoded(0x0103, answer.measure(), |writer| answer.encode(writer)),
        encoded(0x0104, fact.measure(), |writer| fact.encode(writer)),
        encoded(0x0105, unauthorized.measure(), |writer| unauthorized.encode(writer)),
        encoded(0x0106, limited.measure(), |writer| limited.encode(writer)),
        encoded(0x0107, refused.measure(), |writer| refused.encode(writer)),
    ]
}

#[must_use]
pub fn turn_frame() -> skein_channel::Frame {
    let turn = turn_record();
    encoded(0x0101, turn.measure(), |writer| turn.encode(writer))
}

#[must_use]
pub fn answer_frame() -> skein_channel::Frame {
    let answer = answer_record();
    encoded(0x0103, answer.measure(), |writer| answer.encode(writer))
}

pub const TOKEN_ONE: &[u8] = b"TOKEN_ONE";
pub const TOKEN_TWO: &[u8] = b"TOKEN_TWO";
pub const ACCOUNT_ID: &[u8] = b"ID_ONE";
pub const IDENTITY: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
#[must_use]
pub fn grant(form: jig_domain_host::Form) -> jig_domain_host::Grant {
    jig_domain_host::Grant { form, account: 1, generation: 4, valid: Duration::from_secs(30) }
}
#[must_use]
pub fn values() -> Box<[Option<jig_protocol_engine_workers::GrantValue>]> {
    Box::new([
        Some(jig_protocol_engine_workers::GrantValue::Withdrawn {
            id: Secret::new(Box::from(ACCOUNT_ID)),
            bearer: Secret::new(Box::from(TOKEN_ONE)),
        }),
        Some(jig_protocol_engine_workers::GrantValue::Application { secret: Secret::new(Box::from(TOKEN_TWO)) }),
    ])
}
#[must_use]
pub fn credential_value(scheme: jig_protocol_engine_workers::Scheme) -> jig_protocol_engine_workers::GrantValue {
    jig_protocol_engine_workers::GrantValue::Credential { scheme, token: Secret::new(Box::from(TOKEN_ONE)) }
}
#[must_use]
pub fn parts() -> Box<[jig_protocol_worker_wire::Part]> {
    Box::new([jig_protocol_worker_wire::Part {
        token: skein_lib::Token::new(500),
        bytes: Box::from(b"items".as_slice()),
    }])
}
#[must_use]
pub fn assignment() -> jig_domain_host::Assignment {
    use jig_domain_host as host;
    use skein_lib::Token;
    host::Assignment {
        assignment: host::RunAssignment {
            run: Token::new(7),
            attempt: Token::new(2),
            identity: IDENTITY,
            workspace: Some(host::Workspace { workstream: 9, items: Token::new(500) }),
            save: true,
            charter: Box::from(b"charter".as_slice()),
            grants: Box::new([grant(host::Form::Withdrawn), grant(host::Form::Application)]),
        },
        turns: Box::new([Box::from([0, 127]), Box::from([255, 1, 2])]),
        answered: Box::new([host::AnsweredCall {
            name: Box::from([0, 2, 7]),
            tool: Box::from(b"read".as_slice()),
            answer: host::SettledAnswer::Host { error: true, body: Box::from(b"answer".as_slice()) },
        }]),
        messages: Box::new([host::StartMessage {
            name: Token::new(8),
            sender: Box::from(b"sender".as_slice()),
            words: Box::from(b"message".as_slice()),
        }]),
    }
}
#[must_use]
pub fn assignment_down(channel: skein_lib::Token) -> jig_protocol_engine_workers::Event {
    jig_protocol_engine_workers::Event::Down {
        channel,
        down: jig_domain_host::LinkDown::Assign { assignment: assignment() },
        parts: parts(),
        values: values(),
    }
}
