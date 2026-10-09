//! One page at a time, with a generation and its watch freshness.
//! Jig 20.3 adds reads, object sources and navigation ordering; jig 22 fills each
//! work page. See client/domain/client.md, 5 and 8.2.
use crate::{address::Address, contract::Limit};
pub mod conversations;
pub mod deployment;
pub mod inbox;
pub mod project;
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
