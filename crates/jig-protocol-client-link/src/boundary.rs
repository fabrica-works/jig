//! Owner routing and shell exchanges (protocol/client.md, 2 and 5).
use alloc::boxed::Box;
use jig_domain_client::contract;
use skein_lib::{RequestEnvelope, RequestKey, Token};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Family {
    pub family: u16,
    pub version: u32,
    pub bytes: Box<[u8]>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ask {
    Jig(Box<contract::Ask>),
    Family(Family),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Readable {
    Jig(contract::Read),
    Family(Family),
}
#[derive(Debug, PartialEq, Eq)]
pub enum Readout {
    Jig(contract::ReadResult),
    Family { family: u16, record: Option<Box<[u8]>> },
}
#[derive(Debug, PartialEq, Eq)]
pub enum Answered {
    Jig(contract::Answer),
    Family(Family),
}
#[derive(Debug)]
pub enum Event {
    Send { attempt: Token, key: RequestKey, ask: Ask },
    Read { read: Token, what: Readable },
    Answered { exchange: Token, status: u16, body: Box<[u8]> },
    Failed { exchange: Token },
}
#[derive(Debug, PartialEq, Eq)]
pub enum Request {
    /// A POST; its shell adds the bearer and content type.
    Exchange {
        exchange: Token,
        path: &'static [u8],
        body: Box<[u8]>,
    },
    Abandon {
        exchange: Token,
    },
    Answer {
        attempt: Token,
        envelope: RequestEnvelope,
        answer: Option<Answered>,
    },
    /// One terminal per read.
    Read {
        read: Token,
        result: Readout,
    },
    Fact {
        fact: Fact,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fact {
    UndecodableAnswer,
    UndecodableDocument,
}
