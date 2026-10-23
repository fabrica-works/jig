//! The protocol layer's hand-outs and the hub's link (hosts.md, 6.8).
use alloc::boxed::Box;
use jig_domain_host as host;
use skein_lib::Token;
use smith_host_domain as smith;

#[derive(PartialEq, Eq, Debug)]
pub enum Event {
    Source(Box<host::Event>),
    Link(host::LinkDown),
    Ready,
    Lost,
    GracePassed,
    Resend { run: Token, attempt: Token, turn: u32 },
    RelayCancelled { run: Token, attempt: Token, call: Box<[u8]> },
    Process(Box<smith::process::Event>),
    Live { agent: Token, piece: host::Piece },
    Decoded { agent: Token, start: Result<Box<smith::Start>, Invalid> },
    Encoded { owner: Token, encoded: Encoded },
    Activity { owner: Token, activity: Option<host::Activity> },
    Shutdown,
    Timer,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Request {
    Source {
        owner: Token,
        name: host::SourceName,
        operation: host::sources::Operation,
        deadline: skein_lib::Time,
    },
    SourceFault,
    Hello(host::Hello),
    Link(host::LinkUp),
    CancelRelay {
        run: Token,
        attempt: Token,
        call: Box<[u8]>,
    },
    Stop,
    Process(Box<smith::process::Request>),
    /// The complete hub start also carries the metadata the routing must put
    /// beside its decoded charter, transcript and call names in Smith's start.
    Decode {
        agent: Token,
        run: Token,
        start: Box<host::Start>,
    },
    Encode {
        owner: Token,
        value: Encodable,
    },
    Fact {
        owner: Token,
        body: Box<[u8]>,
    },
}

#[derive(PartialEq, Eq, Debug)]
pub enum Encodable {
    Turn(smith::Turn),
    CallName(smith::CallName),
    Result(smith::RunResult),
}

#[derive(PartialEq, Eq, Debug)]
pub enum Encoded {
    Turn(host::Turn),
    CallName(Box<[u8]>),
    Result(Box<[u8]>),
    Undecodable,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Invalid {
    Charter,
    Transcript,
    CallName,
}
