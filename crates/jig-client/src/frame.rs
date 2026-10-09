//! The shared frame, last delivered facts and selected address.
//! Frame transitions land in jig 20.2; client/domain/client.md, 3 and 8.
use crate::{address::Address, contract::Frame, link::Link};
/// A sibling hand off of who is signed in and where.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FrameShown {
    pub party: Option<u64>,
    pub address: Option<Address>,
    pub link: Link,
}

/// The current committed frame document.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct FrameState {
    pub document: Option<Frame>,
}
