use crate::{
    Bounds, BoundsParts, Credential, CredentialParts, Hello, HelloParts, Hosting, HostingParts, ModelBound,
    ModelBoundParts, Phase, v3,
};
use alloc::boxed::Box;
use skein_lib::{Duration, List, Secret, Writer};

pub const WORKER_A: &[u8] = b"worker-a";
pub const WORKER_B: &[u8] = b"worker-b";
pub const MODEL_SMALL: &[u8] = b"small";

pub const LIMITS: v3::Limits = v3::Limits {
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
    invalid_start_why: 16,
    answer_refused: 2,
    account_refused_code: 16,
    account_refused_reason: 16,
    message_sender: 16,
    message_words: 32,
    workspace_items: 32,
    host_answer_body: 32,
    delivery_answer_evidence: 32,
    answered_call_name: 16,
    answered_call_tool: 16,
    grant_value: 32,
    grant_account_id: 16,
    assignment_charter: 32,
    assignment_transcript: 32,
    assignment_answered: 2,
    assignment_messages: 2,
    assignment_grants: 2,
    inbound_sender: 16,
    inbound_words: 32,
    relay_text_text: 32,
    relayed_name: 16,
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

pub const IDENTITY_FIRST: [u8; 16] = [7; 16];
pub const TOKEN_ONE: &[u8] = b"token-one";
pub const CREDENTIAL_ONE: &[u8] = b"credential-one";
pub const ACCOUNT_ONE: &[u8] = b"account-one";

pub fn grant(table: crate::GrantTable, account: u32, generation: u64) -> crate::Grant {
    let (value, account_id) = match table {
        crate::GrantTable::Accounts => (TOKEN_ONE, ACCOUNT_ONE),
        crate::GrantTable::Application => (CREDENTIAL_ONE, b"".as_slice()),
    };
    crate::Grant::new(
        &LIMITS,
        crate::GrantParts {
            table,
            account,
            generation,
            valid: Duration::from_secs(60),
            value: Secret::new(value.into()),
            account_id: Secret::new(account_id.into()),
        },
    )
    .expect("grant within small limits")
}

fn start_message(name: u64) -> crate::Message {
    crate::Message::new(
        &LIMITS,
        crate::MessageParts { name, sender: b"person".as_slice().into(), words: b"message".as_slice().into() },
    )
    .expect("start message")
}

fn answered_call() -> crate::AnsweredCall {
    let answer =
        crate::HostAnswer::new(&LIMITS, crate::HostAnswerParts { error: true, body: b"host-answer".as_slice().into() })
            .expect("host answer");
    crate::AnsweredCall::new(
        &LIMITS,
        crate::AnsweredCallParts {
            name: b"call-one".as_slice().into(),
            tool: b"tool".as_slice().into(),
            answer: crate::SettledAnswer::Host(answer),
        },
    )
    .expect("answered call")
}

pub fn assignment_resumed() -> crate::Assignment {
    let mut answered = List::with_capacity(2);
    answered.push(answered_call()).expect("answered call slot");
    let mut messages = List::with_capacity(2);
    for name in [MESSAGE_FIRST, MESSAGE_SECOND] {
        messages.push(start_message(name)).expect("two start message slots");
    }
    let mut grants = List::with_capacity(2);
    for table in [crate::GrantTable::Accounts, crate::GrantTable::Application] {
        grants.push(grant(table, 5, 9)).expect("two grant slots");
    }
    let workspace = crate::Workspace::new(
        &LIMITS,
        crate::WorkspaceParts { workstream: 7, items: b"workspace-items".as_slice().into() },
    )
    .expect("workspace");
    crate::Assignment::new(
        &LIMITS,
        crate::AssignmentParts {
            task: 3,
            attempt: 4,
            identity: IDENTITY_FIRST,
            workspace: Some(workspace),
            save: true,
            charter: b"opaque-charter".as_slice().into(),
            transcript: b"\0\0\0\x08turn-one\0\0\0\x08turn-two".as_slice().into(),
            answered,
            messages,
            grants,
        },
    )
    .expect("resumed assignment")
}

pub const SIZES_ONE_SLOT: crate::sizes::Sizes = crate::sizes::Sizes {
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

pub const SIZES_LARGE: crate::sizes::Sizes = crate::sizes::Sizes {
    charter: 67_108_864,
    transcript: 8_388_608,
    turn: 8_388_608,
    message: 1_048_576,
    sender: 1024,
    call_name: 64,
    tool_name: 64,
    call_input: 1_048_576,
    answer: 1_048_576,
    result: 67_108_864,
    detail: 4096,
    grant: 8192,
    account_id: 256,
    name: 256,
    secret: 1024,
    model_name: 128,
    items: 1_048_576,
    left: 65_536,
    saved: 65_536,
    evidence: 1_048_576,
    resource: 1024,
    slots: 65_536,
    calls: 1024,
    turns: 1024,
    inbox: 1024,
    answered: 1024,
    models: 64,
    workstreams: 65_536,
    accounts: 32,
    credentials: 32,
    facts: 4096,
};

fn one_slot_limits() -> v3::Limits {
    crate::sizes::limits(&SIZES_ONE_SLOT).expect("small size bounds")
}

fn full_bytes() -> Box<[u8]> {
    b"xxxxxxxxxxxxxxxx".as_slice().into()
}

pub fn fullest_turn() -> crate::Turn {
    let limits = one_slot_limits();
    let mut refused = List::with_capacity(1);
    refused.push(refusal(MESSAGE_FIRST, crate::MessageRefusal::Ending)).expect("one refusal slot");
    crate::Turn::new(
        &limits,
        crate::TurnParts {
            task: 3,
            attempt: 4,
            number: 7,
            spent: 100,
            read: Some(MESSAGE_FIRST),
            opens_window: true,
            refused,
            body: full_bytes(),
        },
    )
    .expect("full turn")
}

fn fullest_call(limits: &v3::Limits) -> crate::AnsweredCall {
    let answer = crate::HostAnswer::new(limits, crate::HostAnswerParts { error: true, body: full_bytes() })
        .expect("full host answer");
    crate::AnsweredCall::new(
        limits,
        crate::AnsweredCallParts { name: full_bytes(), tool: full_bytes(), answer: crate::SettledAnswer::Host(answer) },
    )
    .expect("full answered call")
}

fn fullest_grants(limits: &v3::Limits) -> List<crate::Grant> {
    let mut grants = List::with_capacity(2);
    for account in [5, 6] {
        let grant = crate::Grant::new(
            limits,
            crate::GrantParts {
                table: crate::GrantTable::Accounts,
                account,
                generation: 9,
                valid: Duration::from_secs(60),
                value: Secret::new(full_bytes()),
                account_id: Secret::new(full_bytes()),
            },
        )
        .expect("full grant");
        grants.push(grant).expect("two grant slots");
    }
    grants
}

pub fn fullest_assignment() -> crate::Assignment {
    let limits = one_slot_limits();
    let mut answered = List::with_capacity(1);
    answered.push(fullest_call(&limits)).expect("one answered call slot");
    let mut messages = List::with_capacity(1);
    messages
        .push(
            crate::Message::new(
                &limits,
                crate::MessageParts { name: MESSAGE_FIRST, sender: full_bytes(), words: full_bytes() },
            )
            .expect("full message"),
        )
        .expect("one message slot");
    let workspace = crate::Workspace::new(&limits, crate::WorkspaceParts { workstream: 7, items: full_bytes() })
        .expect("full workspace");
    crate::Assignment::new(
        &limits,
        crate::AssignmentParts {
            task: 3,
            attempt: 4,
            identity: IDENTITY_FIRST,
            workspace: Some(workspace),
            save: true,
            charter: full_bytes(),
            transcript: full_bytes(),
            answered,
            messages,
            grants: fullest_grants(&limits),
        },
    )
    .expect("full assignment")
}
