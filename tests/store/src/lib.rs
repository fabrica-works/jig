//! The component's simulated crashing disk world (protocol/store.md, 14).
#![forbid(unsafe_code)]
pub mod model;
pub mod patches;
pub mod referee;
pub mod root;
pub mod world;
pub use world::{Restored, World};
