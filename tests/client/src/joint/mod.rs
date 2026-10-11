//! The client and actual engine share one translation boundary
//! (client/domain/testing.md, 5). The front and real stories are added after
//! the complete document carriers; no scripted truth stands in for core facts.
pub mod archive;
pub mod engine;
pub mod front;
pub use front::Joint;
mod offered_packets;
pub mod packets;
pub mod read_packets;
pub mod referee;
pub mod requests;
pub mod session;
pub mod transport;
pub mod truth;
pub mod wire;

#[cfg(test)]
mod stories;
