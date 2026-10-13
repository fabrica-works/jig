//! Total translations of the worker records (protocol/channel.md, section 3).
use alloc::boxed::Box;
use jig_domain_host as host;
use jig_protocol_worker_wire as wire;
use skein_lib::{List, Reader, Token};
pub(crate) fn hello(record: wire::Hello) -> host::Hello {
    let fields = record.into_parts();
    let bounds = fields.bounds.into_parts();
    let mut models = List::with_capacity(bounds.models.len());
    for model in bounds.models.into_boxed() {
        let model = model.into_parts();
        models
            .push(host::ModelBound { name: model.name, window: model.window, output: model.output })
            .expect("one slot per decoded model");
    }
    let mut hosting = List::with_capacity(fields.hosting.len());
    for run in fields.hosting.into_boxed() {
        let run = run.into_parts();
        hosting
            .push(host::Hosting {
                run: Token::new(run.task),
                attempt: Token::new(run.attempt),
                phase: phase(run.phase),
            })
            .expect("one slot per decoded run");
    }
    host::Hello {
        slots: fields.slots,
        stop_bound: fields.stop_bound,
        workstreams: fields.workstreams.into_boxed(),
        hosting: hosting.into_boxed(),
        bounds: host::Bounds {
            models: models.into_boxed(),
            turns: bounds.turns,
            spend: bounds.spend,
            time: bounds.time,
            waiting: bounds.waiting,
            checks: bounds.checks,
            message_bytes: bounds.message_bytes,
            inbox: bounds.inbox,
            unacknowledged: bounds.unacknowledged,
        },
    }
}
fn phase(value: wire::Phase) -> host::Phase {
    match value {
        wire::Phase::Preparing => host::Phase::Preparing,
        wire::Phase::Starting => host::Phase::Starting,
        wire::Phase::Active => host::Phase::Active,
        wire::Phase::Waiting => host::Phase::Waiting,
        wire::Phase::Ending => host::Phase::Ending,
        wire::Phase::Answered => host::Phase::Answered,
    }
}

#[derive(Debug)]
pub(crate) enum Up {
    Turn(wire::Turn),
    Fact(wire::Fact),
    Rejected(wire::Rejected),
    Exhausted(wire::Exhausted),
}
pub(crate) fn decode(kind: u16, body: &[u8], limits: &wire::v1::Limits) -> Option<Up> {
    let mut found = None;
    for rule in wire::KINDS {
        if rule.kind == kind {
            found = Some(rule.body);
            break;
        }
    }
    let kind = found?;
    let reader = &mut Reader::new(body);
    match kind {
        wire::Kind::Turn => Some(Up::Turn(wire::Turn::decode(limits, reader).ok()?)),
        wire::Kind::Fact => Some(Up::Fact(wire::Fact::decode(limits, reader).ok()?)),
        wire::Kind::Rejected => Some(Up::Rejected(wire::Rejected::decode(limits, reader).ok()?)),
        wire::Kind::Exhausted => Some(Up::Exhausted(wire::Exhausted::decode(limits, reader).ok()?)),
        // Call and Answer wait for the crossing vocabulary's rulings (plan ruling 183).
        wire::Kind::Call
        | wire::Kind::Answer
        | wire::Kind::Hello
        | wire::Kind::Assignment
        | wire::Kind::Message
        | wire::Kind::Relayed
        | wire::Kind::TurnKept
        | wire::Kind::TurnBusy
        | wire::Kind::Acknowledge
        | wire::Kind::Cancel
        | wire::Kind::Grant => None,
    }
}
pub(crate) fn up(record: Up) -> host::LinkUp {
    match record {
        Up::Turn(record) => turn(record),
        Up::Fact(record) => fact(record),
        Up::Rejected(record) => {
            let fields = record.into_parts();
            host::LinkUp::Rejected {
                run: Token::new(fields.task),
                attempt: Token::new(fields.attempt),
                account: fields.account,
                generation: fields.generation,
            }
        }
        Up::Exhausted(record) => {
            let fields = record.into_parts();
            host::LinkUp::Exhausted {
                run: Token::new(fields.task),
                attempt: Token::new(fields.attempt),
                account: fields.account,
                retry_after: fields.retry_after,
            }
        }
    }
}
fn turn(record: wire::Turn) -> host::LinkUp {
    let fields = record.into_parts();
    let refused = refusals(fields.refused);
    host::LinkUp::Turn {
        run: Token::new(fields.task),
        attempt: Token::new(fields.attempt),
        turn: host::Turn {
            opens_window: fields.opens_window,
            refused,
            turn: fields.number,
            spent: fields.spent,
            read: optional_token(fields.read),
            body: fields.body,
        },
    }
}
fn refusals(records: List<wire::Refused>) -> Box<[host::Refused]> {
    let mut refused = List::with_capacity(records.len());
    for record in records.into_boxed() {
        let fields = record.into_parts();
        let reason = match fields.reason {
            wire::MessageRefusal::TooLarge => host::MessageRefusal::TooLarge,
            wire::MessageRefusal::Full => host::MessageRefusal::Full,
            wire::MessageRefusal::NameInUse => host::MessageRefusal::NameInUse,
            wire::MessageRefusal::Ending => host::MessageRefusal::Ending,
        };
        refused.push(host::Refused { name: Token::new(fields.name), reason }).expect("one slot per refusal");
    }
    refused.into_boxed()
}
fn optional_token(value: Option<u64>) -> Option<Token> {
    Some(Token::new(value?))
}
fn fact(record: wire::Fact) -> host::LinkUp {
    let fields = record.into_parts();
    let activity = match fields.activity {
        wire::Activity::Phase(value) => host::Activity::Phase(phase(value.into_parts().value)),
        wire::Activity::Long(value) => {
            let fields = value.into_parts();
            let operation = match fields.operation {
                wire::Operation::Checks => host::Operation::Checks,
                wire::Operation::Completion => host::Operation::Completion,
                wire::Operation::Command => host::Operation::Command,
            };
            host::Activity::Long { operation, span: fields.span }
        }
        wire::Activity::LongDone => host::Activity::LongDone,
        wire::Activity::Count(value) => {
            let fields = value.into_parts();
            host::Activity::Count { kind: count_kind(fields.kind), elapsed: fields.elapsed, count: fields.value }
        }
    };
    host::LinkUp::Activity { run: Token::new(fields.task), attempt: Token::new(fields.attempt), activity }
}
fn count_kind(kind: wire::CountKind) -> host::CountKind {
    match kind {
        wire::CountKind::Admitted => host::CountKind::Admitted,
        wire::CountKind::Ended => host::CountKind::Ended,
        wire::CountKind::LlmStarted => host::CountKind::LlmStarted,
        wire::CountKind::LlmRetried => host::CountKind::LlmRetried,
        wire::CountKind::LlmFinished => host::CountKind::LlmFinished,
        wire::CountKind::ToolStarted => host::CountKind::ToolStarted,
        wire::CountKind::ToolFinished => host::CountKind::ToolFinished,
        wire::CountKind::CheckStarted => host::CountKind::CheckStarted,
        wire::CountKind::CheckFinished => host::CountKind::CheckFinished,
        wire::CountKind::TextArrived => host::CountKind::TextArrived,
    }
}
