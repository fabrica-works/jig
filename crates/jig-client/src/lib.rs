//! The application-held client (client/domain/client.md, 3 and 4).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
mod client;
mod limits;
mod store;
pub use client::{Client, Event, LinkView, Request};
pub use jig_client_domain::Limits as DomainLimits;
pub use jig_client_domain::{
    Answered, Ask, Asker, Backoff, Document, Down, Family, Handle, Held, Key, Latest, Link, Outcome, Pending, Progress,
    Readable, Readout, Retry, State, Told, Up, Watchable, Why, contract,
};
pub use jig_client_protocol_link::{Message, Record, Streamed, With};
pub use limits::{Limits, Unusable, max_out, worst_case};
#[cfg(test)]
mod tests;
