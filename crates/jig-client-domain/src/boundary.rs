//! Client handles and exchanges (client/domain/client.md, 3 and 8).
use crate::contract;
use alloc::boxed::Box;
use skein_lib::{List, RequestEnvelope, RequestKey, RequestRecord, Token};
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Watchable {
    Jig(contract::Subject),
    Family(Family),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Readout {
    /// Refused by the client's bound on reads in flight.
    Limit,
    Jig(contract::ReadResult),
    Family {
        family: u16,
        record: Option<Box<[u8]>>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answered {
    Jig(contract::Answer),
    Family(Family),
}
/// The application child that owns a handle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Asker(pub u32);
impl Asker {
    pub const JIG: Self = Self(65536);
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Handle {
    pub asker: Asker,
    pub number: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Opening,
    Live,
    Behind,
    Ended,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Latest {
    Jig(contract::Delivery),
    Family(Family),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Document<'a> {
    pub state: State,
    pub latest: Option<&'a Latest>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Why {
    Unknown,
    NotAllowed,
    Busy,
    SubjectEnded,
    SignedOut,
    Limit,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Start { kept: List<RequestRecord<Held>> },
    Send { key: Key, request: Ask },
    Watch { handle: Handle, subject: Watchable },
    Unwatch { handle: Handle },
    Read { handle: Handle, read: Readable },
    Up { up: Up },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Up {
    Answer { attempt: Token, envelope: RequestEnvelope, answer: Option<Answered>, why: Option<Retry> },
    Watched { watch: Token, event: contract::WatchEvent },
    FamilyDelivered { watch: Token, record: Family },
    WatchEnded { watch: Token, end: contract::WatchEnd },
    ReadDone { read: Token, result: Readout },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Request {
    Down { down: Down },
    Told { told: Told },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Down {
    Save { record: RequestRecord<Held> },
    Erase { key: RequestKey },
    Send { attempt: Token, key: RequestKey, request: Ask },
    Open { watch: Token, subject: Watchable },
    Continue { watch: Token, subject: contract::Subject },
    Close { watch: Token },
    Read { read: Token, what: Readable },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Told {
    Pending { key: Key },
    Answered { key: Key, outcome: Outcome },
    Kept { key: Key, request: Ask },
    Dropped { count: u32 },
    Changed { handle: Handle },
    Text { handle: Handle },
    Ended { handle: Handle, why: Why },
    Page { handle: Handle, page: Readout },
}

/// The application's stable request name, kept beside its wire key.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Key {
    pub asker: Asker,
    pub number: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Held {
    pub key: Key,
    pub request: Ask,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Answered(Answered),
    Unknown,
    Limit,
    Unconfirmed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Progress {
    InFlight,
    Retrying { why: Retry },
    Parked,
    Unknown,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Retry {
    Busy,
    NotReady,
    Lost,
    Undecodable,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pending {
    pub progress: Progress,
    pub party: u64,
    pub owner: Option<u16>,
}
