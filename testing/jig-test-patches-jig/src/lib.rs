//! The fixture patch owner playing jig's slot in the store's plan.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

#[path = "../patches/mod.rs"]
pub mod patches;

pub const NAME: &str = "fixture-jig";

/// Whole map keys, as for jig's own records (store.md 4.2).
pub mod keys {
    pub const NOTES_FROM: &[u8] = &[0x02, b'n'];
    pub const NOTES_TO: &[u8] = &[0x02, b'o'];
    pub const NOTES_ROOT: &str = "Note";
}
