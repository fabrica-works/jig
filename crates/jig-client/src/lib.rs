//! Jig's part of a party client, a child of the application client root.
//! Keeps the frame and link, page working set, intents, reads, watches and drafts.
//! Knows no platform, application values, engine internals or wire encoding.
//! Vocabulary follows client/domain/client.md, 3–4, and contract.md, 2–8.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub mod action;
pub mod acts;
pub mod address;
pub mod boundary;
pub(crate) mod clock;
pub mod confirm;
pub mod contract;
pub mod conversation;
pub mod domain;
pub mod drafts;
pub mod facts;
pub mod fields;
pub mod frame;
pub mod intents;
pub mod limits;
pub mod link;
pub mod notices;
pub mod objects;
pub mod pages;
pub mod reads;
pub mod signin;
pub mod store;
pub mod watches;
pub mod windows;
pub use boundary::{Event, Request};
pub use domain::{Domain, fire, max_out, next_deadline, reclaim, step};
pub use limits::{Limits, worst_case};
#[cfg(test)]
mod tests;
