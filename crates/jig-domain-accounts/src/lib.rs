//! Wanted accounts, component grants and usability (domain/engine.md, section 12).
//! The component owns refreshing, records and generation numbering. This child
//! keeps names, validity, spending and failures, and releases all wants at close.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
mod boundary;
mod domain;
mod limits;
pub use boundary::{
    Event, Fact, Failure, FromComponent, Grant, GrantTable, Place, Request, State, ToComponent, Unread,
};
pub use domain::{Domain, fire, max_out, step};
pub use limits::{Limits, worst_case};
