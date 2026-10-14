//! What shells and people can observe, independent of a root's state
//! (client/domain/testing.md, 6). Applications translate their vocabulary here.
use super::ClientId;
use skein_lib::{Time, Token};

/// A card's stable identity in an application's public tree vocabulary.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CardKey(pub Vec<u8>);
/// A card's visible progress or departure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CardSeen {
    /// Still waiting for a person's decision.
    Waiting,
    /// A keyed decision is pending.
    Deciding,
    /// The decision's outcome is shown.
    Done,
    /// A departure is shown, with or without an explanation.
    Left { reason: bool },
}
/// Facts read from a visible accessibility tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Seen {
    /// The changed tree broke the page outline.
    Outline { fault: jig_view::outline::Fault },
    /// One card; a printed revision overrides its last delivered document's.
    Card { key: CardKey, state: CardSeen, revision: Option<u64> },
    /// Words the person is told were sent.
    Words { task: u64, message: Option<u64>, words: Vec<u8> },
    /// A visible non-work page explains why the whole previous page is hidden.
    HiddenPage,
    /// The visible link status.
    Link { live: bool },
}
/// Metadata of a keyed request at the shell's wire boundary.
#[derive(Clone, Debug)]
pub struct Sent {
    pub attempt: Token,
    pub key: [u8; 16],
    pub card: Option<CardKey>,
    pub revision: Option<(u16, u64)>,
    pub words: Option<(u64, Vec<u8>)>,
}
/// Facts in a delivered shell callback, before the root consumes it.
#[derive(Clone, Debug)]
pub enum Receipt {
    /// The document revision available to a card in the next build.
    Card { key: CardKey, revisions: Vec<(u16, u64)> },
    /// A wire attempt ended; only final answers meet the request deadline.
    Answer { attempt: Token, final_answer: bool, words: Option<(u64, u64)> },
    /// The frame confirms the party of this sign-in.
    Party { party: u64 },
    /// A newly started client can receive a fresh protocol hello.
    Started,
    /// The stream hello requires updating this client; business Frames cannot lift it.
    OutOfDate,
    /// Requests park until the party can sign in again.
    Parked,
}
/// Durable truth reported by the engine boundary, drained once.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Observed {
    /// A completed store commit and its clock.
    Durable { commit: u64, at: Time },
    /// One party's key answered at this durable commit.
    Made { party: u64, key: [u8; 16], commit: u64 },
    /// Words found in committed task activity.
    Kept { task: u64, message: u64, words: Vec<u8>, commit: u64 },
    /// A complete committed activity snapshot; omitted messages are no longer retained.
    Retained { task: u64, messages: Vec<u64>, commit: u64 },
    /// An object stopped waiting independently of this client.
    Stopped { card: CardKey, commit: u64 },
    /// The task ended, so no future word retention is promised.
    Ended { task: u64, commit: u64 },
    /// Truth's link status; compared with what each person was shown.
    Reachable { live: bool },
}
/// One boundary observation made by the loop.
#[derive(Clone, Debug)]
pub enum Observation {
    /// Admit a tab/device to link expectations.
    Client { client: ClientId },
    /// A keyed send is leaving a client's shell.
    Request { client: ClientId, party: u64, sent: Sent },
    /// A shell callback has arrived.
    Delivered { client: ClientId, receipt: Receipt },
    /// The complete visible observations after one build.
    Shown { client: ClientId, seen: Vec<Seen> },
    /// Navigation legitimately removes a page's cards.
    LeftPage { client: ClientId },
    /// The person's decoded display act changes which still-waiting cards are visible.
    WindowChanged { client: ClientId },
    /// A durable engine fact or reachability change.
    Truth(Observed),
    /// Measured client/view/model heap and its checked envelope.
    Bounded { heap: u64, bound: u64 },
}

impl CardKey {
    /// Copy a public tree key as identity, without interpreting owner tokens.
    #[must_use]
    pub fn of(key: jig_view::Key) -> Self {
        let mut bytes = Vec::new();
        match key.owner {
            jig_view::Owner::Jig => bytes.extend_from_slice(&[0, 0, 0]),
            jig_view::Owner::Application { number: connector } => {
                bytes.push(1);
                bytes.extend_from_slice(&connector.to_le_bytes());
            }
        }
        bytes.extend_from_slice(&key.kind.to_le_bytes());
        bytes.extend_from_slice(&key.a.to_le_bytes());
        bytes.extend_from_slice(&key.b.to_le_bytes());
        Self(bytes)
    }
}
