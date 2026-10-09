//! One page at a time, with a generation and its watch freshness.
//! Reads, object sources and parts belong to the current generation. See client/domain/client.md, 5 and 8.2.
use crate::{address::Address, contract::Limit};
pub mod conversations;
pub mod deployment;
pub mod inbox;
pub mod navigation;
pub mod project;
pub(crate) mod replies;
pub(crate) mod routing;
pub mod task;

/// The visible page, including states unrelated to work.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Page {
    /// Waiting for the frame's first delivery.
    Starting,
    /// A jig page, with its generation and freshness.
    Work { address: Address, generation: u32, behind: bool, excess: u32, refused: Option<Limit> },
    /// A page owned by another child of the root.
    Elsewhere,
    /// The sign in has ended.
    SigningIn,
    /// No visible page or object exists here.
    Missing,
    /// This client is outside the engine's versions.
    OutOfDate,
    /// Kept work belongs to another party and stays hidden.
    SomeoneElse,
}

/// The page policy awaiting a durable keyed answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IntentPurpose {
    /// A new chat whose task page opens once made.
    StartChat,
    /// A goal made or proposed from the project board.
    SetGoal,
    /// One explicitly opened read position.
    Seen(crate::contract::Seen),
}
