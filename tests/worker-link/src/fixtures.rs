use jig_protocol_worker_link::{Credential, Limits};
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
    skein_fake_channel::ScriptedPeer::new(
        schema,
        skein_channel::Role::Responder,
        channel_limits(),
        wire::VERSION,
        Box::new([]),
    )
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

use jig_domain_host as host;
use jig_protocol_worker_wire::{self as wire, Part};
use skein_lib::{List, Token, Writer};
pub const TOKEN_ONE: &[u8] = b"access";
pub const TOKEN_TWO: &[u8] = b"app-access";
pub const RUN_SEVEN: Token = Token::new(7);
pub const ATTEMPT_TWO: Token = Token::new(2);
#[must_use]
pub fn codec() -> wire::v7::Limits {
    wire::sizes::limits(&SIZES).expect("small codec limits")
}
#[must_use]
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
#[must_use]
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
#[must_use]
pub fn up_records() -> Vec<(host::LinkUp, Box<[Part]>)> {
    vec![
        (turn(), Box::new([])),
        (
            host::LinkUp::Call {
                run: RUN_SEVEN,
                attempt: ATTEMPT_TWO,
                call: b"call".as_slice().into(),
                tool: b"tool".as_slice().into(),
                writes: true,
                input: b"{}".as_slice().into(),
                deadline: Duration::from_secs(60),
            },
            Box::new([]),
        ),
        (answer(), work_parts()),
        (
            host::LinkUp::Activity {
                run: RUN_SEVEN,
                attempt: ATTEMPT_TWO,
                activity: host::Activity::Long { operation: host::Operation::Command, span: Duration::from_secs(2) },
            },
            Box::new([]),
        ),
        (host::LinkUp::Unauthorized { run: RUN_SEVEN, attempt: ATTEMPT_TWO, account: 1, generation: 3 }, Box::new([])),
        (
            host::LinkUp::Limited {
                run: RUN_SEVEN,
                attempt: ATTEMPT_TWO,
                account: 1,
                reset: Some(Duration::from_secs(60)),
            },
            Box::new([]),
        ),
        (account_refused(), Box::new([])),
    ]
}
#[must_use]
pub fn account_refused() -> host::LinkUp {
    host::LinkUp::AccountRefused {
        run: RUN_SEVEN,
        attempt: ATTEMPT_TWO,
        account: 1,
        code: b"forbidden".as_slice().into(),
        reason: b"suspended".as_slice().into(),
    }
}
#[must_use]
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
#[must_use]
pub fn work_parts() -> Box<[Part]> {
    Box::new([
        Part { token: Token::new(10), bytes: b"left".as_slice().into() },
        Part { token: Token::new(11), bytes: b"saved".as_slice().into() },
    ])
}
#[must_use]
pub fn credential_value(scheme: wire::Scheme) -> wire::GrantValue {
    let parts = wire::CredentialValueParts { scheme, bytes: Secret::new(TOKEN_ONE.into()) };
    wire::GrantValue::Credential(wire::CredentialValue::new(&codec(), parts).expect("credential"))
}
#[must_use]
pub fn application_value() -> wire::GrantValue {
    let parts = wire::ApplicationValueParts { secret: Secret::new(TOKEN_ONE.into()) };
    wire::GrantValue::Application(wire::ApplicationValue::new(&codec(), parts).expect("application"))
}
#[must_use]
pub fn grant_record(value: wire::GrantValue) -> wire::Grant {
    wire::Grant::new(&codec(), wire::GrantParts { number: 1, generation: 3, valid: Duration::from_secs(60), value })
        .expect("grant")
}
#[must_use]
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
#[must_use]
pub fn window() -> Box<[u8]> {
    let mut writer = Writer::new(16);
    skein_codec::piece::write(&mut writer, b"one!").expect("first turn");
    skein_codec::piece::write(&mut writer, b"two!").expect("second turn");
    writer.finish()
}
#[must_use]
pub fn frame(kind: u16, body: &[u8]) -> skein_channel::Frame {
    let mut frame = skein_channel::frame_writer(kind, u32::try_from(body.len()).expect("small body")).expect("frame");
    frame.put(body).expect("measured body");
    frame.finish().expect("complete")
}
#[must_use]
pub fn assignment_frame(transcript: Box<[u8]>) -> skein_channel::Frame {
    let record = assignment_record(transcript);
    let body = encoded(record.measure(), |writer| record.encode(writer));
    frame(0x0180, &body)
}
#[must_use]
pub fn down_frames() -> Vec<skein_channel::Frame> {
    let limits = codec();
    let mut frames = vec![assignment_frame(window())];
    let record = wire::Inbound::new(
        &limits,
        wire::InboundParts {
            task: 7,
            attempt: 2,
            name: 6,
            sender: b"sender".as_slice().into(),
            words: b"later".as_slice().into(),
        },
    )
    .expect("message");
    let body = encoded(record.measure(), |writer| record.encode(writer));
    frames.push(frame(0x0181, &body));
    let record = wire::TurnKept::new(&limits, wire::TurnKeptParts { task: 7, attempt: 2, number: 1 }).expect("kept");
    let body = encoded(record.measure(), |writer| record.encode(writer));
    frames.push(frame(0x0183, &body));
    let record = wire::Acknowledge::new(&limits, wire::AcknowledgeParts { task: 7, attempt: 2 }).expect("acknowledge");
    let body = encoded(record.measure(), |writer| record.encode(writer));
    frames.push(frame(0x0185, &body));
    let record = wire::Cancel::new(&limits, wire::CancelParts { task: 7, attempt: 2 }).expect("cancel");
    let body = encoded(record.measure(), |writer| record.encode(writer));
    frames.push(frame(0x0186, &body));
    let record = wire::Granted::new(
        &limits,
        wire::GrantedParts { task: 7, attempt: 2, grant: grant_record(credential_value(wire::Scheme::Bearer)) },
    )
    .expect("grant");
    let body = encoded(record.measure(), |writer| record.encode(writer));
    frames.push(frame(0x0187, &body));
    frames
}

pub fn encoded<F: FnOnce(&mut Writer) -> Result<(), skein_lib::Overflow>>(measured: u32, encode: F) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(measured).expect("small record"));
    encode(&mut writer).expect("measured record");
    writer.finish()
}

#[must_use]
pub fn call(name: &[u8]) -> host::LinkUp {
    host::LinkUp::Call {
        run: RUN_SEVEN,
        attempt: ATTEMPT_TWO,
        call: name.into(),
        tool: b"tool".as_slice().into(),
        writes: true,
        input: b"{}".as_slice().into(),
        deadline: Duration::from_secs(60),
    }
}
#[must_use]
pub fn relayed(name: &[u8], busy: bool) -> skein_channel::Frame {
    let limits = codec();
    let answer = if busy {
        wire::RelayAnswer::Busy
    } else {
        wire::RelayAnswer::Text(
            wire::RelayText::new(&limits, wire::RelayTextParts { error: false, text: b"answer".as_slice().into() })
                .expect("answer"),
        )
    };
    let record = wire::Relayed::new(&limits, wire::RelayedParts { task: 7, attempt: 2, name: name.into(), answer })
        .expect("relayed");
    frame(0x0182, &encoded(record.measure(), |writer| record.encode(writer)))
}
#[must_use]
pub fn turn_busy() -> skein_channel::Frame {
    let record = wire::TurnBusy::new(&codec(), wire::TurnBusyParts { task: 7, attempt: 2, number: 1 }).expect("busy");
    frame(0x0184, &encoded(record.measure(), |writer| record.encode(writer)))
}
#[must_use]
pub fn turn_kept() -> skein_channel::Frame {
    let record = wire::TurnKept::new(&codec(), wire::TurnKeptParts { task: 7, attempt: 2, number: 1 }).expect("kept");
    frame(0x0183, &encoded(record.measure(), |writer| record.encode(writer)))
}
#[must_use]
pub fn acknowledge() -> skein_channel::Frame {
    let record = wire::Acknowledge::new(&codec(), wire::AcknowledgeParts { task: 7, attempt: 2 }).expect("acknowledge");
    frame(0x0185, &encoded(record.measure(), |writer| record.encode(writer)))
}

#[must_use]
pub fn engine_accepting_with(limits: &Limits) -> skein_fake_channel::ScriptedPeer {
    let codec = wire::sizes::limits(&limits.sizes).expect("sizes");
    let schema = wire::schema(&codec).expect("schema");
    let channel = jig_protocol_worker_link::channel_limits(limits).expect("channel limits");
    skein_fake_channel::ScriptedPeer::new(schema, skein_channel::Role::Responder, channel, wire::VERSION, Box::new([]))
        .expect("engine")
}

#[must_use]
pub fn joined_limits() -> Limits {
    limits()
}
#[must_use]
pub fn grant(form: host::Form) -> host::Grant {
    host::Grant { form, account: 1, generation: 3, valid: Duration::from_secs(60) }
}
#[must_use]
pub fn assignment() -> host::Assignment {
    host::Assignment {
        assignment: host::RunAssignment {
            run: RUN_SEVEN,
            attempt: ATTEMPT_TWO,
            identity: [9; 16],
            workspace: Some(host::Workspace { workstream: 3, items: Token::new(10) }),
            save: true,
            live: jig_domain_host::LiveAsk::NONE,
            charter: b"charter".as_slice().into(),
            grants: Box::new([grant(host::Form::Credential(host::Scheme::Bearer)), grant(host::Form::Application)]),
        },
        turns: Box::new([Box::from(b"one!".as_slice()), Box::from(b"two!".as_slice())]),
        answered: Box::new([host::AnsweredCall {
            name: b"call".as_slice().into(),
            tool: b"tool".as_slice().into(),
            answer: host::SettledAnswer::Host { error: false, body: b"answer".as_slice().into() },
        }]),
        messages: Box::new([host::StartMessage {
            name: Token::new(5),
            sender: b"sender".as_slice().into(),
            words: b"words".as_slice().into(),
        }]),
    }
}
#[must_use]
pub fn assignment_parts() -> Box<[Part]> {
    Box::new([Part { token: Token::new(10), bytes: b"items".as_slice().into() }])
}
#[must_use]
pub fn relay_answer(name: &[u8]) -> host::LinkDown {
    host::LinkDown::Relayed {
        run: RUN_SEVEN,
        attempt: ATTEMPT_TWO,
        call: name.into(),
        answer: host::Relayed::Answer { error: false, text: b"answer".as_slice().into() },
    }
}
