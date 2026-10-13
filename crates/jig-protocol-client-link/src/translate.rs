//! Exhaustive owner translations (protocol/client.md, 3 and 5).
pub mod answers;
pub mod asks;
pub mod documents;
mod facts;
pub mod live;
use alloc::boxed::Box;
use jig_protocol_client_wire::v4 as w;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Undecodable {
    Malformed,
    TooLarge,
    Version,
}
pub(crate) fn too_large() -> w::Problem {
    w::Problem { path: w::Path::OwnedRecord, reason: skein_codec::Reason::Bound }
}
pub(crate) fn text_problem(_: Box<[u8]>) -> w::Problem {
    too_large()
}
pub(crate) fn piece_problem(reason: skein_codec::Reason) -> w::Problem {
    w::Problem { path: w::Path::OwnedRecord, reason }
}
pub mod objects;
pub mod pages;

pub mod reads;

pub mod watches;
