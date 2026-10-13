//! The real store in domain worlds (protocol/store.md, 14).
#![forbid(unsafe_code)]
mod files;
pub use files::Files;
mod store;
pub use store::{Fault, Pending, Store, component_limits};
#[cfg(test)]
mod tests;
