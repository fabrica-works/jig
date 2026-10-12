//! The engine's store component (protocol/store.md, sections 2 and 3).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub mod boundary;
pub mod intervals;
pub mod keys;
pub use boundary::{
    Cursor, Event, Family, Get, Load, OpenRefusal, Request, Restore, Row, SecretOwner, Secrets, TranscriptRead, Write,
};

#[cfg(test)]
mod tests;
