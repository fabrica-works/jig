//! Client domain world and reusable shell harness (client/domain/testing.md, 4).
#![forbid(unsafe_code)]
pub mod harness;
pub mod model;
pub mod scripted;
pub mod stories;
pub mod test;
pub use harness::{App, ClientId, Front, Outcome, Settings, Until, World};
pub use scripted::Scripted;
pub use test::Test;
