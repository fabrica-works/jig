//! The client contract's records for both ends (protocol/client.md, sections 3 and 9).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

#[rustfmt::skip]
#[path = "generated/v11.rs"]
pub mod v11;
pub use v11::*;

/// The client wire version this release reads and writes.
pub const VERSION: u32 = 11;

#[cfg(test)]
mod tests;
