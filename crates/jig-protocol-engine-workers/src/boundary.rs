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
#[derive(Clone, Debug)]
pub struct GrantValue {
    pub value: Secret,
    pub account_id: Secret,
}
#[derive(Debug)]
pub enum Event {
    Accepted { connection: Token },
    Lower { connection: Token, event: LowerEvent },
    Gone { connection: Token },
    Down { channel: Token, down: host::LinkDown, parts: Box<[Part]>, values: Box<[Option<GrantValue>]> },
    NotAdmitted { channel: Token, text: Box<[u8]> },
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
    Hello { channel: Token, hello: host::Hello },
    Up { channel: Token, up: host::LinkUp, parts: Box<[Part]> },
    Lost { channel: Token },
    Closed,
}
