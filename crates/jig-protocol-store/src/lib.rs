//! The engine's store component (protocol/store.md, sections 2 and 3).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub mod boundary;
pub mod codec;
pub mod intervals;

#[rustfmt::skip]
#[path = "generated/component_v1.rs"]
pub mod component_v1;
#[rustfmt::skip]
#[path = "generated/notes_v1.rs"]
pub mod notes_v1;
pub mod keys;
pub use boundary::{
    Cursor, Event, Family, Get, Load, OpenRefusal, Request, Restore, Row, SecretOwner, Secrets, TranscriptRead, Write,
};

#[cfg(test)]
mod tests;
