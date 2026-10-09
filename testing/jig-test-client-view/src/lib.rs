//! The test application's view composition (client/domain/root.md, section 9).
//! Each child owns its words and bindings; this root only routes slots and acts.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub mod bindings;
pub mod drawing;
pub mod family;
pub mod limits;
pub mod root;
pub mod words;
pub use jig_test_client_domain::Action;
pub use limits::{Limits, worst_case};
pub use root::{View, build, decode};
#[cfg(test)]
mod tests;
