//! Smith payloads are translated in the worker layer (channel.md, 6.1 and 6.4).
use alloc::boxed::Box;
use jig_domain_host as hub;
use jig_test_worker::{Encodable, Encoded, Invalid};
use skein_lib::{List, Reader, Token, Writer};
use smith_host_domain as smith;
use smith_host_protocol::Limits;
use smith_protocol_channel::Endpoints;

pub(super) fn encode(value: Encodable, limits: &Limits, endpoints: &Endpoints) -> Encoded {
    match value {
        Encodable::Turn(turn) => {
            match smith_protocol_channel::encode_turn(turn.body.value(), &limits.transcript, endpoints) {
                Ok(body) => Encoded::Turn(hub::Turn {
                    opens_window: turn.opens_window,
                    refused: Box::default(),
                    turn: turn.number,
                    spent: turn.spent,
                    read: turn.read,
                    body,
                }),
                Err(_) => Encoded::Undecodable,
            }
        }
        Encodable::CallName(name) => {
            let record = smith_channel::CallName::new(
                &limits.bodies,
                smith_channel::CallNameParts {
                    activation: name.activation,
                    completion: name.completion,
                    position: name.position,
                },
            );
            let Ok(record) = record else { return Encoded::Undecodable };
            let Ok(length) = usize::try_from(record.measure()) else { return Encoded::Undecodable };
            let mut writer = Writer::new(length);
            if record.encode(&mut writer).is_err() {
                return Encoded::Undecodable;
            }
            Encoded::CallName(writer.finish())
        }
        Encodable::Result(result) => match result {
            smith::RunResult::Accepted { outcome } => {
                match smith_protocol_channel::encode_result(outcome.value(), &limits.charter) {
                    Ok(body) => Encoded::Result(body),
                    Err(_) => Encoded::Undecodable,
                }
            }
            smith::RunResult::Refused { refusal } => Encoded::Result(Box::from(refused(refusal))),
            // The root retains these typed dispositions beside the hand-out.
            smith::RunResult::Parked | smith::RunResult::Failed { .. } => Encoded::Result(Box::default()),
        },
    }
}

fn refused(refusal: smith::Refusal) -> &'static [u8] {
    match refusal {
        smith::Refusal::Busy => b"agent busy",
        smith::Refusal::Invalid(invalid) => match invalid {
            smith::RunInvalid::HostPart { .. } => b"agent host part bound",
            smith::RunInvalid::Room { .. } => b"agent model room bound",
            smith::RunInvalid::Messages => b"agent messages invalid",
            smith::RunInvalid::CharterVersion => b"agent charter version invalid",
            smith::RunInvalid::MalformedCharter => b"agent charter malformed",
            smith::RunInvalid::Activation => b"agent activation invalid",
            smith::RunInvalid::Window => b"agent window invalid",
            smith::RunInvalid::Conventions => b"agent conventions invalid",
            smith::RunInvalid::TooLarge => b"agent charter too large",
            smith::RunInvalid::Workspace => b"agent workspace invalid",
            smith::RunInvalid::Grants => b"agent grants invalid",
            smith::RunInvalid::Outcome => b"agent outcome invalid",
            smith::RunInvalid::Budget(budget) => match budget {
                smith::BudgetRefusal::Ceiling(ceiling) => match ceiling {
                    smith::Ceiling::Turns(_) => b"agent turns ceiling",
                    smith::Ceiling::Spend(_) => b"agent spend ceiling",
                    smith::Ceiling::Time(_) => b"agent time ceiling",
                },
                smith::BudgetRefusal::Empty(part) => match part {
                    smith::Part::Turns => b"agent turns empty",
                    smith::Part::Spend => b"agent spend empty",
                    smith::Part::Time => b"agent time empty",
                },
                smith::BudgetRefusal::Reserve(part) => match part {
                    smith::Part::Turns => b"agent turns reserve",
                    smith::Part::Spend => b"agent spend reserve",
                    smith::Part::Time => b"agent time reserve",
                },
                smith::BudgetRefusal::Checks { .. } => b"agent checks reserve",
                smith::BudgetRefusal::Completion { .. } => b"agent completion reserve",
            },
            smith::RunInvalid::Price(rate) => match rate {
                smith::Rate::Input => b"agent input price invalid",
                smith::Rate::Cached => b"agent cached price invalid",
                smith::Rate::CacheWrite => b"agent cache write price invalid",
                smith::Rate::Output => b"agent output price invalid",
            },
            smith::RunInvalid::Llm => b"agent llm invalid",
            smith::RunInvalid::Conversation => b"agent conversation invalid",
            smith::RunInvalid::Endpoint => b"agent endpoint invalid",
            smith::RunInvalid::Effort => b"agent effort invalid",
        },
    }
}

pub(super) fn activity(body: &[u8], limits: &Limits) -> Option<hub::Activity> {
    let mut reader = Reader::new(body);
    let Ok(fact) = smith_channel::Fact::decode(&limits.bodies, &mut reader) else { return None };
    if !reader.is_empty() {
        return None;
    }
    let kind = match fact.kind() {
        smith_channel::FactKind::Admitted => hub::CountKind::Admitted,
        smith_channel::FactKind::Ended => hub::CountKind::Ended,
        smith_channel::FactKind::LlmStarted => hub::CountKind::LlmStarted,
        smith_channel::FactKind::LlmRetried => hub::CountKind::LlmRetried,
        smith_channel::FactKind::LlmFinished => hub::CountKind::LlmFinished,
        smith_channel::FactKind::ToolStarted => hub::CountKind::ToolStarted,
        smith_channel::FactKind::ToolFinished => hub::CountKind::ToolFinished,
        smith_channel::FactKind::CheckStarted => hub::CountKind::CheckStarted,
        smith_channel::FactKind::CheckFinished => hub::CountKind::CheckFinished,
        smith_channel::FactKind::TextArrived => hub::CountKind::TextArrived,
    };
    Some(hub::Activity::Count { kind, elapsed: fact.elapsed(), count: fact.count() })
}

pub(super) fn decode(run: Token, source: hub::Start, agent: &crate::Agent) -> Result<Box<smith::Start>, Invalid> {
    let Ok(charter) = smith_protocol_channel::decode_charter(&source.charter, &agent.limits.charter, &agent.endpoints)
    else {
        return Err(Invalid::Charter);
    };
    let charter = smith::Charter::new(charter, agent.host_limits.charter_bytes).ok_or(Invalid::Charter)?;
    let piece = smith_protocol_channel::transcript_piece(&source.turns).ok_or(Invalid::Transcript)?;
    let transcript = match smith_protocol_channel::decode_transcript(
        &piece,
        u32::try_from(agent.host_limits.transcript_bytes.checked_div(4).ok_or(Invalid::Transcript)?)
            .or(Err(Invalid::Transcript))?,
        agent.limits.transcript.found_hits,
        &agent.limits.transcript,
        &agent.endpoints,
    ) {
        Ok(Some(value)) => {
            Some(smith::Transcript::new(value, agent.host_limits.transcript_bytes).ok_or(Invalid::Transcript)?)
        }
        Ok(None) => None,
        Err(_) => return Err(Invalid::Transcript),
    };
    let mut answered = List::with_capacity(u32::try_from(source.answered.len()).or(Err(Invalid::CallName))?);
    for call in source.answered {
        let mut reader = Reader::new(&call.name);
        let name = match smith_channel::CallName::decode(&agent.limits.bodies, &mut reader) {
            Ok(value) if reader.is_empty() => smith::CallName {
                activation: value.activation(),
                completion: value.completion(),
                position: value.position(),
            },
            Ok(_) | Err(_) => return Err(Invalid::CallName),
        };
        let reply = match call.answer {
            hub::SettledAnswer::Host { error, body } => smith::SavedReply::Host { error, body },
            hub::SettledAnswer::Delivery { outcome, evidence } => {
                smith::SavedReply::Delivery(Box::new(delivery(outcome, evidence)?))
            }
        };
        answered.push(smith::AnsweredCall { name, tool: call.tool, reply }).expect("answer count");
    }
    let mut messages = List::with_capacity(u32::try_from(source.messages.len()).or(Err(Invalid::Charter))?);
    for message in source.messages {
        messages
            .push(smith::Message { name: message.name, label: message.sender, text: message.words })
            .expect("inbox count");
    }
    let mut grants = List::with_capacity(u32::try_from(source.grants.len()).or(Err(Invalid::Charter))?);
    for grant in source.grants {
        if let Some(grant) = jig_test_worker::translate::grant(grant) {
            grants.push(grant).expect("grant count");
        }
    }
    Ok(Box::new(smith::Start {
        identity: Some(smith_domain_run::Identity::new(source.identity)),
        live: None,
        logical_run: run,
        activation: source.activation,
        workspace: source.workspace,
        charter,
        transcript,
        answered: answered.into_boxed(),
        messages: messages.into_boxed(),
        directories: Box::default(),
        grants: grants.into_boxed(),
    }))
}

fn delivery(outcome: hub::DeliveryOutcome, evidence: Box<[u8]>) -> Result<smith::Delivery, Invalid> {
    Ok(match outcome {
        hub::DeliveryOutcome::Delivered => {
            let receipt = smith::Receipt::new(0, evidence).ok_or(Invalid::CallName)?;
            smith::Delivery::Delivered(smith::Delivered::new(Box::new([receipt])).ok_or(Invalid::CallName)?)
        }
        hub::DeliveryOutcome::Nothing => smith::Delivery::Nothing,
        hub::DeliveryOutcome::Stale => smith::Delivery::Stale,
        hub::DeliveryOutcome::Refused => {
            smith::Delivery::Refused(smith::DeliveryRefusal::new(None, evidence).ok_or(Invalid::CallName)?)
        }
        hub::DeliveryOutcome::Failed => smith::Delivery::Failed(smith::DeliveryFailure {
            directory: 0,
            reason: smith::DeliveryReason::Unknown,
            diagnostic: smith::Diagnostic::new(&evidence, 0),
        }),
    })
}

pub(super) fn process(event: smith_host_protocol::ProcessEvent) -> Option<jig_test_worker::Event> {
    use smith_host_protocol::{OpenEvent, ProcessEvent};
    let event = match event {
        ProcessEvent::Spawned { agent, process } => smith::process::Event::Spawned { owner: agent, process },
        ProcessEvent::Unspawned { agent, detail } => smith::process::Event::Unspawned { owner: agent, detail },
        ProcessEvent::Exited { agent, exit } => smith::process::Event::Exited { owner: agent, exit: exited(exit) },
        ProcessEvent::Reaped { agent, detail } => smith::process::Event::Reaped { owner: agent, detail },
        ProcessEvent::Channel { agent, event } => match *event {
            OpenEvent::Opened { .. } | OpenEvent::Live { .. } | OpenEvent::WriteFailed => return None,
            OpenEvent::Hangup { .. } => smith::process::Event::Hangup { owner: agent },
            OpenEvent::Sent { .. } => smith::process::Event::Sent { owner: agent },
            OpenEvent::Unsent { .. } => smith::process::Event::Unsent { owner: agent },
            event @ (OpenEvent::Answer { .. }
            | OpenEvent::Admitted
            | OpenEvent::MessageRefused { .. }
            | OpenEvent::Waiting { .. }
            | OpenEvent::Long { .. }
            | OpenEvent::LongDone
            | OpenEvent::Turn { .. }
            | OpenEvent::Fact { .. }
            | OpenEvent::Unauthorized { .. }
            | OpenEvent::Limited { .. }
            | OpenEvent::Refused { .. }
            | OpenEvent::Call { .. }
            | OpenEvent::Withdraw { .. }) => smith::process::Event::Received { owner: agent, message: channel(event)? },
        },
    };
    Some(jig_test_worker::Event::Process(Box::new(event)))
}
fn exited(exit: Option<skein_io::kernel::Exit>) -> Option<smith::Exit> {
    match exit {
        Some(skein_io::kernel::Exit::Code(code)) => Some(smith::Exit::Code(code)),
        Some(skein_io::kernel::Exit::Signal(signal)) => Some(smith::Exit::Signal(signal)),
        None => None,
    }
}
fn channel(event: smith_host_protocol::OpenEvent) -> Option<smith::Up> {
    use smith_host_protocol::OpenEvent;
    Some(match event {
        OpenEvent::MessageRefused { name, reason } => smith::Up::MessageRefused { name, reason },
        OpenEvent::Answer { answer, .. } => smith::Up::Answer { answer },
        OpenEvent::Admitted => smith::Up::Admitted,
        OpenEvent::Waiting { read } => smith::Up::Waiting { read },
        OpenEvent::Long { operation, span } => smith::Up::Long { operation, span },
        OpenEvent::LongDone => smith::Up::LongDone,
        OpenEvent::Turn { turn } => smith::Up::Turn { turn },
        OpenEvent::Fact { body } => smith::Up::Fact { body },
        OpenEvent::Unauthorized { account, generation } => smith::Up::Unauthorized { account, generation },
        OpenEvent::Limited { account, reset } => smith::Up::Limited { account, reset },
        OpenEvent::Refused { account, code, reason } => smith::Up::Refused { account, code, reason },
        OpenEvent::Call { call, name, deadline, ask } => smith::Up::Call { call, name, deadline, ask },
        OpenEvent::Withdraw { call } => smith::Up::Withdraw { call },
        OpenEvent::Opened { .. }
        | OpenEvent::Live { .. }
        | OpenEvent::WriteFailed
        | OpenEvent::Hangup { .. }
        | OpenEvent::Sent { .. }
        | OpenEvent::Unsent { .. } => return None,
    })
}
