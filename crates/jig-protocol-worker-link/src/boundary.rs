//! Streams belong to the layer; the dial policy belongs here (channel.md, 10.1).
use alloc::boxed::Box;
use jig_domain_host as host;
use jig_protocol_worker_wire::Part;
use skein_channel::{Closed, Lower, LowerEvent};
use skein_lib::{Secret, Time, Token};
#[derive(Debug)]
pub enum Event {
    Hello { hello: host::Hello },
    Up { up: host::LinkUp, parts: Box<[Part]> },
    Dialled { connection: Token },
    Unreachable,
    Lower { connection: Token, event: LowerEvent },
    Gone { connection: Token },
    CancelRelay { run: Token, attempt: Token, call: Box<[u8]> },
}
#[derive(Debug)]
pub enum Request {
    Ready,
    RelayCancelled {
        run: Token,
        attempt: Token,
        call: Box<[u8]>,
    },
    Resend {
        run: Token,
        attempt: Token,
        turn: u32,
    },
    Lost,
    Down {
        down: host::LinkDown,
        parts: Box<[Part]>,
    },
    Value {
        table: host::GrantTable,
        account: u32,
        generation: u64,
        expires: Time,
        value: Secret,
        account_id: Secret,
    },
    Dial,
    Lower {
        connection: Token,
        lower: Lower,
    },
    Close {
        connection: Token,
    },
    Abort {
        connection: Token,
    },
    /// A content-free loss, including the refusal's operator reason.
    Fact {
        why: Closed,
    },
}
