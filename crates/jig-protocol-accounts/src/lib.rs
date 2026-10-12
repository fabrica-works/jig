//! Accounts at the protocol boundary (protocol/llm.md, sections 5 and 6).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#[cfg(test)]
mod tests;
pub mod translate;
