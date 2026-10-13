//! The protocol routing's vocabulary (protocol/client.md, sections 2 and 8).
use alloc::boxed::Box;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_views as views;
use skein_lib::{Duration, List, ReplyTo, Secret, Token, Wall};
#[derive(Debug)]
pub enum Event {
    Io(skein_io::Event),
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
    Provider {
        exchange: Token,
        verdict: Verdict,
    },
    Stop,
    Close,
    Abort,
}
#[derive(Debug)]
pub enum Request {
    Io(skein_io::Request),
    Core(Box<core::Event>),
    Application(ToApplication),
    Secrets(SecretsRequest),
    Provider { exchange: Token, provider: u16, call: ProviderCall },
    Link { link: Box<[u8]> },
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
        record: Box<[u8]>,
        digest: [u8; 32],
    },
    Read {
        reply: ReplyTo,
        sign_in: u64,
        family: u16,
        version: u32,
        record: Box<[u8]>,
    },
    Watch {
        watcher: Token,
        sign_in: u64,
        family: u16,
        version: u32,
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
    Keep { sign_in: u64, value: Kept },
    Erase { sign_in: u64 },
    Get { sign_in: u64 },
}
#[derive(Debug)]
pub enum SecretsAnswer {
    Kept { sign_in: u64 },
    Erased { sign_in: u64 },
    Got { sign_in: u64, value: Option<Kept> },
    Failed,
}
#[derive(Clone, Debug)]
pub struct Kept {
    pub digest: Secret,
    pub expires: Wall,
}
#[derive(Debug)]
pub struct ProviderCall {
    pub target: Box<[u8]>,
    pub body: Box<[u8]>,
}
#[derive(Debug)]
pub enum Verdict {
    Identity(people::Identity),
    Refused,
}
#[derive(Clone, Debug)]
pub struct Settings {
    pub listener: skein_io::Addr,
    pub link: Option<people::Identity>,
    pub heartbeat: Duration,
    pub served: List<Served>,
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
