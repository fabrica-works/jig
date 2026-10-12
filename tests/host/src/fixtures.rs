//! Host records used by the lifecycle scripts. Callback tokens are encoded
//! as opaque names; names are opaque bytes.
use jig_domain_host::LinkDown;
use jig_domain_host::{Ask, Assignment, Event, Finish, RunAssignment};
use skein_lib::{Duration, Reader, ReplyTo, Token};
#[must_use]
pub fn assign(reply_to: ReplyTo, assignment: RunAssignment) -> Event {
    assign_with(reply_to, assignment, Box::new([]))
}
#[must_use]
pub fn assign_with(
    reply_to: ReplyTo,
    assignment: RunAssignment,
    messages: Box<[jig_domain_host::StartMessage]>,
) -> Event {
    Event::Down(LinkDown::Assign {
        reply_to,
        assignment: Assignment { messages, assignment, turns: Box::new([]), answered: Box::new([]) },
    })
}

#[must_use]
pub fn inbound(run: Token, attempt: Token, name: Token, words: Box<[u8]>) -> Event {
    Event::Down(LinkDown::Message { run, attempt, name, sender: Box::new([]), words })
}
#[must_use]
pub fn called(owner: Token, call: Token, ask: Ask) -> Event {
    Event::Called { owner, call: Box::from(call.raw().to_be_bytes()), ask }
}
#[must_use]
pub fn withdrawn(owner: Token, call: Token) -> Event {
    Event::Withdrawn { owner, call: Box::from(call.raw().to_be_bytes()) }
}
#[must_use]
pub fn finished(owner: Token, finish: Finish) -> Event {
    Event::Finished { read: None, owner, turns: 0, spent: 0, finish }
}
#[must_use]
pub fn deliver(title: Box<[u8]>) -> Ask {
    Ask::Deliver { title, body: Box::new([]) }
}
#[must_use]
pub fn relay(input: Box<[u8]>) -> Ask {
    Ask::Relay { tool: Box::new([]), writes: false, input, deadline: Duration::from_secs(1) }
}
#[must_use]
pub fn callback(name: &[u8]) -> Token {
    Token::new(Reader::new(name).u64().expect("encoded callback"))
}

#[must_use]
pub fn bounds() -> jig_domain_host::Bounds {
    jig_domain_host::Bounds {
        models: Box::new([]),
        turns: 8,
        spend: 100,
        time: Duration::from_secs(60),
        waiting: Duration::from_secs(10),
        checks: Duration::from_secs(5),
        message_bytes: 64,
        inbox: 4,
        unacknowledged: 2,
    }
}
#[must_use]
pub fn opened() -> Event {
    Event::Opened { stop_bound: Duration::from_secs(2), workstreams: Box::new([3]), bounds: bounds() }
}

/// A full start inbox with each message at the byte bound.
#[must_use]
pub fn messages(limits: &jig_domain_host::Limits) -> Box<[jig_domain_host::StartMessage]> {
    let len = usize::try_from(limits.event_bytes).expect("test bound fits");
    (0..limits.inbox)
        .map(|name| jig_domain_host::StartMessage {
            name: Token::new(u64::from(name)),
            sender: Box::new([]),
            words: vec![0; len].into_boxed_slice(),
        })
        .collect()
}
