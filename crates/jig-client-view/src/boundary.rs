//! Inputs and outputs toward the application root, no application values.
//! Each read and watch has one terminal. Sends enter the root's keyed table;
//! answers arrive only once durable. See client/domain/client.md, 4.
use crate::{
    action::Act,
    address::Place,
    frame::me::FrameShown,
    intents::ByValue,
    store::{Record, StoreKey},
};
use jig_client::Link;
use jig_client::contract::{Answer, Part};
use skein_lib::{Duration, List, Token};
/// The person offset from universal time, in minutes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Offset {
    pub minutes: i32,
}

/// How the shell enters a place in its history.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum How {
    Push,
    Replace,
}

/// A supplied presentation configuration cannot be admitted.
#[derive(Debug, PartialEq, Eq)]
pub enum Refused {
    Navigation,
    Arrangement,
}

/// One input routed by the application root.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    /// What the application-held client tells the UI.
    Client { told: jig_client::Told },
    /// Its kept records, the initial place and the seed for fresh request keys.
    Start { records: List<Record>, place: Place, seed: u64 },
    /// Restored table requests, counted once per scope after Start.
    Kept { scope: u64, requests: u32 },
    /// One restored request retired, with or without a known outcome.
    KeptGone { scope: u64 },
    /// The shell reports a place it went.
    Went { place: Place },
    /// The application left for one of its own places.
    Left,
    /// A person acted on the view.
    Act { action: Act },
    /// A program asks by value.
    Ask { ask: ByValue },
    /// The application's page names the whole list of jig objects and sections.
    Named { objects: List<jig_client::contract::ObjectKey>, sections: List<crate::page::Section> },
    /// A sibling requests the current frame within this step.
    Frame { asked: Token },
}

/// One effect or hand off routed by the application root.
#[derive(Debug, PartialEq, Eq)]
pub enum Request {
    /// An event for the application-held client, under the UI's asker.
    Client { asked: jig_client::Event },
    /// Show a jig address in history.
    Show { place: Place, how: How },
    /// Save this domain record in the small store.
    Save { records: List<Record> },
    /// Erase this domain record.
    Erase { records: List<StoreKey> },
    /// Have the part owner show its value.
    ShowPart { part: Part, scope: jig_client::contract::PartScope },
    /// The old page no longer shows its parts.
    PartsLeft,
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
}
