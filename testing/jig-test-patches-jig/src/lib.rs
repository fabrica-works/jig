//! The fixture patch owner playing jig's slot in the store's plan.
//! Its kept texts and patches are skein-codegen's `patch` command's, run from
//! skein at jig's pinned revision: `patch <this crate> --owner fixture-jig`
//! (`--baseline` for the first text).
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
