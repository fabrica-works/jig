//! The text shell's pure rendering and commands (client/domain/view.md, 15).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
mod render;
pub use render::{Print, render, worst_case};
#[cfg(test)]
mod tests;
