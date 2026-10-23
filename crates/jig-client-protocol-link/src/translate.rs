//! Exhaustive owner translations (protocol/client.md, 3 and 5).
pub mod answers;
pub mod asks;
pub mod documents;
mod facts;
pub mod live;
mod llm;
mod model_sets;
mod operations;
pub(crate) mod programs;
mod task_wait;
use alloc::boxed::Box;
use jig_client_protocol_wire::v18 as w;
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
pub(crate) mod turn;

pub mod watches;

pub(crate) mod envelope;
pub mod store;

mod compound;

mod queries;

mod account_hosts;

pub mod settings;
