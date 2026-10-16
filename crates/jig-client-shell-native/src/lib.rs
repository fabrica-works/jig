//! The native client's HTTP transport (protocol/client.md, sections 8 and 12).
#![forbid(unsafe_code)]
mod connections;
mod files;
mod shell;
mod sign_in;
pub use connections::{Conn, Connections, Limits};
pub use shell::{Settings, Shell};
pub use sign_in::Unlinked;
#[cfg(test)]
mod tests;
