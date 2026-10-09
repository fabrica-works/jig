//! Keep: the child/table routing shape and exhaustive owner variants.
//! Replace: Ops connector sides, family addresses and application vocabulary.
//! The children keep documents, visibility and pending acts. The root retains
//! only those children and the request table (client/domain/root.md, 3–10).
//! Each step routes one event; take saves table records before keyed sends.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub mod address;
pub mod boundary;
pub mod domain;
pub mod infrastructure;
pub mod limits;
pub mod observability;
pub mod owner;
mod parts;
mod route;
pub mod store;
pub use address::{Address, Place};
pub use boundary::{Action, Answered, Ask, Event, Fact, JigOut, Request};
pub use domain::{Domain, configure_navigation, fire, new, next_deadline, provider, shown, step, take};
pub use limits::{Limits, max_out, worst_case};
#[cfg(test)]
mod tests;
