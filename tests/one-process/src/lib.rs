//! The actual one-process service, clients and hosted agents (service.md, 8 and 10).
pub mod fixtures;
pub mod referee;
pub mod world;
mod world_signals;
pub use jig_client_protocol_world::native::Script;
