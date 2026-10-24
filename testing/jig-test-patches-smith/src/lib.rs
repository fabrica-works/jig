//! Fixture owner of the turn held by a transcript frame (store.md 8.6).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
#[path = "../patches/mod.rs"]
pub mod patches;
pub const NAME: &str = "fixture-smith";
pub const ROOT: &str = "Turn";
pub const FRESH: &str = "fresh-windows";
