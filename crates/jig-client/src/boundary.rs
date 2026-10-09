//! Inputs and outputs toward the application root, no application values.
//! Each read and watch has one terminal. Sends enter the root's keyed table;
//! answers arrive only once durable. See client/domain/client.md, 4.
use crate::{
    action::Act,
    address::{Address, Place},
    contract::{Answer, Ask, Part, Read, ReadResult, Subject, WatchEnd, WatchEvent},
    facts::Fact,
    frame::FrameShown,
    intents::{ByValue, Progress},
    link::Link,
    store::{Record, StoreKey},
};
use alloc::boxed::Box;
use skein_lib::{Duration, List, Token};
/// The person offset from universal time, in minutes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Offset {
    pub minutes: i32,
}

/// One input routed by the application root.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Event {
    /// Start with this domain records and display time offset.
    Start { address: Address, kept: List<Record>, offset: Offset },
    /// The shell reports a place it went.
    Went { place: Place },
    /// A person acted on the view.
    Act { act: Act },
    /// A program asks by value.
    Ask { ask: ByValue },
    /// The table intent progress changed.
    Progress { request: Token, progress: Progress },
    /// The table handed the one durable answer.
    Answered { request: Token, answer: Answer },
    /// The one terminal of a read.
    Read { read: Token, result: ReadResult },
    /// A nonterminal watch event.
    Watch { watch: Token, event: WatchEvent },
    /// The one terminal of a watch.
    Ended { watch: Token, end: WatchEnd },
    /// A keyed request envelope ended the sign in.
    SignedOut,
    /// A sibling requests the current frame within this step.
    Frame { asked: Token },
}

/// One effect or hand off routed by the application root.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Request {
    /// Ask the root table to keep and send a request.
    Send { request: Token, scope: u64, ask: Box<Ask> },
    /// Read once without a key.
    Read { read: Token, what: Read },
    /// Open a live watch without a key.
    Open { watch: Token, subject: Subject },
    /// Close a watch and await its terminal.
    Close { watch: Token },
    /// Show a jig address in history.
    Show { address: Address, replace: bool },
    /// Save this domain record in the small store.
    Save { record: Record },
    /// Erase this domain record.
    Erase { key: StoreKey },
    /// Sign in and come back to an address.
    SignIn { provider: u16, back: Address },
    /// Have the part owner show its value.
    ShowPart { part: Part, generation: u32 },
    /// The old page no longer shows its parts.
    PartsLeft { generation: u32 },
    /// Tell siblings and the table the shared link.
    Link { link: Link },
    /// Tell the table whose parked requests may be sent.
    Confirmed { party: Option<u64> },
    /// Tell the table the engine answer retention.
    Retention { retention: Duration },
    /// Answer a sibling hand off in this step.
    FrameIs { asked: Token, frame: FrameShown },
    /// Tell a program its by value durable outcome.
    Answered { asked: Token, answer: Answer },
    /// Tell a program that retention ended without a known outcome.
    Unknown { asked: Token },
    /// Report a content free fact.
    Fact { fact: Fact },
}
