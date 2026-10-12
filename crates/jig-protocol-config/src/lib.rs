//! Strict configuration values and operator problems (protocol/configuration.md, 1 and 10).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub mod address;
mod document;
pub mod forms;
mod problem;
pub mod read;
pub use address::Address;
pub use document::{DocumentLimits, document};
pub use forms::{digest, secret};
pub use problem::{PROBLEMS, Path, Problem, Problems, Segment, What};
#[cfg(test)]
mod tests;
