use crate::{Credential, Limits};
use skein_lib::{Duration, Secret};
pub const WORKER_A: &[u8] = b"worker-a";
pub const SECRET_A: &[u8] = b"secret-a";
pub const PAYLOAD: u32 = 3;
pub const SEED_ONE: u64 = 1;
pub const SIZES: jig_protocol_worker_wire::sizes::Sizes = jig_protocol_worker_wire::sizes::Sizes {
    program_sources: 2,
    piece: 16,
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
    facts: 390,
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
pub fn credential() -> Credential {
    Credential { name: WORKER_A.into(), secret: Secret::new(SECRET_A.into()), payload: PAYLOAD }
}

use jig_domain_host as host;
use jig_protocol_worker_wire::{self as wire, Part};
use skein_lib::{List, Token, Writer};
pub const TOKEN_ONE: &[u8] = b"access";
pub const RUN_SEVEN: Token = Token::new(7);
pub const ATTEMPT_TWO: Token = Token::new(2);
pub fn codec() -> wire::v7::Limits {
    wire::sizes::limits(&SIZES).expect("small codec limits")
}
pub fn hello() -> host::Hello {
    host::Hello {
        slots: 1,
        stop_bound: Duration::from_secs(1),
        workstreams: Box::new([3]),
        hosting: Box::new([host::Hosting { run: RUN_SEVEN, attempt: ATTEMPT_TWO, phase: host::Phase::Active }]),
        bounds: host::Bounds {
            program_endpoints: Box::new([]),
            models: Box::new([host::ModelBound { name: b"model".as_slice().into(), window: 16, output: 8 }]),
            turns: 1,
            spend: 10,
            time: Duration::from_secs(60),
            waiting: Duration::from_secs(5),
            checks: Duration::from_secs(10),
            message_bytes: 16,
            inbox: 1,
            unacknowledged: 1,
        },
    }
}
pub fn turn() -> host::LinkUp {
    host::LinkUp::Turn {
        run: RUN_SEVEN,
        attempt: ATTEMPT_TWO,
        turn: host::Turn {
            opens_window: true,
            refused: Box::new([host::Refused { name: Token::new(4), reason: host::MessageRefusal::Full }]),
            turn: 1,
            spent: 3,
            read: Some(Token::new(5)),
            body: b"turn".as_slice().into(),
        },
    }
}
pub fn answer() -> host::LinkUp {
    host::LinkUp::Answer {
        run: RUN_SEVEN,
        attempt: ATTEMPT_TWO,
        answer: host::Answer {
            read: Some(Token::new(5)),
            refused: Box::new([]),
            turns: 1,
            spent: 3,
            ending: host::Ending::Ended {
                outcome: b"result".as_slice().into(),
                work: host::Work { left: Some(Token::new(10)), saved: Some(Token::new(11)) },
            },
        },
    }
}
pub fn work_parts() -> Box<[Part]> {
    Box::new([
        Part { token: Token::new(10), bytes: b"left".as_slice().into() },
        Part { token: Token::new(11), bytes: b"saved".as_slice().into() },
    ])
}
pub fn credential_value(scheme: wire::Scheme) -> wire::GrantValue {
    let parts = wire::CredentialValueParts { scheme, bytes: Secret::new(TOKEN_ONE.into()) };
    wire::GrantValue::Credential(wire::CredentialValue::new(&codec(), parts).expect("credential"))
}
pub fn application_value() -> wire::GrantValue {
    let parts = wire::ApplicationValueParts { secret: Secret::new(TOKEN_ONE.into()) };
    wire::GrantValue::Application(wire::ApplicationValue::new(&codec(), parts).expect("application"))
}
pub fn grant_record(value: wire::GrantValue) -> wire::Grant {
    wire::Grant::new(&codec(), wire::GrantParts { number: 1, generation: 3, valid: Duration::from_secs(60), value })
        .expect("grant")
}
pub fn assignment_record(transcript: Box<[u8]>) -> wire::Assignment {
    let limits = codec();
    let workspace =
        wire::Workspace::new(&limits, wire::WorkspaceParts { workstream: 3, items: b"items".as_slice().into() })
            .expect("workspace");
    let mut messages = List::with_capacity(1);
    messages
        .push(
            wire::Message::new(
                &limits,
                wire::MessageParts { name: 5, sender: b"sender".as_slice().into(), words: b"words".as_slice().into() },
            )
            .expect("message"),
        )
        .expect("room");
    let mut grants = List::with_capacity(2);
    grants.push(grant_record(credential_value(wire::Scheme::Bearer))).expect("room");
    grants.push(grant_record(application_value())).expect("room");
    let mut answered = List::with_capacity(1);
    let answer =
        wire::HostAnswer::new(&limits, wire::HostAnswerParts { error: false, body: b"answer".as_slice().into() })
            .expect("answer");
    answered
        .push(
            wire::AnsweredCall::new(
                &limits,
                wire::AnsweredCallParts {
                    name: b"call".as_slice().into(),
                    tool: b"tool".as_slice().into(),
                    answer: wire::SettledAnswer::Host(answer),
                },
            )
            .expect("call"),
        )
        .expect("room");
    wire::Assignment::new(
        &limits,
        wire::AssignmentParts {
            task: 7,
            attempt: 2,
            identity: [9; 16],
            workspace: Some(workspace),
            save: true,
            live: wire::LiveAsk::new(&limits, wire::LiveAskParts { text: false, reasoning: false, input: false })
                .expect("live ask"),
            charter: b"charter".as_slice().into(),
            transcript,
            answered,
            messages,
            grants,
        },
    )
    .expect("assignment")
}
pub fn window() -> Box<[u8]> {
    let mut writer = Writer::new(16);
    skein_codec::piece::write(&mut writer, b"one!").expect("first turn");
    skein_codec::piece::write(&mut writer, b"two!").expect("second turn");
    writer.finish()
}

fn writer(measured: u32) -> Writer {
    Writer::new(usize::try_from(measured).expect("small record"))
}

pub fn assignment_bytes(record: &wire::Assignment) -> Box<[u8]> {
    let mut bytes = writer(record.measure());
    record.encode(&mut bytes).expect("measured assignment");
    bytes.finish()
}

pub fn granted_bytes(record: &wire::Granted) -> Box<[u8]> {
    let mut bytes = writer(record.measure());
    record.encode(&mut bytes).expect("measured granted");
    bytes.finish()
}

pub fn inbound_bytes(record: &wire::Inbound) -> Box<[u8]> {
    let mut bytes = writer(record.measure());
    record.encode(&mut bytes).expect("measured inbound");
    bytes.finish()
}
