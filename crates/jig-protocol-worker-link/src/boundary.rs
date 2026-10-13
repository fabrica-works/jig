//! Streams belong to the layer; the dial policy belongs here (channel.md, 10.1).
use skein_channel::{Closed, Lower, LowerEvent};
use skein_lib::Token;
#[derive(Debug)]
pub enum Event {
    Dialled { connection: Token },
    Unreachable,
    Lower { connection: Token, event: LowerEvent },
    Gone { connection: Token },
}
#[derive(Debug)]
pub enum Request {
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
