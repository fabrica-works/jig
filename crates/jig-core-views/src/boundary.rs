//! Opaque subjects and pacing requests (`domain/engine.md`, 11).
//! The parent owns documents, authorization and each subject's meaning.

use skein_lib::{List, Token};

/// A bounded slice of a subject's document, interpreted only by its owner.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Window {
    pub rows: u32,
    pub after: Option<u64>,
}

/// Why a watch admission was refused.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Refusal {
    /// A configured capacity is exhausted.
    Busy,
}

/// Why a watch ended.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum End {
    /// Its watcher left.
    Unwatched,
    /// Its subject left the engine.
    Gone,
    /// Its party may no longer see the subject.
    NotAllowed,
    /// Its party signed out.
    SignedOut,
    /// Its parent is stopping.
    Stopping,
}

/// The parent's result of making a document; its content stays with the parent.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Made {
    /// A whole bounded document is ready.
    Ready,
    /// No visible document remains.
    Gone { why: End },
}

/// Events from the parent.
#[derive(PartialEq, Eq, Debug)]
pub enum Event {
    /// Admit an opaque subject and party under a unique watcher token.
    Watch { watcher: Token, party: Token, subject: Token, window: Window },
    /// The parent finished changing these subjects.
    Changed { subjects: List<Token> },
    /// The parent made the requested document.
    Made { watcher: Token, outcome: Made },
    /// A delivery was consumed or dropped.
    Delivered { watcher: Token },
    /// The watcher left.
    Unwatch { watcher: Token },
    /// A subject ended or became invisible.
    Gone { subject: Token, why: End },
}

/// Requests to the parent.
#[derive(PartialEq, Eq, Debug)]
pub enum Request {
    /// The watch was admitted.
    Watching { watcher: Token },
    /// Make the current document within this watch's window.
    Make { watcher: Token, subject: Token, window: Window },
    /// Deliver the document the parent holds until durable.
    Deliver { watcher: Token },
    /// Admission found no capacity.
    Refused { watcher: Token, refusal: Refusal },
    /// The watch has released its last outstanding request.
    Ended { watcher: Token, end: End },
}
