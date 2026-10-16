//! HTTP calls and printed links (protocol/client.md, 17.1, 17.3 and 17.4).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
mod call;
mod printed;
use alloc::boxed::Box;
pub use call::{Call, answered, exchange, stream};
pub use printed::{Printed, Unlinked, printed};
use skein_lib::bytes;
#[derive(Debug, PartialEq, Eq)]
pub struct Mapping {
    prefix: Box<[u8]>,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Unmapped {
    Prefix,
}
impl Mapping {
    pub fn new(prefix: &[u8]) -> Result<Self, Unmapped> {
        if prefix.len() > 64 || !prefix.starts_with(b"/") || prefix.ends_with(b"/") {
            return Err(Unmapped::Prefix);
        }
        Ok(Self { prefix: bytes::copy_of(prefix) })
    }
}
#[cfg(test)]
mod tests;
