//! Client stories through native HTTP bytes (protocol/client.md, 14).
#![forbid(unsafe_code)]
pub mod client;
mod engine;
pub mod family;
pub mod fixtures;
mod front;
pub mod net;
pub use engine::{held_event, now_event, watcher};
pub use front::Protocol;
pub use front::observe;
pub mod protocol {
    pub use crate::fixtures::world;
}

pub mod native;

pub mod referee;
pub mod stories;
