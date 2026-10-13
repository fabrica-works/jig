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
pub fn rejected_record() -> jig_protocol_worker_wire::Rejected {
    use jig_protocol_worker_wire as wire;
    let codec = codec();
    wire::Rejected::new(&codec, wire::RejectedParts { task: 7, attempt: 2, account: 1, generation: 4 })
        .expect("rejected")
}
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

pub fn answer_with_resource() -> jig_protocol_worker_wire::Answer {
    use jig_protocol_worker_wire as wire;
    let mut fields = answer_record().into_parts();
    let wire::Ending::Ended(ended) = fields.ending else {
        panic!("the fixture ended");
    };
    let codec = codec();
    let resource =
        wire::Resource::new(&codec, wire::ResourceParts { resource: Some(Box::from(b"resource".as_slice())) })
            .expect("resource");
    let preparation = wire::PreparationValue::new(
        &codec,
        wire::PreparationValueParts { value: wire::Preparation::Permanent(resource) },
    )
    .expect("preparation");
    let failed = wire::Failed::new(
        &codec,
        wire::FailedParts {
            failure: wire::Failure::Unprepared(preparation),
            detail: Box::from(b"detail".as_slice()),
            work: ended.into_parts().work,
        },
    )
    .expect("failed");
    fields.ending = wire::Ending::Failed(failed);
    wire::Answer::new(&codec, fields).expect("answer")
}
