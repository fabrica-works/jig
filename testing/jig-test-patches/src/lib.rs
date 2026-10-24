//! The application's fixture patch owner; no test family reaches the production schema.
//! Its kept texts and patches are skein-codegen's `patch` command's, run from
//! skein at jig's pinned revision: `patch <this crate> --owner fixture --after
//! fixture-jig 1` (`--baseline` for the first text).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

#[path = "../patches/mod.rs"]
pub mod patches;

pub const NAME: &str = "fixture";

/// Application key bytes; the store adds tag 2 (store.md 4.2).
pub mod keys {
    pub const SHELVES_FROM: &[u8] = b"s";
    pub const SHELVES_TO: &[u8] = b"t";
    pub const SHELVES_ROOT: &str = "Shelf";
    pub const LABELS_FROM: &[u8] = b"l";
    pub const LABELS_TO: &[u8] = b"m";
    pub const LABELS_ROOT: &str = "Label";
}
