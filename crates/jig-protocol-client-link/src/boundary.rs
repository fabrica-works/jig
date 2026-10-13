//! Owner routing and shell exchanges (protocol/client.md, 2 and 5).
use alloc::boxed::Box;
use jig_domain_client::contract;
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
/// The owner of a small-store record (protocol/client.md, 10).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Keeper {
    Jig,
    Table,
    Family(u16),
}
/// The store version its routing owner reads at start.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StoreVersion {
    pub keeper: Keeper,
    pub version: u16,
}
/// A record the shell read with its keeper-tagged storage key.
#[derive(Debug)]
pub struct StoreRead {
    pub key: Box<[u8]>,
    pub record: Box<[u8]>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Saved {
    Jig(jig_domain_client::store::Record),
    Table { version: u16, record: RequestRecord<Ask> },
    Family { family: u16, version: u16, key: Box<[u8]>, bytes: Box<[u8]> },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Erased {
    Jig(jig_domain_client::store::StoreKey),
    Table(RequestKey),
    Family { family: u16, key: Box<[u8]> },
}
#[derive(Debug)]
pub enum Event {
    Start {
        versions: List<StoreVersion>,
        kept: List<StoreRead>,
    },
    Save {
        record: Saved,
    },
    Erase {
        key: Erased,
    },
    /// Every write and removal of this step is durable.
    Written,
    /// This step's writes failed; withhold and fail its held exchanges.
    WriteFailed,
    Send {
        attempt: Token,
        key: RequestKey,
        ask: Ask,
    },
    Read {
        read: Token,
        what: Readable,
    },
    Answered {
        exchange: Token,
        status: u16,
        body: Box<[u8]>,
    },
    Failed {
        exchange: Token,
    },
    Open {
        watch: Token,
        subject: Watchable,
    },
    Continue {
        watch: Token,
        subject: contract::Subject,
    },
    Close {
        watch: Token,
    },
    StreamOpened {
        status: u16,
    },
    /// A body piece of at most `Limits::stream_chunk` bytes.
    StreamBytes {
        bytes: Box<[u8]>,
    },
    StreamEnded,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Request {
    Write {
        key: Box<[u8]>,
        record: Box<[u8]>,
    },
    Remove {
        key: Box<[u8]>,
    },
    Kept {
        record: Saved,
    },
    Dropped {
        keeper: Keeper,
    },
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
    OpenStream,
    CloseStream,
    Watch {
        watch: Token,
        event: contract::WatchEvent,
    },
    FamilyDelivery {
        watch: Token,
        record: Family,
    },
    Ended {
        watch: Token,
        end: contract::WatchEnd,
    },
    Hello {
        family: Option<u16>,
        versions: contract::VersionRange,
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
    UndecodableEvent,
    StreamDropped,
    UndecodableStore,
}
