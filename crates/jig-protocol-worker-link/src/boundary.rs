//! Streams belong to the layer; the dial policy belongs here (channel.md, 10.1).
use alloc::boxed::Box;
use jig_domain_host as host;
use jig_protocol_worker_wire::Part;
use skein_channel::{Closed, Lower, LowerEvent};
use skein_lib::{Secret, Time, Token};
#[derive(Debug)]
pub enum Event {
    SourceFault,
    Hello { hello: host::Hello },
    Up { up: host::LinkUp, parts: Box<[Part]> },
    Dialled { connection: Token },
    Unreachable,
    Stop,
    Abort,
    Lower { connection: Token, event: LowerEvent },
    Gone { connection: Token },
    CancelRelay { run: Token, attempt: Token, call: Box<[u8]> },
}
/// A grant's value: a credential with its scheme, or an application's
/// secret, or a program source's name.
#[derive(Debug)]
pub enum GrantValue {
    Credential { scheme: Scheme, token: Secret },
    Application { secret: Secret },
}
/// How a credential is presented (protocol/channel.md, 8).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scheme {
    Bearer,
    Key,
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
    GracePassed,
    Closed,
    Down {
        down: host::LinkDown,
        parts: Box<[Part]>,
    },
    Value {
        account: u32,
        generation: u64,
        expires: Time,
        value: GrantValue,
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
