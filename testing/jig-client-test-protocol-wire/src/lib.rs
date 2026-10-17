//! The test application's client records (protocol/client.md, 11).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
#[rustfmt::skip]
#[path = "generated/v1.rs"]
pub mod v1;
pub const VERSION: u32 = 1;

pub mod codec;
