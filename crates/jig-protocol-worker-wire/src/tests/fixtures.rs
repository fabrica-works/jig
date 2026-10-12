use crate::{
    Bounds, BoundsParts, Credential, CredentialParts, Hello, HelloParts, Hosting, HostingParts, ModelBound,
    ModelBoundParts, Phase, v1,
};
use alloc::boxed::Box;
use skein_lib::{Duration, List, Secret, Writer};

pub const WORKER_A: &[u8] = b"worker-a";
pub const WORKER_B: &[u8] = b"worker-b";
pub const MODEL_SMALL: &[u8] = b"small";

pub const LIMITS: v1::Limits = v1::Limits {
    credential_name: 16,
    credential_secret: 32,
    model_bound_name: 16,
    bounds_models: 2,
    hello_workstreams: 2,
    hello_hosting: 2,
    turn_refused: 2,
    turn_body: 32,
    call_name: 16,
    call_tool: 16,
    call_input: 32,
    work_left_some: 16,
    work_saved_some: 16,
    ended_outcome: 32,
    resource_resource_some: 16,
    failed_detail: 16,
    answer_refused: 2,
};

pub fn encode(hello: &Hello) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(hello.measure()).expect("small hello"));
    hello.encode(&mut writer).expect("measured room");
    writer.finish()
}

pub fn credential(name: &[u8], secret: &[u8]) -> Credential {
    Credential::new(&LIMITS, CredentialParts { name: name.into(), secret: Secret::new(secret.into()), payload: 1 })
        .expect("small credential")
}

pub fn model() -> ModelBound {
    ModelBound::new(&LIMITS, ModelBoundParts { name: MODEL_SMALL.into(), window: 8192, output: 1024 })
        .expect("small model")
}

pub fn hello_fullest() -> Hello {
    let mut models = List::with_capacity(2);
    models.push(model()).expect("first model slot");
    models.push(model()).expect("second model slot");
    let bounds = Bounds::new(
        &LIMITS,
        BoundsParts {
            models,
            turns: 40,
            spend: 100,
            time: Duration::from_secs(300),
            waiting: Duration::from_secs(60),
            checks: Duration::from_secs(20),
            message_bytes: 32,
            inbox: 2,
            unacknowledged: 2,
        },
    )
    .expect("small agent bounds");
    let mut workstreams = List::with_capacity(2);
    workstreams.push(7).expect("first workstream slot");
    workstreams.push(8).expect("second workstream slot");
    let mut hosting = List::with_capacity(2);
    for phase in [Phase::Active, Phase::Answered] {
        hosting
            .push(Hosting::new(&LIMITS, HostingParts { task: 3, attempt: 4, phase }).expect("hosting record"))
            .expect("two hosting slots");
    }
    Hello::new(&LIMITS, HelloParts { slots: 2, stop_bound: Duration::from_secs(95), workstreams, hosting, bounds })
        .expect("full hello within small list limits")
}

pub fn hello_smallest() -> Hello {
    // Three empty counts, plus the hello's and agent bounds' fixed fields.
    Hello::decode(&LIMITS, &mut skein_lib::Reader::new(&[0; 72])).expect("smallest hello")
}

pub const MESSAGE_FIRST: u64 = 41;
pub const MESSAGE_SECOND: u64 = 42;

pub fn refusal(name: u64, reason: crate::MessageRefusal) -> crate::Refused {
    crate::Refused::new(&LIMITS, crate::RefusedParts { name, reason }).expect("named refusal")
}

fn refusals() -> List<crate::Refused> {
    let mut refused = List::with_capacity(2);
    refused.push(refusal(MESSAGE_FIRST, crate::MessageRefusal::TooLarge)).expect("first refusal slot");
    refused.push(refusal(MESSAGE_SECOND, crate::MessageRefusal::Ending)).expect("second refusal slot");
    refused
}

pub fn turn_with_refusals() -> crate::Turn {
    crate::Turn::new(
        &LIMITS,
        crate::TurnParts {
            task: 3,
            attempt: 4,
            number: 7,
            spent: 100,
            read: Some(MESSAGE_FIRST),
            opens_window: true,
            refused: refusals(),
            body: b"opaque-turn".as_slice().into(),
        },
    )
    .expect("turn within small limits")
}

fn work() -> crate::Work {
    crate::Work::new(
        &LIMITS,
        crate::WorkParts { left: Some(b"left-work".as_slice().into()), saved: Some(b"saved-work".as_slice().into()) },
    )
    .expect("application's opaque work")
}

fn answer(ending: crate::Ending) -> crate::Answer {
    crate::Answer::new(
        &LIMITS,
        crate::AnswerParts {
            task: 3,
            attempt: 4,
            turns: 7,
            spent: 100,
            read: Some(MESSAGE_FIRST),
            refused: refusals(),
            ending,
        },
    )
    .expect("answer within small limits")
}

pub fn answer_ended() -> crate::Answer {
    let ended =
        crate::Ended::new(&LIMITS, crate::EndedParts { outcome: b"opaque-result".as_slice().into(), work: work() })
            .expect("ended result");
    answer(crate::Ending::Ended(ended))
}

pub fn answer_failed() -> crate::Answer {
    let run = crate::RunFailureValue::new(&LIMITS, crate::RunFailureValueParts { value: crate::RunFailure::Model })
        .expect("typed run failure");
    let failed = crate::Failed::new(
        &LIMITS,
        crate::FailedParts {
            failure: crate::Failure::Run(run),
            detail: b"bounded-detail".as_slice().into(),
            work: work(),
        },
    )
    .expect("failed answer");
    answer(crate::Ending::Failed(failed))
}

fn fact(activity: crate::Activity) -> crate::Fact {
    crate::Fact::new(&LIMITS, crate::FactParts { task: 3, attempt: 4, activity }).expect("typed activity")
}

pub fn fact_count() -> crate::Fact {
    let count = crate::Count::new(
        &LIMITS,
        crate::CountParts { kind: crate::CountKind::LlmRetried, elapsed: Duration::from_secs(2), value: 3 },
    )
    .expect("count activity");
    fact(crate::Activity::Count(count))
}

pub fn fact_long() -> crate::Fact {
    let long = crate::Long::new(
        &LIMITS,
        crate::LongParts { operation: crate::Operation::Checks, span: Duration::from_secs(6) },
    )
    .expect("long activity");
    fact(crate::Activity::Long(long))
}
