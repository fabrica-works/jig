//! The client contract's records for both ends (protocol/client.md, sections 3 and 9).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

#[rustfmt::skip]
#[path = "generated/v18.rs"]
pub mod v18;
pub use v18::*;

/// The client wire version this release reads and writes.
pub const VERSION: u32 = 18;

#[cfg(test)]
mod tests;
