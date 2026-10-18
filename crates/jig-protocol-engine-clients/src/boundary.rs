//! The protocol routing's vocabulary (protocol/client.md, sections 2 and 8).
use alloc::boxed::Box;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_views as views;
use skein_lib::{Duration, List, ReplyTo, Secret, Token, Wall};
#[derive(Debug)]
pub enum Event {
    Transport(FromTransport),
    Reply {
        to: ReplyTo,
        reply: people::Reply,
    },
    SignInRefused {
        to: ReplyTo,
    },
    SignInEnded {
        sign_in: u64,
    },
    Read {
        reply: ReplyTo,
        answer: core::reads::Answer,
    },
    Admitted {
        watcher: Token,
    },
    WatchRefused {
        watcher: Token,
        why: core::WatchRefusal,
    },
    Delivery {
        watcher: Token,
        document: Box<core::documents::Document>,
    },
    WatchEnded {
        watcher: Token,
        why: views::End,
    },
    RunText {
        watcher: Token,
        turn: views::TurnName,
        from: u32,
        to: u32,
        text: Box<[u8]>,
        cut: bool,
        call: Option<views::Call>,
    },
    Application(FromApplication),
    Secrets(SecretsAnswer),
    Mint {
        exchange: Token,
        identity: people::Identity,
    },
    Stop,
    Close,
    Abort,
}
#[derive(Debug)]
pub enum Request {
    Transport(ToTransport),
    Core(Box<core::Event>),
    Application(ToApplication),
    Secrets(SecretsRequest),
    Minted { exchange: Token, credential: Secret, expires: Wall },
    MintRefused { exchange: Token, why: MintRefusal },
    Link { secret: Secret },
    Closed,
}
#[derive(Debug)]
pub enum ToApplication {
    Request {
        reply_to: ReplyTo,
        sign_in: u64,
        key: [u8; 16],
        family: u16,
        version: u32,
        scope: people::ApplicationScope,
        record: Box<[u8]>,
        digest: [u8; 32],
    },
    Read {
        reply: ReplyTo,
        sign_in: u64,
        family: u16,
        version: u32,
        scope: people::ApplicationScope,
        window: jig_domain_core::documents::Window,
        record: Box<[u8]>,
    },
    Watch {
        watcher: Token,
        sign_in: u64,
        family: u16,
        version: u32,
        scope: people::ApplicationScope,
        window: jig_domain_core::documents::Window,
        record: Box<[u8]>,
    },
}
#[derive(Debug)]
pub enum FromApplication {
    Answered { to: ReplyTo, family: u16, record: Box<[u8]> },
    Read { reply: ReplyTo, family: u16, record: Option<Box<[u8]>> },
    Delivery { watcher: Token, family: u16, record: Box<[u8]> },
    Undecodable { to: ReplyTo },
}
#[derive(Debug)]
pub enum SecretsRequest {
    Keep { owner: SecretOwner, value: Kept },
    Erase { owner: SecretOwner },
    Get { owner: SecretOwner },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecretOwner {
    SignIn(u64),
    Service(u64),
}
#[derive(Debug)]
pub enum SecretsAnswer {
    Kept { owner: SecretOwner },
    Erased { owner: SecretOwner },
    Got { owner: SecretOwner, value: Option<Kept> },
    Failed,
}
#[derive(Clone, Debug)]
pub struct Kept {
    pub digest: Secret,
    pub expires: Wall,
}
/// Why an application's mint made no credential (protocol/client.md, 8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MintRefusal {
    NotApplication,
    Bound,
    NotReady,
    Busy,
}
#[derive(Clone, Debug)]
pub struct Settings {
    pub link: Option<people::Identity>,
    pub deployment_provider: u16,
    pub heartbeat: Duration,
    /// Default watch window from the receiving core's document limits.
    pub rows: u32,
    pub served: List<Served>,
    /// Providers of kind application, whose verified identities Mint accepts.
    pub providers: List<u16>,
    pub seed: Secret,
    pub stream: u64,
}
#[derive(Clone, Copy, Debug)]
pub struct Served {
    pub family: Option<u16>,
    pub first: u32,
    pub last: u32,
}

/// Messages from an opaque transport (protocol/client.md, 4.2).
#[derive(Debug, PartialEq)]
pub enum FromTransport {
    Exchange { exchange: Token, credential: Option<Secret>, message: Message },
    StreamOpen { stream: Token, credential: Option<Secret> },
    StreamTaken { stream: Token },
    StreamGone { stream: Token },
}
#[derive(Debug, PartialEq)]
pub enum Message {
    Request(Box<[u8]>),
    Read(Box<[u8]>),
    Watch(Box<[u8]>),
    Unwatch(Box<[u8]>),
    Gone(Box<[u8]>),
    SignInLink(Box<[u8]>),
    SignInService(Box<[u8]>),
    SignOut(Box<[u8]>),
}
#[derive(Debug)]
pub enum ToTransport {
    Reply { exchange: Token, reply: Reply },
    Streamed { stream: Token, message: Streamed },
    StreamEnded { stream: Token, why: Ended },
}
#[derive(Debug)]
pub enum Reply {
    Record(Box<[u8]>),
    NotARecord,
    Conflict,
}
#[derive(Debug)]
pub enum Streamed {
    Record(Box<[u8]>),
    Heartbeat,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ended {
    SignedOut,
    Stopping,
    Busy,
}
