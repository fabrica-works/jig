//! Client attempts and envelopes (protocol/client.md, 2, 5 and 13).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub mod boundary;
mod component;
pub mod limits;
pub mod translate;
pub use boundary::*;
pub use component::Link;
pub use limits::{Limits, Unusable, worst_case};
pub use translate::Undecodable;
#[cfg(test)]
mod tests;
