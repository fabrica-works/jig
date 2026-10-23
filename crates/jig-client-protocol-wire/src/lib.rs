//! The client contract's records for both ends (protocol/client.md, sections 3 and 9).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

#[rustfmt::skip]
#[path = "generated/v17.rs"]
pub mod v17;
pub use v17::*;

/// The client wire version this release reads and writes.
pub const VERSION: u32 = 17;

#[cfg(test)]
mod tests;
