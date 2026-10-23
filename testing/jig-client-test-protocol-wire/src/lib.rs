//! The test application's client records (protocol/client.md, 11).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
#[rustfmt::skip]
#[path = "generated/v2.rs"]
pub mod v2;
pub const VERSION: u32 = 2;

pub mod codec;
