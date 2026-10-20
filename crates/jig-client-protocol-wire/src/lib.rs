//! The client contract's records for both ends (protocol/client.md, sections 3 and 9).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

#[rustfmt::skip]
#[path = "generated/v10.rs"]
pub mod v10;
pub use v10::*;

/// The client wire version this release reads and writes.
pub const VERSION: u32 = 10;

#[cfg(test)]
mod tests;
