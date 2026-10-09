//! Level-triggered volatile watches (`domain/engine.md`, 11).
//! The child keeps opaque subjects, windows and pacing marks. Its parent owns
//! documents and authorization; changes coalesce while a delivery is in flight.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
mod boundary;
mod domain;
mod facts;
mod limits;
#[cfg(test)]
mod tests;
mod text;
mod watch;
pub use boundary::{Call, End, Event, Follow, Made, Refusal, Request, TurnName, Window};
pub use domain::{Domain, fire, max_out, step};
pub use facts::Fact;
pub use limits::{Limits, worst_case};
