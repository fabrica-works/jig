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
    Call(wire::Call),
    Answer(wire::Answer),
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
        wire::Kind::Call => Some(Up::Call(wire::Call::decode(limits, reader).ok()?)),
        wire::Kind::Answer => Some(Up::Answer(wire::Answer::decode(limits, reader).ok()?)),
        wire::Kind::Fact => Some(Up::Fact(wire::Fact::decode(limits, reader).ok()?)),
        wire::Kind::Rejected => Some(Up::Rejected(wire::Rejected::decode(limits, reader).ok()?)),
        wire::Kind::Exhausted => Some(Up::Exhausted(wire::Exhausted::decode(limits, reader).ok()?)),
        wire::Kind::Hello
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
pub(crate) fn up(record: Up, parts: &mut Parts) -> host::LinkUp {
    match record {
        Up::Turn(record) => turn(record),
        Up::Call(record) => call(record),
        Up::Answer(record) => answer(record, parts),
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

#[derive(Debug)]
pub(crate) struct Parts {
    next: u64,
    values: List<wire::Part>,
}
impl Parts {
    pub fn new(next: u64) -> Self {
        Self { next, values: List::with_capacity(3) }
    }
    pub fn optional(&mut self, bytes: Option<Box<[u8]>>) -> Option<Token> {
        let bytes = bytes?;
        let token = Token::new(self.next);
        self.next = self.next.checked_add(1).expect("part tokens never wrap in a process life");
        self.values.push(wire::Part { token, bytes }).expect("at most resource, left and saved work");
        Some(token)
    }
    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
    pub fn next(&self) -> u64 {
        self.next
    }
    pub fn finish(self) -> Box<[wire::Part]> {
        self.values.into_boxed()
    }
}
pub(crate) fn answer(record: wire::Answer, parts: &mut Parts) -> host::LinkUp {
    let fields = record.into_parts();
    let ending = ending(fields.ending, parts);
    host::LinkUp::Answer {
        run: Token::new(fields.task),
        attempt: Token::new(fields.attempt),
        answer: host::Answer {
            read: optional_token(fields.read),
            refused: refusals(fields.refused),
            turns: fields.turns,
            spent: fields.spent,
            ending,
        },
    }
}
fn ending(value: wire::Ending, parts: &mut Parts) -> host::Ending {
    match value {
        wire::Ending::Refused(record) => {
            let refusal = match record.into_parts().value {
                wire::Refusal::Busy => host::Refusal::Busy,
                wire::Refusal::Invalid(record) => {
                    let invalid = match record.into_parts().value {
                        wire::Invalid::Charter => host::Invalid::Charter,
                        wire::Invalid::Transcript => host::Invalid::Transcript,
                        wire::Invalid::Grants => host::Invalid::Grants,
                        wire::Invalid::Messages => host::Invalid::Messages,
                    };
                    host::Refusal::Invalid(invalid)
                }
            };
            host::Ending::Refused(refusal)
        }
        wire::Ending::Ended(record) => {
            let fields = record.into_parts();
            host::Ending::Ended { outcome: fields.outcome, work: work(fields.work, parts) }
        }
        wire::Ending::Parked(record) => host::Ending::Parked { work: work(record.into_parts().work, parts) },
        wire::Ending::Failed(record) => {
            let fields = record.into_parts();
            let failure = failure(fields.failure, parts);
            host::Ending::Failed { failure, detail: fields.detail, work: work(fields.work, parts) }
        }
    }
}
fn work(value: wire::Work, parts: &mut Parts) -> host::Work {
    let fields = value.into_parts();
    let left = parts.optional(fields.left);
    let saved = parts.optional(fields.saved);
    host::Work { left, saved }
}
fn failure(value: wire::Failure, parts: &mut Parts) -> host::Failure {
    match value {
        wire::Failure::Unprepared(record) => {
            let preparation = match record.into_parts().value {
                wire::Preparation::Transient => host::Preparation::Transient,
                wire::Preparation::Permanent(record) => {
                    host::Preparation::Permanent { resource: parts.optional(record.into_parts().resource) }
                }
            };
            host::Failure::Unprepared(preparation)
        }
        wire::Failure::Run(record) => host::Failure::Run(run_failure(record.into_parts().value)),
        wire::Failure::Agent(record) => host::Failure::Agent(agent_failure(record.into_parts().value)),
        wire::Failure::Cancelled(record) => {
            let reason = match record.into_parts().value {
                wire::CancelReason::Engine => host::Reason::Engine,
                wire::CancelReason::Contact => host::Reason::Contact,
                wire::CancelReason::Shutdown => host::Reason::Shutdown,
            };
            host::Failure::Cancelled(reason)
        }
    }
}
fn run_failure(value: wire::RunFailure) -> host::RunFailure {
    match value {
        wire::RunFailure::Model => host::RunFailure::Model,
        wire::RunFailure::Budget => host::RunFailure::Budget,
        wire::RunFailure::Policy => host::RunFailure::Policy,
        wire::RunFailure::Cancelled => host::RunFailure::Cancelled,
        wire::RunFailure::Stale => host::RunFailure::Stale,
        wire::RunFailure::Exhausted => host::RunFailure::Exhausted,
    }
}
fn agent_failure(value: wire::AgentFailure) -> host::AgentFailure {
    match value {
        wire::AgentFailure::Unstarted => host::AgentFailure::Unstarted,
        wire::AgentFailure::Exited => host::AgentFailure::Exited,
        wire::AgentFailure::Rules => host::AgentFailure::Rules,
        wire::AgentFailure::NoProgress => host::AgentFailure::NoProgress,
        wire::AgentFailure::WallTime => host::AgentFailure::WallTime,
    }
}

pub(crate) fn call(record: wire::Call) -> host::LinkUp {
    let fields = record.into_parts();
    host::LinkUp::Call {
        run: Token::new(fields.task),
        attempt: Token::new(fields.attempt),
        call: fields.name,
        tool: fields.tool,
        writes: fields.writes,
        input: fields.input,
        deadline: fields.deadline,
    }
}
