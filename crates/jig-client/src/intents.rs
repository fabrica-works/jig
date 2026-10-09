//! Intents remain until the root table gives their one durable answer.
//! Joining and progress transitions land in jig 20.4; object policy is jig 21.
//! See client/domain/client.md, 6, and client/domain/root.md, 11.
use crate::contract::Ask;
use alloc::boxed::Box;
use skein_lib::Token;
/// What the root table says about an intent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Progress {
    /// One attempt is waiting for its answer.
    InFlight,
    /// Waiting to send again.
    Retrying,
    /// Waiting for the link or its party.
    Parked,
    /// Retention ended without a known outcome.
    Unknown,
}

/// A program ask, with a caller token to receive its outcome.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ByValue {
    pub asked: Token,
    pub scope: u64,
    pub ask: Box<Ask>,
}
