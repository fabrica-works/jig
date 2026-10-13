//! The worker's channel owner (protocol/channel.md, 4 and 10.1).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
mod backoff;
mod boundary;
mod component;
mod connection;
mod limits;
pub use boundary::{Event, Request};
pub use component::{Component, Credential, Unusable};
pub use limits::{Limits, MAX_OUT_DOWN, MAX_OUT_FIRE, channel_limits, worst_case};
#[cfg(test)]
mod tests;
