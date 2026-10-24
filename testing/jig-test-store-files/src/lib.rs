//! The real store in domain worlds (protocol/store.md, 14).
#![forbid(unsafe_code)]
mod disk;
mod files;
pub use files::Files;
mod durable;
mod store;
pub use store::{Fault, Pending, Store, component_limits};
#[cfg(test)]
mod tests;
