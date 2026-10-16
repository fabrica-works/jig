//! Client domain and API vocabulary (client/domain/client.md, 3).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
mod boundary;
pub mod contract;
mod domain;
pub mod limits;
mod watches;
pub use boundary::*;
pub use domain::{Domain, fire, next_deadline, reclaim, step};
pub use limits::{Backoff, Limits, max_out, worst_case};
#[cfg(test)]
mod tests;
