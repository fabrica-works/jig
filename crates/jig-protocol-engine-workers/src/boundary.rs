//! Routing's owned vocabulary (protocol/channel.md, sections 2, 3 and 8).
use alloc::boxed::Box;
use jig_domain_host as host;
use jig_protocol_worker_wire::Part;
use skein_channel::{Lower, LowerEvent};
use skein_io::digest::Digest;
use skein_lib::{Secret, Token};
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Worker {
    pub name: Box<[u8]>,
    pub digest: Digest,
}
/// A grant's value: a credential with its scheme, or an application's
/// secret. `Withdrawn` keeps a provider account id beside a bearer token for
/// jig's live test only, until JP13 removes it.
#[derive(Clone, Debug)]
pub enum GrantValue {
    Credential { scheme: Scheme, token: Secret },
    Withdrawn { id: Secret, bearer: Secret },
    Application { secret: Secret },
}
/// How a credential is presented (protocol/channel.md, 8).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scheme {
    Bearer,
    Key,
}
#[derive(Debug)]
pub enum Event {
    Accepted {
        connection: Token,
    },
    Lower {
        connection: Token,
        event: LowerEvent,
    },
    Gone {
        connection: Token,
    },
    Down {
        channel: Token,
        down: host::LinkDown,
        parts: Box<[Part]>,
        values: Box<[Option<GrantValue>]>,
    },
    /// The fleet's busy answer, handled below the hub (protocol/channel.md, 5.2; domain/hosts.md, 6.1).
    TurnBusy {
        channel: Token,
        run: Token,
        attempt: Token,
        turn: u32,
    },
    NotAdmitted {
        channel: Token,
        text: Box<[u8]>,
    },
    Stop,
    Close,
    Abort,
}
#[derive(Debug)]
pub enum Request {
    Take { connection: Token },
    TurnAway { connection: Token },
    Lower { connection: Token, lower: Lower },
    Close { connection: Token },
    Abort { connection: Token },
    Hello { channel: Token, worker: u32, hello: host::Hello },
    Up { channel: Token, up: host::LinkUp, parts: Box<[Part]> },
    Lost { channel: Token },
    Closed,
}
