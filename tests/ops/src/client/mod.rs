//! Ops vocabulary adapters on the generic App/Front/World and independent referee.
mod app;
mod script;
pub mod stories;
mod transport;
pub use app::{Ops, limits};
pub use script::{Document, OpsScript, Subject};
pub use transport::Scripted;
